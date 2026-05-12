use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::confirmation::ConfirmationStore;
use crate::models::PaletteRunResponse;
use crate::rate_limit::{DailyQuotaLimiter, SlidingWindowLimiter};
use crate::router::RoutedIntent;
use crate::tools::manifest::Manifest;
use crate::traits::planner::{Planner, PlannerCtx};

mod exports;
mod file_ops;
mod index;
mod llm_bridge;
mod memory;
mod admission;
mod notes;
pub(crate) mod runner;

/// Re-export the inner execute_plan for use by actors::executor.
pub(crate) use runner::execute_plan as run_palette_step;

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
        mut on_token: Option<&mut dyn FnMut(&str)>,
    ) -> Result<PaletteRunResponse, String> {
        // Phase 2 runtime path: Structured planner + executor.
        // Keep Phase 1 path as fallback when phase2 is disabled.
        let phase2_enabled = crate::db::read_setting(&self.db_path, "phase2_enabled")
            .map_err(|e| e.to_string())?
            .map(|v| v == "1")
            .unwrap_or(false);
        if !phase2_enabled {
            return runner::run_palette(self, input, confirmation_token.as_deref(), on_token);
        }

        let planner = crate::actors::planner::StructuredPlanner::new();
        let intent = planner.classify(input).map_err(|e| e.to_string())?;

        let manifest = Manifest::get();
        let available_tools = manifest
            .all_tools()
            .into_iter()
            .filter(|t| t.phase <= 2)
            .map(|t| t.name)
            .collect::<Vec<_>>();

        let memory_snippets = self.recall_memory(input).unwrap_or_default();
        let ctx = PlannerCtx {
            memory_snippets,
            available_tools,
        };

        let mut noop = |_chunk: &str| {};
        let mut_token: &mut dyn FnMut(&str) = on_token.as_deref_mut().unwrap_or(&mut noop);
        let plan = planner.plan(&intent, &ctx, mut_token).map_err(|e| e.to_string())?;

        // Pure chat/no-op plans still go through the stable Phase 1 runner.
        if plan.steps.is_empty() {
            return runner::run_palette(self, input, confirmation_token.as_deref(), on_token);
        }

        crate::actors::executor::execute_structured_plan(
            self,
            input,
            &plan,
            confirmation_token.as_deref(),
            on_token,
        )
    }
}
