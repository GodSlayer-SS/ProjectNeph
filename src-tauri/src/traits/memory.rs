use anyhow::Result;
use serde::{Deserialize, Serialize};

// ── Query / Result types ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Query {
    pub text: String,
    /// Optional pre-computed embedding (skips re-embedding if provided).
    pub embedding: Option<Vec<f32>>,
    /// Max number of results requested.
    pub limit: usize,
    /// Filter by memory kind: "fact" | "preference" | "episode" | "procedural"
    pub kind_filter: Option<String>,
}

impl Query {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            embedding: None,
            limit: 5,
            kind_filter: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHit {
    pub id: i64,
    pub content: String,
    pub kind: String,
    pub score: f32,
    pub tier: MemoryTier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryTier {
    Hot,  // in-process session HashMap
    Warm, // SQLite
    Cold, // LanceDB (Phase 2)
}

// ── Item to store ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    /// Database row-id if this item was loaded from warm/cold storage.
    /// `None` for items constructed in-process (hot tier, admission candidates).
    pub id: Option<i64>,
    pub kind: String,
    pub content: String,
    pub pinned: bool,
    pub tier: MemoryTier,
    /// Pre-computed relevance score — filled by search, `None` for new items.
    pub score: Option<f32>,
}

impl MemoryItem {
    /// Construct a new in-process item (no DB id yet, no score).
    pub fn new(kind: impl Into<String>, content: impl Into<String>, tier: MemoryTier) -> Self {
        Self {
            id: None,
            kind: kind.into(),
            content: content.into(),
            pinned: false,
            tier,
            score: None,
        }
    }
}

pub type MemoryId = i64;

// ── The Trait ─────────────────────────────────────────────────────────────────

/// A memory tier. All three tiers implement this.
/// Hot (`memory/hot.rs`), Warm (`memory/warm.rs`), Cold (`memory/cold.rs`).
pub trait MemoryStore: Send + Sync {
    fn tier(&self) -> MemoryTier;

    /// Retrieve top-k memories matching a query.
    fn search(&self, q: &Query) -> Result<Vec<MemoryHit>>;

    /// Persist a new memory item. Returns its ID.
    fn store(&self, item: &MemoryItem) -> Result<MemoryId>;

    /// Soft-delete a memory item.
    fn forget(&self, id: MemoryId) -> Result<()>;
}
