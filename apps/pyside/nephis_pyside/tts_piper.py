"""TTS — Piper local TTS (Phase 2 stub).

Piper is a fast, local neural text-to-speech engine that runs on CPU.
It requires no API key and produces high-quality voice output offline.

Phase 1: Returns empty bytes — EdgeTTS (cloud) is the primary TTS.
Phase 2: Implement when:
  1. EdgeTTS latency or reliability is measured to be a problem.
  2. OR the user explicitly requests offline TTS.

Phase 2 implementation plan:
  1. `pip install piper-tts`
  2. Download a voice model: `python -m piper --download-voice en_US-amy-medium`
  3. Replace `synthesize()` with: `piper.Voice(model_path).synthesize(text) -> bytes (WAV)`

Reference: https://github.com/rhasspy/piper
"""

import logging

log = logging.getLogger("nephis_pyside.tts_piper")


class TtsPiper:
    """
    Piper local TTS — Phase 2 stub.

    Phase 1: synthesize() returns empty bytes (EdgeTTS is used instead).
    Phase 2: Replace with real Piper voice synthesis.
    """

    def __init__(self, model_path: str | None = None):
        self.model_path = model_path
        self._available = False

        if model_path is not None:
            try:
                import piper  # noqa: F401
                self._available = True
                log.info("TtsPiper initialised with model: %s", model_path)
            except ImportError:
                log.info(
                    "piper-tts not installed. Phase 2 stub active. "
                    "Install with: pip install piper-tts"
                )
        else:
            log.debug("TtsPiper: no model_path provided. Phase 2 stub active.")

    def synthesize(self, text: str) -> bytes:
        """
        Phase 1: always returns empty bytes (EdgeTTS handles TTS).
        Phase 2: returns WAV bytes from Piper neural synthesis.
        """
        if not self._available or not text.strip():
            return b""

        try:
            import piper
            voice = piper.PiperVoice.load(self.model_path)
            import io
            buf = io.BytesIO()
            with wave := voice.synthesize_stream_raw(text):
                for chunk in wave:
                    buf.write(chunk)
            return buf.getvalue()
        except Exception as e:
            log.error("Piper synthesis failed: %s", e)
            return b""
