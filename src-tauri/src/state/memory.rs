use std::sync::atomic::Ordering;

use anyhow::Result;
use rusqlite::{params, Connection};

use crate::embeddings;
use crate::memory::cold::ColdMemory;
use crate::models::MemoryItem;
use crate::traits::memory::{MemoryItem as TierMemoryItem, MemoryStore, MemoryTier, Query};

use super::AppState;

/// Memory relevance decays with age so stale rows do not dominate recall.
const MEMORY_HALF_LIFE_DAYS: f32 = 90.0;

fn updated_at_decay_factor(updated_at: &str) -> f32 {
    let ts = if let Ok(t) = chrono::NaiveDateTime::parse_from_str(updated_at, "%Y-%m-%d %H:%M:%S") {
        t
    } else if let Ok(d) = chrono::NaiveDate::parse_from_str(updated_at, "%Y-%m-%d") {
        match d.and_hms_opt(0, 0, 0) {
            Some(t) => t,
            None => return 1.0,
        }
    } else {
        return 1.0;
    };
    let now = chrono::Local::now().naive_local();
    let age_days = (now - ts).num_seconds().max(0) as f32 / 86400.0;
    (-age_days / MEMORY_HALF_LIFE_DAYS).exp()
}

impl AppState {
    pub fn reembed_existing_memories(&self) -> Result<u64> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare("SELECT id, content FROM memory WHERE deleted_at IS NULL ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))?;
        let mut updated = 0u64;
        for row in rows.flatten() {
            let (id, content) = row;
            self.upsert_memory_embedding(&conn, id, &content)?;
            updated += 1;
        }
        Ok(updated)
    }

