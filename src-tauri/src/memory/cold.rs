use std::sync::Arc;

use anyhow::Result;

use crate::embeddings;
use crate::traits::memory::{MemoryHit, MemoryId, MemoryItem, MemoryStore, MemoryTier, Query};

/// Cold memory tier backed by LanceDB (Phase 2).
///
/// Storage lives under `%LOCALAPPDATA%/Neph/lancedb` (same root as DB/logs).
pub struct ColdMemory {
    rt: tokio::runtime::Runtime,
    db_path: String,
}

impl ColdMemory {
    pub fn new() -> Self {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        let root = dirs_next::data_local_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("Neph")
            .join("lancedb");
        let _ = std::fs::create_dir_all(&root);
        Self {
            rt,
            db_path: root.to_string_lossy().to_string(),
        }
    }

    fn ensure_table(&self) -> Result<lancedb::table::Table> {
        self.rt.block_on(async {
            use arrow_array::{Float32Array, Int64Array, RecordBatch, StringArray};
            use arrow_schema::{DataType, Field, Schema};

            let db = lancedb::connect(&self.db_path).execute().await?;

            if let Ok(tbl) = db.open_table("memories").execute().await {
                return Ok(tbl);
            }

            let ndims = 384i32;
            let schema = Arc::new(Schema::new(vec![
                Field::new("id", DataType::Int64, false),
                Field::new("kind", DataType::Utf8, false),
                Field::new("content", DataType::Utf8, false),
                Field::new(
                    "embedding",
                    DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), ndims),
                    true,
                ),
            ]));

            // Create an empty table with the desired schema.
            let empty = RecordBatch::try_new(
                schema,
                vec![
                    Arc::new(Int64Array::from_iter_values(std::iter::empty::<i64>())),
                    Arc::new(StringArray::from_iter_values(std::iter::empty::<&str>())),
                    Arc::new(StringArray::from_iter_values(std::iter::empty::<&str>())),
                    Arc::new(arrow_array::FixedSizeListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(
                        std::iter::empty::<Option<Vec<Option<f32>>>>(),
                        ndims,
                    )),
                ],
            )?;

            let tbl = db.create_table("memories", empty).execute().await?;
            let _ = tbl
                .create_index(&["embedding"], lancedb::index::Index::Auto)
                .execute()
                .await;
            Ok(tbl)
        })
    }

    fn embed_or_zero(text: &str) -> Vec<f32> {
        embeddings::embed_text(text).unwrap_or_else(|_| vec![0.0; 384])
    }

    fn fixed_list_from_vec(vec: &[f32]) -> arrow_array::FixedSizeListArray {
        use arrow_array::types::Float32Type;
        arrow_array::FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
            std::iter::once(Some(vec.iter().map(|v| Some(*v)).collect::<Vec<_>>())),
            vec.len() as i32,
        )
    }
}

impl Default for ColdMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore for ColdMemory {
    fn tier(&self) -> MemoryTier {
        MemoryTier::Cold
    }

    fn search(&self, q: &Query) -> Result<Vec<MemoryHit>> {
        let query_vec = q.embedding.clone().unwrap_or_else(|| Self::embed_or_zero(&q.text));
        let tbl = self.ensure_table()?;

        self.rt.block_on(async {
            let mut out = Vec::new();
            let results = tbl
                .query()
                .nearest_to(&query_vec)?
                .limit(q.limit as u32)
                .execute()
                .await?;

            let batches = results.try_collect::<Vec<_>>().await?;
            for batch in batches {
                let id_col = batch.column_by_name("id").and_then(|c| c.as_any().downcast_ref::<arrow_array::Int64Array>());
                let kind_col = batch.column_by_name("kind").and_then(|c| c.as_any().downcast_ref::<arrow_array::StringArray>());
                let content_col = batch
                    .column_by_name("content")
                    .and_then(|c| c.as_any().downcast_ref::<arrow_array::StringArray>());
                let dist_col = batch
                    .column_by_name("_distance")
                    .and_then(|c| c.as_any().downcast_ref::<arrow_array::Float32Array>());

                let rows = batch.num_rows();
                for i in 0..rows {
                    let id = id_col.and_then(|c| if c.is_null(i) { None } else { Some(c.value(i)) }).unwrap_or(0);
                    let kind = kind_col
                        .and_then(|c| if c.is_null(i) { None } else { Some(c.value(i).to_string()) })
                        .unwrap_or_else(|| "memory".into());
                    let content = content_col
                        .and_then(|c| if c.is_null(i) { None } else { Some(c.value(i).to_string()) })
                        .unwrap_or_default();
                    let dist = dist_col.and_then(|c| if c.is_null(i) { None } else { Some(c.value(i)) }).unwrap_or(1.0);
                    let score = 1.0 / (1.0 + dist.max(0.0));
                    if !content.is_empty() {
                        out.push(MemoryHit {
                            id,
                            content,
                            kind,
                            score,
                            tier: MemoryTier::Cold,
                        });
                    }
                }
            }
            Ok(out)
        })
    }

    fn store(&self, item: &MemoryItem) -> Result<MemoryId> {
        let tbl = self.ensure_table()?;
        let embedding = Self::embed_or_zero(&item.content);

        self.rt.block_on(async {
            use arrow_array::{Int64Array, RecordBatch, StringArray};
            use arrow_schema::Schema;

            // We don't have a stable cross-tier id yet; use a timestamp-ish id.
            let id = chrono::Utc::now().timestamp_millis();
            let schema: Arc<Schema> = tbl.schema().await?;

            let batch = RecordBatch::try_new(
                schema,
                vec![
                    Arc::new(Int64Array::from(vec![id])),
                    Arc::new(StringArray::from(vec![item.kind.clone()])),
                    Arc::new(StringArray::from(vec![item.content.clone()])),
                    Arc::new(Self::fixed_list_from_vec(&embedding)),
                ],
            )?;
            tbl.add(batch).execute().await?;
            Ok(id)
        })
    }

    fn forget(&self, id: MemoryId) -> Result<()> {
        let tbl = self.ensure_table()?;
        self.rt.block_on(async {
            tbl.delete(&format!("id = {id}")).await?;
            Ok(())
        })
    }
}

