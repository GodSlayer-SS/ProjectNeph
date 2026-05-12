/// Cancellation token shared across the voice pipeline.
///
/// The token lives for the duration of one Tauri session (i.e. the app process).
/// It is passed as an `Arc` into both the TTS playback loop and the VAD detection
/// loop so either can signal the other to stop.
///
/// Usage:
/// ```rust
/// let ct = CancelToken::new();
/// // Clone before spawning TTS thread.
/// let tts_ct = ct.clone();
/// // In VAD loop — speech detected while TTS is playing:
/// ct.cancel();
/// // In TTS loop:
/// while !tts_ct.is_cancelled() { /* write audio frames */ }
/// // After cancel:
/// ct.reset(); // Ready for the next utterance.
/// ```

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Clone)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    /// Signal cancellation.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Check if cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    /// Reset for reuse in the next turn.
    pub fn reset(&self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}