    pub fn save_memory(&self, kind: &str, content: &str) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO memory (kind, content) VALUES (?1, ?2)",
            params![kind, content],
        )?;
        let id = conn.last_insert_rowid();
        self.upsert_memory_embedding(&conn, id, content)?;
        // Phase 2 cold tier (LanceDB). Best effort so warm path remains reliable.
        let cold = ColdMemory::new();
        let _ = cold.store(&TierMemoryItem {
            id: None,
            kind: kind.to_string(),
            content: content.to_string(),
            pinned: false,
            tier: MemoryTier::Cold,
            score: None,
        });
        Ok(())
    }

    pub fn recall_memory(&self, query: &str) -> Result<Vec<String>> {
        let conn = self.connect()?;
        let query_embedding = self.embed_text(query);
        let vec_ok = self.sqlite_vec_loaded.load(Ordering::Relaxed);
        let mut stmt = conn.prepare(
            "SELECT m.content, m.updated_at, v.embedding
             FROM memory m
             LEFT JOIN memory_vec v ON v.id = m.id
             WHERE m.deleted_at IS NULL
             ORDER BY m.updated_at DESC
             LIMIT 200",
        )?;
        let rows = stmt.query_map([], |row| {
            let content: String = row.get(0)?;
            let updated_at: String = row.get(1)?;
            let embedding_blob: Option<Vec<u8>> = row.get(2)?;
            Ok((content, updated_at, embedding_blob))
        })?;
        let mut scored: Vec<(f32, String)> = Vec::new();
        for row in rows.flatten() {
            let (content, updated_at, embedding_blob) = row;
            let lexical = if content.to_lowercase().contains(&query.to_lowercase()) {
                0.7
            } else {
                0.0
            };
            let stub_semantic = if vec_ok {
                embedding_blob
                    .as_deref()
                    .map(Self::blob_to_embedding)
                    .map(|v| self.cosine_similarity(&query_embedding, &v))
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            let decay = updated_at_decay_factor(&updated_at);
            let score = (lexical + (stub_semantic * 0.3)) * decay;
            if score > 0.05 {
                scored.push((score, content));
            }
        }
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut merged = scored
            .into_iter()
            .take(5)
            .map(|(_, c)| c)
            .collect::<Vec<_>>();

        // Phase 2 cold tier retrieval merge.
        let cold = ColdMemory::new();
        if let Ok(cold_hits) = cold.search(&Query {
            text: query.to_string(),
            embedding: Some(query_embedding),
            limit: 5,
            kind_filter: None,
        }) {
            for hit in cold_hits {
                if !merged.iter().any(|m| m == &hit.content) {
                    merged.push(hit.content);
                }
                if merged.len() >= 5 {
                    break;
                }
            }
        }
        Ok(merged)
    }

    pub fn list_memory(&self, query: Option<&str>) -> Result<Vec<MemoryItem>> {
        let conn = self.connect()?;
        if let Some(search) = query {
            let mut stmt = conn.prepare(
                "SELECT id, kind, content, pinned, created_at
                 FROM memory
                 WHERE deleted_at IS NULL AND content LIKE ?1
                 ORDER BY pinned DESC, updated_at DESC
                 LIMIT 100",
            )?;
            let rows = stmt.query_map(params![format!("%{search}%")], |row| {
                Ok(MemoryItem {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    content: row.get(2)?,
                    pinned: row.get::<_, i64>(3)? == 1,
                    created_at: row.get(4)?,
                })
            })?;
            return Ok(rows.filter_map(|item| item.ok()).collect());
        }

        let mut stmt = conn.prepare(
            "SELECT id, kind, content, pinned, created_at
             FROM memory
             WHERE deleted_at IS NULL
             ORDER BY pinned DESC, updated_at DESC
             LIMIT 100",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(MemoryItem {
                id: row.get(0)?,
                kind: row.get(1)?,
                content: row.get(2)?,
                pinned: row.get::<_, i64>(3)? == 1,
                created_at: row.get(4)?,
            })
        })?;
        Ok(rows.filter_map(|item| item.ok()).collect())
    }

    pub fn update_memory(&self, id: i64, content: &str) -> Result<bool> {
        let conn = self.connect()?;
        let changed = conn.execute(
            "UPDATE memory SET content = ?1, updated_at = datetime('now') WHERE id = ?2 AND deleted_at IS NULL AND editable = 1",
            params![content, id],
        )?;
        if changed > 0 {
            self.upsert_memory_embedding(&conn, id, content)?;
            let cold = ColdMemory::new();
            let _ = cold.store(&TierMemoryItem {
                id: None,
                kind: "memory".to_string(),
                content: content.to_string(),
                pinned: false,
                tier: MemoryTier::Cold,
                score: None,
            });
        }
        Ok(changed > 0)
    }

    pub fn set_memory_pin(&self, id: i64, pinned: bool) -> Result<bool> {
        let conn = self.connect()?;
        let changed = conn.execute(
            "UPDATE memory SET pinned = ?1, updated_at = datetime('now') WHERE id = ?2 AND deleted_at IS NULL",
            params![if pinned { 1 } else { 0 }, id],
        )?;
        Ok(changed > 0)
    }

    pub fn delete_memory(&self, id: i64) -> Result<bool> {
        let conn = self.connect()?;
        let changed = conn.execute(
            "UPDATE memory SET deleted_at = datetime('now'), updated_at = datetime('now') WHERE id = ?1 AND deleted_at IS NULL",
            params![id],
        )?;
        Ok(changed > 0)
    }

    fn upsert_memory_embedding(&self, conn: &Connection, id: i64, content: &str) -> Result<()> {
        let hash = self.content_hash(content);
        let existing: Option<String> = conn
            .query_row(
                "SELECT hash FROM embedding_meta WHERE source_table = 'memory' AND source_id = ?1",
                params![id],
                |row| row.get(0),
            )
            .ok();
        if existing.as_deref() == Some(hash.as_str()) {
            return Ok(());
        }
        conn.execute(
            "INSERT INTO embedding_meta (source_table, source_id, model, dim, hash, embedded_at)
             VALUES ('memory', ?1, ?3, 384, ?2, datetime('now'))
             ON CONFLICT(source_table, source_id) DO UPDATE SET model = excluded.model, hash = excluded.hash, embedded_at = datetime('now')",
            params![id, hash, embeddings::mode_name()],
        )?;
        let embedding = self.embed_text(content);
        let blob = Self::embedding_to_blob(&embedding);
        conn.execute(
            "INSERT INTO memory_vec (id, embedding) VALUES (?1, ?2)
             ON CONFLICT(id) DO UPDATE SET embedding = excluded.embedding",
            params![id, blob],
        )?;
        Ok(())
    }

    fn embed_text(&self, text: &str) -> Vec<f32> {
        embeddings::embed_text(text).unwrap_or_else(|_| vec![0.0; 384])
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>()
    }

    fn content_hash(&self, text: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn embedding_to_blob(values: &[f32]) -> Vec<u8> {
        let mut out = Vec::with_capacity(values.len() * 4);
        for value in values {
            out.extend_from_slice(&value.to_le_bytes());
        }
        out
    }

    fn blob_to_embedding(blob: &[u8]) -> Vec<f32> {
        blob.chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
    }
}
