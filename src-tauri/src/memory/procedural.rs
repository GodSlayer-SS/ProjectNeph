/// memory/procedural.rs — Procedural (skills) memory store (Blueprint §4, §7).
///
/// Stores how-to knowledge: named procedures and step sequences that Nephis
/// has learned from previous task executions or user-defined skill files.
///
/// Architecture:
/// - Skills loaded from `~/.nephis/skills/*.toml` (see `skills.rs`)
/// - Procedural memories stored in SQLite `memory` table with `kind = 'procedural'`
/// - Retrieval: exact name match, then keyword FTS
///
/// Phase 1: Read from warm memory with `kind = 'procedural'` filter.
/// Phase 2: Skill rehearsal — automatically re-run a stored skill and update the
///          procedure if the outcome changed.

use anyhow::Result;

use crate::traits::memory::{MemoryHit, MemoryId, MemoryItem, MemoryStore, MemoryTier, Query};

/// Procedural memory: wraps warm memory with `kind = 'procedural'` filtering.
pub struct ProceduralMemory {
    warm: super::warm::WarmMemory,
}

impl ProceduralMemory {
    pub fn new(state: std::sync::Arc<crate::state::AppState>) -> Self {
        Self { warm: super::warm::WarmMemory::new(state) }
    }

    /// Store a procedure (skill steps) in long-term memory.
    pub fn store_procedure(&self, name: &str, steps_yaml: &str) -> Result<MemoryId> {
        let item = MemoryItem {
            id: None,
            kind: "procedural".into(),
            content: format!("skill:{name}\n{steps_yaml}"),
            score: None,
            tier: MemoryTier::Warm,
            pinned: true, // procedures are always pinned
        };
        self.warm.store(&item)
    }

    /// Look up a procedure by exact skill name.
    pub fn find_procedure(&self, name: &str) -> Result<Option<MemoryHit>> {
        let q = Query {
            text: format!("skill:{name}"),
            embedding: None,
            limit: 1,
            kind_filter: Some("procedural".into()),
        };
        let hits = self.warm.search(&q)?;
        Ok(hits.into_iter().next())
    }

    /// List all stored procedures (for `>skills` command).
    pub fn list_procedures(&self) -> Result<Vec<MemoryHit>> {
        let q = Query {
            text: "skill:".into(),
            embedding: None,
            limit: 100,
            kind_filter: Some("procedural".into()),
        };
        self.warm.search(&q)
    }
}
