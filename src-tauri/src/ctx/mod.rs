pub mod active_window;
pub mod clipboard;
pub mod ocr;
pub mod screenshot;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ContextSnapshot {
    pub active_window_title: String,
    pub active_process_name: String,
    pub clipboard_preview: String,
}

pub fn collect_snapshot() -> ContextSnapshot {
    ContextSnapshot {
        active_window_title: active_window::active_window_title(),
        active_process_name: active_window::active_process_name(),
        clipboard_preview: clipboard::clipboard_preview(),
    }
}
