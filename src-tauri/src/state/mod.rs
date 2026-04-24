use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::confirmation::ConfirmationStore;
use crate::models::PaletteRunResponse;
use crate::rate_limit::{DailyQuotaLimiter, SlidingWindowLimiter};
use crate::router::RoutedIntent;

mod exports;
mod file_ops;
mod index;
mod llm_bridge;
mod memory;
mod notes;
mod runner;

/// Application state: DB path, active provider, intent cache, and token usage.
pub struct AppState {
    pub db_path: PathBuf,
    /// True when sqlite-vec extension and vec0 virtual tables were created successfully at startup.
    pub sqlite_vec_loaded: AtomicBool,
    pub provider: Mutex<String>,
    pub command_cache: Mutex<Vec<(String, RoutedIntent)>>,
    pub token_usage: Mutex<(u64, u64)>,
    pub confirmations: Mutex<ConfirmationStore>,
    pub palette_limiter: Mutex<SlidingWindowLimiter>,
    pub provider_quota: DailyQuotaLimiter,
}

impl AppState {
    pub fn new(db_path: PathBuf, meta: crate::db::DbInitMeta) -> Self {
        let state = Self {
            db_path,
            sqlite_vec_loaded: AtomicBool::new(meta.sqlite_vec_loaded),
            provider: Mutex::new("groq".to_string()),
            command_cache: Mutex::new(Vec::new()),
            token_usage: Mutex::new((0, 0)),
            confirmations: Mutex::new(ConfirmationStore::new()),
            palette_limiter: Mutex::new(SlidingWindowLimiter::per_minute(120)),
            provider_quota: DailyQuotaLimiter::with_defaults(),
        };
        let _ = state.initial_scan_apps();
        let _ = state.reembed_existing_memories();
        state
    }

    pub(crate) fn connect(&self) -> anyhow::Result<Connection> {
        Ok(Connection::open(&self.db_path)?)
    }

    pub fn run_palette_command(
        &self,
        input: &str,
        confirmation_token: Option<String>,
        on_token: Option<&mut dyn FnMut(&str)>,
    ) -> Result<PaletteRunResponse, String> {
        runner::run_palette(self, input, confirmation_token.as_deref(), on_token)
    }
}
