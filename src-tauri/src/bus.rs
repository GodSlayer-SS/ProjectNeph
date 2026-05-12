/// bus.rs — In-process typed event bus (Blueprint §4).
///
/// Phase 1: Thin passthrough — actors communicate via Tauri's own event bus
/// (`AppHandle::emit`). This module documents the intended event taxonomy and
/// provides typed constants so typos are caught at compile time.
///
/// Phase 2: Replace with a proper pub/sub bus (e.g. `tokio::sync::broadcast`)
/// so actors can subscribe without going through the WebView2 IPC round-trip.
///
/// Usage today: import `bus::EVENT_*` constants instead of bare string literals.

// ── Voice pipeline events ─────────────────────────────────────────────────────

/// Emitted by VoiceActor: current voice FSM state (VoiceState JSON).
pub const EVENT_VOICE_STATE: &str = "voice:state";

/// Emitted by VoiceActor: RMS amplitude for orb animation (f32).
pub const EVENT_VOICE_AMPLITUDE: &str = "voice:amplitude";

/// Emitted by STT: partial transcript text (SttPartial JSON).
pub const EVENT_STT_PARTIAL: &str = "stt:partial";

/// Emitted by STT: final confirmed transcript (SttFinal JSON).
pub const EVENT_STT_FINAL: &str = "stt:final";

// ── LLM streaming events ──────────────────────────────────────────────────────

/// Emitted by PlannerActor/TTS listener: a streamed token chunk (String).
pub const EVENT_LLM_TOKEN: &str = "llm:token";

/// Emitted when the full LLM response is complete (String with final text).
pub const EVENT_LLM_DONE: &str = "llm:done";

/// Emitted on LLM error (String with message).
pub const EVENT_LLM_ERROR: &str = "llm:error";

// ── Latency telemetry events ──────────────────────────────────────────────────

/// Emitted after STT completes: STT latency in ms (u128).
pub const EVENT_VOICE_LATENCY_STT: &str = "voice:latency_stt";

/// Emitted after first LLM token: LLM TTFT in ms (u128).
pub const EVENT_VOICE_LATENCY_LLM: &str = "voice:latency_llm";

/// Emitted after first TTS audio frame: TTS first-audio ms (u128).
pub const EVENT_VOICE_LATENCY_TTS: &str = "voice:latency_tts_first_audio";

/// Emitted after full pipeline: VoiceLatency JSON (total, stt_ms, llm_ms).
pub const EVENT_VOICE_LATENCY: &str = "voice:latency";
