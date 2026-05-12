use anyhow::Result;
use serde::{Deserialize, Serialize};

// ── Audio primitives ──────────────────────────────────────────────────────────

/// Raw PCM f32 samples with metadata.
#[derive(Debug, Clone)]
pub struct AudioChunk {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

/// A stream of audio chunks from the microphone. In Phase 1 this is a
/// `std::sync::mpsc::Receiver<AudioChunk>` wrapped for convenience.
/// Future phases may upgrade to `tokio::sync::mpsc`.
pub type AudioStream = std::sync::mpsc::Receiver<AudioChunk>;

// ── STT Events ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SttEvent {
    /// A partial (non-final) transcript fragment.
    Partial { text: String },
    /// The final transcript for this utterance.
    Final { text: String, confidence: f32 },
    /// An error from the STT provider.
    Error { message: String },
}

// ── The Trait ─────────────────────────────────────────────────────────────────

/// Anything that does speech-to-text.
/// Phase 1 implementations:
///   - `GroqWhisperStt`  — cloud, ~250 ms, primary
///   - `FasterWhisperStt` — local CPU int8, ~600 ms, fallback (Python sidecar)
pub trait SttProvider: Send + Sync {
    fn name(&self) -> &str;

    /// Transcribe a batch of audio samples.  `on_event` is called with each
    /// `SttEvent::Partial` and the terminal `SttEvent::Final`.
    fn transcribe(
        &self,
        audio: AudioChunk,
        on_event: &mut dyn FnMut(SttEvent),
    ) -> Result<String>;
}
