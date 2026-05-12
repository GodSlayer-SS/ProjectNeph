/// actors/memory.rs — Memory actor (Blueprint §4).
///
/// Owns the `MemoryBus` and exposes:
/// - `inject_context(query: &str, limit: usize) -> String`
///   Returns a formatted context snippet for injection into the planner prompt.
/// - `post_session_admit(&MemoryItem)` 
///   Queues a candidate for LLM-based admission control.
///
/// Phase 1: `MemoryBus` is constructed inline. This actor provides a clean
/// typed wrapper so callers don't import `MemoryBus` directly.

use std::sync::Arc;

use crate::memory::MemoryBus;
use crate::state::AppState;
use crate::traits::memory::{MemoryItem, MemoryStore};

pub struct MemoryActor {
    bus: MemoryBus,
}

impl MemoryActor {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { bus: MemoryBus::new(state) }
    }

    /// Build a context string for injection into the planner system prompt.
    pub fn context_for(&self, query: &str, limit: usize) -> String {
        self.bus.context_for(query, limit)
    }

    /// Persist a new memory item to hot + optionally warm tier.
    pub fn remember(
        &self,
        item: &MemoryItem,
        propagate_to_warm: bool,
    ) -> anyhow::Result<crate::traits::memory::MemoryId> {
        self.bus.remember(item, propagate_to_warm)
    }

    /// Post-session: flush hot tier into warm (admission control should filter).
    pub fn flush_hot(&self) -> anyhow::Result<usize> {
        self.bus.flush_hot_to_warm()
    }
}
