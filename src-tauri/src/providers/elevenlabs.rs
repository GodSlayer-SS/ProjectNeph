/// providers/elevenlabs.rs — ElevenLabs Flash TTS provider (Blueprint §3, Phase 4).
///
/// Blueprint §3: "TTS: EdgeTTS (free, fast) primary; Piper local; ElevenLabs Flash optional"
/// Blueprint Phase 4: "ElevenLabs Flash voice option"
///
/// Phase 4 stub — returns an informative error until:
///   1. An ElevenLabs API key is saved via `>set-provider elevenlabs`
///   2. The `elevenlabs` feature is enabled in settings
///
/// When implemented: calls `https://api.elevenlabs.io/v1/text-to-speech/{voice_id}/stream`
/// with the Flash model (`eleven_flash_v2_5`) for ~75ms TTFB.

use anyhow::{bail, Result};

/// ElevenLabs Flash streaming TTS (Phase 4).
pub struct ElevenLabsProvider {
    pub voice_id: String,
}

impl ElevenLabsProvider {
    /// Default voice: "Rachel" (natural English, low latency with Flash model).
    pub fn new() -> Self {
        Self {
            voice_id: "21m00Tcm4TlvDq8ikWAM".to_string(),
        }
    }

    pub fn with_voice(voice_id: impl Into<String>) -> Self {
        Self { voice_id: voice_id.into() }
    }

    /// Synthesize `text` to audio bytes using ElevenLabs Flash model.
    ///
    /// Phase 4 implementation note:
    ///   POST https://api.elevenlabs.io/v1/text-to-speech/{voice_id}/stream
    ///   Headers: xi-api-key: {key}, Content-Type: application/json
    ///   Body: { "text": "...", "model_id": "eleven_flash_v2_5",
    ///           "voice_settings": { "stability": 0.5, "similarity_boost": 0.75 } }
    pub fn synthesize(&self, _text: &str, _api_key: &str) -> Result<Vec<u8>> {
        bail!(
            "ElevenLabs TTS is a Phase 4 feature. \
             Set an ElevenLabs API key and enable it in Settings → AI → Voice."
        )
    }
}

impl Default for ElevenLabsProvider {
    fn default() -> Self {
        Self::new()
    }
}
