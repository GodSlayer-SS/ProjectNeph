/// memory/warm.rs — Phase 2: SQLite-backed warm memory tier.
///
/// This is a thin adapter over the existing `state/memory.rs` methods so that
/// the new `MemoryStore` trait can be used uniformly. No logic is duplicated —
/// `AppState::recall_memory`, `save_memory`, and `delete_memory` are re-used
/// verbatim through a shared `Arc<AppState>`.
///
/// Search delegates to `AppState::recall_memory` (lexical + sqlite-vec ANN + decay,
/// merged with LanceDB cold tier). File-name FTS5 lives under `db/migrations` for
/// file tools; optional Python reranker is not wired yet.

use std::sync::Arc;

use anyhow::Result;

use crate::state::AppState;
use crate::traits::memory::{MemoryHit, MemoryId, MemoryItem, MemoryStore, MemoryTier, Query};

pub struct WarmMemory {
    state: Arc<AppState>,
}

impl WarmMemory {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

impl MemoryStore for WarmMemory {
    fn tier(&self) -> MemoryTier {
        MemoryTier::Warm
    }

    fn search(&self, q: &Query) -> Result<Vec<MemoryHit>> {
        // Delegates to AppState::recall_memory which already does
        // lexical + cosine similarity + decay scoring.
        let results = self.state.recall_memory(&q.text)?;
        let hits = results
            .into_iter()
            .take(q.limit)
            .enumerate()
            .map(|(i, content)| MemoryHit {
                id: i as i64,    // warm tier doesn't expose rowid from recall; id is rank
                content,
                kind: "memory".into(),
                score: 1.0 - i as f32 * 0.1, // decaying score by rank
                tier: MemoryTier::Warm,
            })
            .collect();
        Ok(hits)
    }

    fn store(&self, item: &MemoryItem) -> Result<MemoryId> {
        self.state.save_memory(&item.kind, &item.content)?;
        // save_memory doesn't return the new rowid — return 0 as sentinel.
        Ok(0)
    }

    fn forget(&self, id: MemoryId) -> Result<()> {
        let deleted = self.state.delete_memory(id)?;
        if !deleted {
            anyhow::bail!("memory item {} not found or already deleted", id);
        }
        Ok(())
    }
}
