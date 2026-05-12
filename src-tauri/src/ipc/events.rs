/// ipc/events.rs — Typed Tauri event names for the frontend IPC surface.
///
/// These constants match `bus.rs` but are in `ipc/` so they can be
/// co-located with the Tauri command surface types.
///
/// TypeScript equivalents are in `src/ipc/events.ts`.

pub use crate::bus::{
    EVENT_LLM_DONE, EVENT_LLM_ERROR, EVENT_LLM_TOKEN, EVENT_STT_FINAL, EVENT_STT_PARTIAL,
    EVENT_VOICE_AMPLITUDE, EVENT_VOICE_LATENCY, EVENT_VOICE_LATENCY_LLM, EVENT_VOICE_LATENCY_STT,
    EVENT_VOICE_LATENCY_TTS, EVENT_VOICE_STATE,
};
