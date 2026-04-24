//! Cold-start timing anchors (first palette infra ready, first LLM completion).
//!
//! Plan targets (advisory): hotkey infra ≤500ms, first LLM completion ≤1500ms from process start.
//! Violations emit `tracing::warn!` for support bundles / perf tuning — not hard failures.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

static SETUP_START: OnceLock<Instant> = OnceLock::new();
static FIRST_AI_LOGGED: AtomicBool = AtomicBool::new(false);

const HOTKEY_READY_TARGET_MS: u64 = 500;
const FIRST_LLM_TARGET_MS: u64 = 1500;

pub fn record_setup_start() {
    let _ = SETUP_START.set(Instant::now());
}

pub fn log_palette_infra_ready(label: &str) {
    if let Some(t0) = SETUP_START.get() {
        let ms = t0.elapsed().as_millis() as u64;
        tracing::info!(
            target: "neph_startup",
            phase = label,
            ms,
            target_ms = HOTKEY_READY_TARGET_MS,
            "palette infrastructure ready"
        );
        if ms > HOTKEY_READY_TARGET_MS {
            tracing::warn!(
                target: "neph_startup",
                phase = label,
                ms,
                target_ms = HOTKEY_READY_TARGET_MS,
                "cold-start budget exceeded (hotkey / palette infra)"
            );
        }
    }
}

pub fn log_first_llm_completion_if_needed() {
    if FIRST_AI_LOGGED.swap(true, Ordering::SeqCst) {
        return;
    }
    if let Some(t0) = SETUP_START.get() {
        let ms = t0.elapsed().as_millis() as u64;
        tracing::info!(
            target: "neph_startup",
            phase = "first_llm_completion",
            ms,
            target_ms = FIRST_LLM_TARGET_MS,
            "first LLM completion since process start"
        );
        if ms > FIRST_LLM_TARGET_MS {
            tracing::warn!(
                target: "neph_startup",
                phase = "first_llm_completion",
                ms,
                target_ms = FIRST_LLM_TARGET_MS,
                "cold-start budget exceeded (first LLM completion)"
            );
        }
    }
}
