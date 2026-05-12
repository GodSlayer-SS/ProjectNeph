/// memory/hot.rs — Phase 2: in-session volatile memory tier.
///
/// Lives in a `Mutex<HashMap>` for the lifetime of the process.
/// Cleared on restart; never persisted to disk.
///
/// Used for:
///   - Injecting context within a multi-turn voice session
///   - Staging admission candidates before the post-session flush to `warm`
///   - Quick lookup for recently spoken/heard facts (last 64 items)

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use anyhow::Result;

use crate::traits::memory::{MemoryHit, MemoryId, MemoryItem, MemoryStore, MemoryTier, Query};

const MAX_HOT_ITEMS: usize = 64;

/// A single hot-tier record.
#[derive(Debug, Clone)]
struct HotRecord {
    id: i64,
    kind: String,
    content: String,
    pinned: bool,
}

/// Hot memory — per-process, wiped on restart.
#[derive(Clone)]
pub struct HotMemory {
    inner: Arc<Mutex<HotInner>>,
}

struct HotInner {
    records: VecDeque<HotRecord>,
    next_id: i64,
}

impl HotMemory {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HotInner {
                records: VecDeque::with_capacity(MAX_HOT_ITEMS),
                next_id: -1, // negative IDs never collide with SQLite positive rowids
            })),
        }
    }

    /// Snapshot the current hot memory for injection into the system prompt.
    /// Returns the most recent `limit` items formatted as a bulleted list.
    pub fn context_snippet(&self, limit: usize) -> String {
        let inner = self.inner.lock().unwrap();
        inner
            .records
            .iter()
            .rev()
            .take(limit)
            .map(|r| format!("• [{}] {}", r.kind, r.content))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Drain the hot tier — used by admission control post-session.
    pub fn drain(&self) -> Vec<MemoryItem> {
        let mut inner = self.inner.lock().unwrap();
        inner
            .records
            .drain(..)
            .map(|r| MemoryItem {
                id: None,
                kind: r.kind,
                content: r.content,
                pinned: r.pinned,
                tier: MemoryTier::Hot,
                score: None,
            })
            .collect()
    }
}

impl Default for HotMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore for HotMemory {
    fn tier(&self) -> MemoryTier {
        MemoryTier::Hot
    }

    fn search(&self, q: &Query) -> Result<Vec<MemoryHit>> {
        let inner = self.inner.lock().unwrap();
        let query_lc = q.text.to_lowercase();
        let mut hits: Vec<MemoryHit> = inner
            .records
            .iter()
            .filter(|r| {
                if let Some(kf) = &q.kind_filter {
                    &r.kind != kf
                } else {
                    true
                }
            })
            .filter(|r| r.content.to_lowercase().contains(&query_lc))
            .map(|r| MemoryHit {
                id: r.id,
                content: r.content.clone(),
                kind: r.kind.clone(),
                score: 0.9, // hot tier always ranks above warm
                tier: MemoryTier::Hot,
            })
            .collect();
        hits.truncate(q.limit);
        Ok(hits)
    }

    fn store(&self, item: &MemoryItem) -> Result<MemoryId> {
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_id;
        inner.next_id -= 1;
        if inner.records.len() >= MAX_HOT_ITEMS {
            inner.records.pop_front(); // evict oldest
        }
        inner.records.push_back(HotRecord {
            id,
            kind: item.kind.clone(),
            content: item.content.clone(),
            pinned: item.pinned,
        });
        Ok(id)
    }

    fn forget(&self, id: MemoryId) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.records.retain(|r| r.id != id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hot_store_and_search() {
        let mem = HotMemory::new();
        let item = MemoryItem {
            id: None,
            kind: "fact".into(),
            content: "prefers dark mode".into(),
            pinned: false,
            tier: MemoryTier::Hot,
            score: None,
        };
        let id = mem.store(&item).unwrap();
        assert!(id < 0);

        let hits = mem.search(&Query::text("dark mode")).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].content, "prefers dark mode");
    }

    #[test]
    fn hot_evicts_at_max() {
        let mem = HotMemory::new();
        for i in 0..=MAX_HOT_ITEMS {
            mem.store(&MemoryItem {
                id: None,
                kind: "fact".into(),
                content: format!("item {i}"),
                pinned: false,
                tier: MemoryTier::Hot,
                score: None,
            })
            .unwrap();
        }
        let inner = mem.inner.lock().unwrap();
        assert_eq!(inner.records.len(), MAX_HOT_ITEMS);
        // Oldest evicted; newest preserved.
        assert!(inner.records.back().unwrap().content.contains("64"));
    }
}
