/**
 * src/ipc/events.ts — Typed Tauri event name constants.
 *
 * Mirrors `src-tauri/src/bus.rs` — keep in sync.
 * Import these instead of bare string literals to catch typos at compile time.
 */

// ── Voice pipeline ────────────────────────────────────────────────────────────
export const EVENT_VOICE_STATE = "voice:state" as const;
export const EVENT_VOICE_AMPLITUDE = "voice:amplitude" as const;
export const EVENT_STT_PARTIAL = "stt:partial" as const;
export const EVENT_STT_FINAL = "stt:final" as const;

// ── LLM streaming ─────────────────────────────────────────────────────────────
export const EVENT_LLM_TOKEN = "llm:token" as const;
export const EVENT_LLM_DONE = "llm:done" as const;
export const EVENT_LLM_ERROR = "llm:error" as const;

// ── Latency telemetry ─────────────────────────────────────────────────────────
export const EVENT_VOICE_LATENCY = "voice:latency" as const;
export const EVENT_VOICE_LATENCY_STT = "voice:latency_stt" as const;
export const EVENT_VOICE_LATENCY_LLM = "voice:latency_llm" as const;
export const EVENT_VOICE_LATENCY_TTS = "voice:latency_tts_first_audio" as const;
