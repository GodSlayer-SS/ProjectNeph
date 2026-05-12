"""VAD — Silero Voice Activity Detection (CPU, sub-ms per chunk)."""

import logging
import struct

log = logging.getLogger("nephis_pyside.vad")

SAMPLE_RATE = 16000
CHUNK_SAMPLES = 512  # ~32ms at 16kHz — Silero's required chunk size


class VadSilero:
    """
    Wrapper around Silero VAD.
    process() returns True if speech is detected in the audio chunk.
    """

    def __init__(self):
        self._model = None
        self._utils = None
        try:
            import torch
            model, utils = torch.hub.load(
                repo_or_dir="snakers4/silero-vad",
                model="silero_vad",
                force_reload=False,
                onnx=True,  # ONNX is faster and doesn't require a full torch install
            )
            self._model = model
            self._utils = utils
            log.info("Silero VAD loaded (ONNX).")
        except Exception as e:
            log.warning("Silero VAD unavailable: %s — using energy threshold fallback.", e)

    def process(self, audio_bytes: bytes, sample_rate: int = SAMPLE_RATE) -> bool:
        """Return True if this chunk contains speech."""
        if self._model is not None:
            return self._silero_detect(audio_bytes, sample_rate)
        return self._energy_detect(audio_bytes)

    def _silero_detect(self, audio_bytes: bytes, sample_rate: int) -> bool:
        try:
            import torch
            samples = len(audio_bytes) // 2
            audio_np = [
                struct.unpack_from("<h", audio_bytes, i * 2)[0] / 32768.0
                for i in range(samples)
            ]
            tensor = torch.tensor(audio_np, dtype=torch.float32)
            confidence = self._model(tensor, sample_rate).item()
            return confidence > 0.5
        except Exception as e:
            log.warning("Silero inference error: %s", e)
            return self._energy_detect(audio_bytes)

    def _energy_detect(self, audio_bytes: bytes) -> bool:
        """Simple RMS energy threshold fallback (no ML required)."""
        if not audio_bytes:
            return False
        samples = len(audio_bytes) // 2
        total = sum(
            struct.unpack_from("<h", audio_bytes, i * 2)[0] ** 2
            for i in range(samples)
        )
        rms = (total / samples) ** 0.5 if samples else 0
        return rms > 500  # empirical threshold for 16-bit PCM
