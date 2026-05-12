use anyhow::Result;

// ── TTS primitives ───────────────────────────────────────────────────────────

/// A chunk of synthesised audio — raw PCM or a container format.
#[derive(Debug, Clone)]
pub struct AudioChunk {
    pub data: Vec<u8>,
    /// MIME hint, e.g. "audio/wav", "audio/mp3".
    pub format: String,
}

/// A stream of text fragments produced by the planner / LLM.
pub type TextStream = std::sync::mpsc::Receiver<String>;

// ── The Trait ─────────────────────────────────────────────────────────────────

/// Anything that does text-to-speech.
/// Phase 1 implementations:
///   - `EdgeTtsSynthesizer` — free, decent quality, streaming (Python sidecar)
///   - `PiperSynthesizer`   — fully local, fast (Python sidecar, Phase 2)
pub trait TtsProvider: Send + Sync {
    fn name(&self) -> &str;

    /// Synthesize `text` and call `on_chunk` for each audio chunk as it arrives.
    /// The caller is responsible for playing chunks in order.
    fn synthesize(
        &self,
        text: &str,
        on_chunk: &mut dyn FnMut(AudioChunk),
    ) -> Result<()>;
}
