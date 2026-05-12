"""TTS — EdgeTTS streaming synthesis."""

import asyncio
import io
import logging

log = logging.getLogger("nephis_pyside.tts")

# Default voice — good quality, English, natural cadence.
DEFAULT_VOICE = "en-US-AndrewNeural"


class TtsEdge:
    """
    Synthesize text via Microsoft EdgeTTS (free, no API key needed).
    Returns raw WAV bytes that the Rust actor plays via cpal.
    """

    def __init__(self, voice: str = DEFAULT_VOICE):
        self.voice = voice
        try:
            import edge_tts  # noqa: F401
            self._available = True
            log.info("EdgeTTS initialised with voice: %s", voice)
        except ImportError:
            self._available = False
            log.warning("edge-tts not installed; TTS will return silence.")

    def synthesize(self, text: str) -> bytes:
        """Synchronous wrapper — runs the async EdgeTTS call in a fresh event loop."""
        if not self._available or not text.strip():
            return b""
        try:
            return asyncio.run(self._synthesize_async(text))
        except Exception as e:
            log.error("EdgeTTS synthesis failed: %s", e)
            return b""

    async def _synthesize_async(self, text: str) -> bytes:
        import edge_tts
        communicate = edge_tts.Communicate(text, self.voice)
        buf = io.BytesIO()
        async for chunk in communicate.stream():
            if chunk["type"] == "audio":
                buf.write(chunk["data"])
        return buf.getvalue()
