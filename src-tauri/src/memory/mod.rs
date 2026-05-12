/// memory/mod.rs — unified cross-tier memory search.
///
/// The `MemoryBus` searches Hot → Warm in priority order, merges results,
/// deduplicates by content hash, and returns the top-k across tiers.
///
/// Cold tier is LanceDB (`memory/cold.rs`): vector search merged with warm recall
/// in `AppState::recall_memory`.

pub mod hot;
pub mod warm;
pub mod cold;
pub mod procedural;
pub mod admission;

use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;

use crate::traits::memory::{MemoryHit, MemoryId, MemoryItem, MemoryStore, MemoryTier, Query};
use hot::HotMemory;
use warm::WarmMemory;
use cold::ColdMemory;

// ── MemoryBus ─────────────────────────────────────────────────────────────────

/// Unified memory access layer. Combines Hot + Warm + Cold (LanceDB).
///
/// Phase 2 role: inject context into every planner prompt and admit new
/// memories from the voice session into the warm tier.
pub struct MemoryBus {
    pub hot: HotMemory,
    pub warm: WarmMemory,
    pub cold: ColdMemory,
}

impl MemoryBus {
    pub fn new(state: Arc<crate::state::AppState>) -> Self {
        Self {
            hot: HotMemory::new(),
            warm: WarmMemory::new(state),
            cold: ColdMemory::new(),
        }
    }

    /// Search Hot then Warm; merge, deduplicate by content, return top-k.
    pub fn search(&self, q: &Query) -> Result<Vec<MemoryHit>> {
        let mut hot_hits = self.hot.search(q).unwrap_or_default();
        let mut warm_hits = self.warm.search(q).unwrap_or_default();
        let mut cold_hits = self.cold.search(q).unwrap_or_default();

        // Deduplicate by content (hot wins on collisions).
        let mut seen: HashSet<String> = hot_hits.iter().map(|h| h.content.clone()).collect();
        warm_hits.retain(|h| seen.insert(h.content.clone()));
        cold_hits.retain(|h| seen.insert(h.content.clone()));

        hot_hits.append(&mut warm_hits);
        hot_hits.append(&mut cold_hits);
        hot_hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        hot_hits.truncate(q.limit);
        Ok(hot_hits)
    }

    /// Build a concise context string for injection into system prompts.
    /// Returns empty string if nothing relevant found.
    pub fn context_for(&self, query: &str, limit: usize) -> String {
        // 1. Hot — most recent session facts.
        let hot_snippet = self.hot.context_snippet(limit / 2 + 1);

        // 2. Warm — relevant long-term memories.
        let q = Query { text: query.into(), embedding: None, limit: limit / 2, kind_filter: None };
        let warm_snippet = self.warm
            .search(&q)
            .unwrap_or_default()
            .into_iter()
            .map(|h| format!("• [{}] {}", h.kind, h.content))
            .collect::<Vec<_>>()
            .join("\n");

        let parts: Vec<&str> = [hot_snippet.as_str(), warm_snippet.as_str()]
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect();

        if parts.is_empty() {
            String::new()
        } else {
            format!("### Context\n{}", parts.join("\n"))
        }
    }

    /// Persist a new memory to hot (immediate) and warm (async) tiers.
    /// `propagate_to_warm` — set false during a session, true at admission time.
    pub fn remember(&self, item: &MemoryItem, propagate_to_warm: bool) -> Result<MemoryId> {
        let id = self.hot.store(item)?;
        if propagate_to_warm {
            self.warm.store(item)?;
        }
        Ok(id)
    }

    /// Post-session admission: flush hot tier to warm for any item worth keeping.
    /// Called by the admission control flow (Phase 2.5).
    pub fn flush_hot_to_warm(&self) -> Result<usize> {
        let items = self.hot.drain();
        let n = items.len();
        for item in &items {
            let warm_item = MemoryItem { tier: MemoryTier::Warm, ..item.clone() };
            let _ = self.warm.store(&warm_item);
        }
        Ok(n)
    }
}
