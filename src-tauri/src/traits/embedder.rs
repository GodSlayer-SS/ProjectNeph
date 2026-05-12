use anyhow::Result;

/// Anything that produces dense vector embeddings.
/// Phase 1: `FasembedEmbedder` (in-process, fallback)
/// Phase 2: `SidecarEmbedder` (Python sidecar, bge-small via sentence-transformers)
pub trait Embedder: Send + Sync {
    fn name(&self) -> &str;
    /// Dimension of produced vectors.
    fn dim(&self) -> usize;
    /// Embed a batch of texts. Returns one vector per input text.
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}
