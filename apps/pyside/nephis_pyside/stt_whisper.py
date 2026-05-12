"""STT — Groq Whisper API (primary) with faster-whisper CPU fallback."""

import io
import logging
import os
import tempfile

log = logging.getLogger("nephis_pyside.stt")

# ── Groq Whisper (primary, ~250ms) ────────────────────────────────────────────

class GroqWhisperClient:
    def __init__(self):
        try:
            from groq import Groq
            self._client = Groq(api_key=os.environ.get("GROQ_API_KEY", ""))
            log.info("GroqWhisperClient initialised.")
        except Exception as e:
            log.warning("Groq client unavailable: %s", e)
            self._client = None

    def transcribe(self, audio_bytes: bytes, sample_rate: int = 16000) -> str | None:
        if self._client is None:
            return None
        try:
            # Groq expects a file-like object — write to a temp wav file.
            with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as f:
                _write_wav(f, audio_bytes, sample_rate)
                tmp_path = f.name
            with open(tmp_path, "rb") as f:
                result = self._client.audio.transcriptions.create(
                    file=("audio.wav", f, "audio/wav"),
                    model="whisper-large-v3-turbo",
                    response_format="text",
                )
            os.unlink(tmp_path)
            return str(result).strip()
        except Exception as e:
            log.warning("Groq Whisper failed: %s", e)
            return None


# ── faster-whisper CPU fallback (~600ms) ─────────────────────────────────────

class FasterWhisperClient:
    def __init__(self):
        try:
            from faster_whisper import WhisperModel
            self._model = WhisperModel("small", device="cpu", compute_type="int8")
            log.info("faster-whisper model loaded (CPU int8).")
        except Exception as e:
            log.warning("faster-whisper unavailable: %s", e)
            self._model = None

    def transcribe(self, audio_bytes: bytes, sample_rate: int = 16000) -> str | None:
        if self._model is None:
            return None
        try:
            import numpy as np
            audio_np = (
                np.frombuffer(audio_bytes, dtype=np.int16).astype(np.float32) / 32768.0
            )
            segments, _ = self._model.transcribe(audio_np, beam_size=5, language="en")
            return " ".join(s.text for s in segments).strip()
        except Exception as e:
            log.warning("faster-whisper transcribe failed: %s", e)
            return None

    def transcribe_with_partials(
        self, audio_bytes: bytes, sample_rate: int = 16000
    ) -> tuple[list[str], str] | None:
        """
        Return partial segment texts plus the final joined transcript.

        Note: this is not real-time streaming; it's incremental segments produced
        during transcription, suitable for emitting multiple `stt:partial` events
        on the Rust side.
        """
        if self._model is None:
            return None
        try:
            import numpy as np

            audio_np = (
                np.frombuffer(audio_bytes, dtype=np.int16).astype(np.float32) / 32768.0
            )
            segments, _ = self._model.transcribe(audio_np, beam_size=5, language="en")
            partials: list[str] = []
            texts: list[str] = []
            for s in segments:
                t = (s.text or "").strip()
                if not t:
                    continue
                texts.append(t)
                partials.append(" ".join(texts).strip())
            final = " ".join(texts).strip()
            return partials, final
        except Exception as e:
            log.warning("faster-whisper transcribe_with_partials failed: %s", e)
            return None


# ── Unified SttWhisper ────────────────────────────────────────────────────────

class SttWhisper:
    """Tries Groq Whisper first; falls back to faster-whisper CPU if unavailable."""

    def __init__(self):
        self._groq = GroqWhisperClient()
        self._local = FasterWhisperClient()

    def transcribe(self, audio_bytes: bytes, sample_rate: int = 16000) -> str:
        result = self._groq.transcribe(audio_bytes, sample_rate)
        if result:
            return result
        result = self._local.transcribe(audio_bytes, sample_rate)
        if result:
            return result
        return ""

    def transcribe_with_partials(self, audio_bytes: bytes, sample_rate: int = 16000) -> tuple[list[str], str]:
        """
        Return (partials, final_text).

        Groq Whisper is treated as non-streaming: partials will be empty.
        """
        result = self._groq.transcribe(audio_bytes, sample_rate)
        if result:
            return [], result
        local = self._local.transcribe_with_partials(audio_bytes, sample_rate)
        if local:
            return local
        # Fall back to the non-partial path (if available) to avoid total failure.
        fallback = self._local.transcribe(audio_bytes, sample_rate) or ""
        return [], fallback


# ── WAV writer helper ─────────────────────────────────────────────────────────

def _write_wav(f, pcm_bytes: bytes, sample_rate: int, channels: int = 1, bits: int = 16):
    """Write a minimal RIFF WAV header followed by PCM data."""
    import struct
    data_len = len(pcm_bytes)
    f.write(b"RIFF")
    f.write(struct.pack("<I", 36 + data_len))
    f.write(b"WAVE")
    f.write(b"fmt ")
    f.write(struct.pack("<I", 16))           # chunk size
    f.write(struct.pack("<H", 1))            # PCM
    f.write(struct.pack("<H", channels))
    f.write(struct.pack("<I", sample_rate))
    f.write(struct.pack("<I", sample_rate * channels * bits // 8))
    f.write(struct.pack("<H", channels * bits // 8))
    f.write(struct.pack("<H", bits))
    f.write(b"data")
    f.write(struct.pack("<I", data_len))
    f.write(pcm_bytes)
