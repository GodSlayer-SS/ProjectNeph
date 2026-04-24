# Embedding and vector search (v0.1)

Neph v0.1 ships a **truthful, lightweight** retrieval stack:

- **Stub embedding (`neph-stub-hash-v0.1`)**: a deterministic 384-dimensional vector derived from UTF-8 bytes (not a sentence-transformer). It is useful for rough similarity only.
- **sqlite-vec (optional)**: when the `sqlite_vec` native extension loads at startup, vec0 virtual tables are created and `settings.sqlite_vec_loaded` is set to `1`. If loading fails (common in locked-down or portable environments), the app keeps running; **semantic contribution in recall is disabled** and the UI shows **Vector search: disabled**.
- **Keyword recall**: `>recall` always considers substring overlap on stored memory text.
- **Half-life**: recall scores are multiplied by an exponential decay from `updated_at` (90-day half-life) so older rows do not dominate.

A future release may add **real MiniLM / fastembed** with versioned re-embed and explicit migration; that is **not** claimed in v0.1.
