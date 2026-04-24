use std::sync::{Mutex, OnceLock};

use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

static EMBEDDER: OnceLock<Mutex<Option<TextEmbedding>>> = OnceLock::new();

fn embedder() -> &'static Mutex<Option<TextEmbedding>> {
    EMBEDDER.get_or_init(|| {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::BGESmallENV15).with_show_download_progress(false),
        )
        .ok();
        Mutex::new(model)
    })
}

pub fn embed_text(text: &str) -> Result<Vec<f32>> {
    let lock = embedder()
        .lock()
        .map_err(|_| anyhow::anyhow!("embedding lock poisoned"))?;
    if let Some(model) = lock.as_ref() {
        let vectors = model.embed(vec![text.to_string()], None)?;
        if let Some(vector) = vectors.into_iter().next() {
            return Ok(vector);
        }
    }
    anyhow::bail!("fastembed model unavailable")
}

pub fn mode_name() -> &'static str {
    if let Ok(lock) = embedder().lock() {
        if lock.is_some() {
            return "fastembed_bge_small_v1";
        }
    }
    "fastembed_bge_small_v1_unavailable"
}
