/// actors/ui_bridge.rs — UI bridge actor (Blueprint §4).
///
/// Owns all Tauri event emissions that update the frontend UI state.
/// Centralises event emission so the rest of the Rust code never calls
/// `app.emit()` with bare string literals — they call typed methods here.
///
/// Phase 1: Thin wrappers over `AppHandle::emit` using `bus::EVENT_*` constants.
/// Phase 2: Add buffering, debounce for high-frequency events (voice:amplitude),
///          and frontend-state reconciliation.

use tauri::{AppHandle, Emitter};

use crate::actors::voice::{SttFinal, SttPartial, VoiceAmplitude, VoiceLatency, VoiceState};
use crate::bus;

pub struct UiBridge {
    app: AppHandle,
}

impl UiBridge {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    // ── Voice state ───────────────────────────────────────────────────────────

    pub fn emit_voice_state(&self, state: VoiceState) {
        let _ = self.app.emit(bus::EVENT_VOICE_STATE, state);
    }

    pub fn emit_voice_amplitude(&self, amplitude: VoiceAmplitude) {
        let _ = self.app.emit(bus::EVENT_VOICE_AMPLITUDE, amplitude);
    }

    // ── STT ──────────────────────────────────────────────────────────────────

    pub fn emit_stt_partial(&self, partial: SttPartial) {
        let _ = self.app.emit(bus::EVENT_STT_PARTIAL, partial);
    }

    pub fn emit_stt_final(&self, final_text: SttFinal) {
        let _ = self.app.emit(bus::EVENT_STT_FINAL, final_text);
    }

    // ── LLM streaming ────────────────────────────────────────────────────────

    pub fn emit_llm_token(&self, chunk: &str) {
        let _ = self.app.emit(bus::EVENT_LLM_TOKEN, chunk);
    }

    pub fn emit_llm_done(&self, output: &str) {
        let _ = self.app.emit(bus::EVENT_LLM_DONE, output);
    }

    pub fn emit_llm_error(&self, message: &str) {
        let _ = self.app.emit(bus::EVENT_LLM_ERROR, message);
    }

    // ── Latency telemetry ─────────────────────────────────────────────────────

    pub fn emit_latency(&self, payload: VoiceLatency) {
        let _ = self.app.emit(bus::EVENT_VOICE_LATENCY, payload);
    }

    pub fn emit_stt_ms(&self, ms: u128) {
        let _ = self.app.emit(bus::EVENT_VOICE_LATENCY_STT, ms);
    }

    pub fn emit_llm_ms(&self, ms: u128) {
        let _ = self.app.emit(bus::EVENT_VOICE_LATENCY_LLM, ms);
    }

    pub fn emit_tts_first_audio_ms(&self, ms: u128) {
        let _ = self.app.emit(bus::EVENT_VOICE_LATENCY_TTS, ms);
    }
}
