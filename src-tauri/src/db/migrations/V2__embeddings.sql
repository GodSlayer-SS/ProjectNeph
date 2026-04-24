CREATE TABLE IF NOT EXISTS embedding_meta (
  source_table TEXT NOT NULL,
  source_id INTEGER NOT NULL,
  model TEXT NOT NULL,
  dim INTEGER NOT NULL,
  hash TEXT NOT NULL,
  embedded_at TEXT NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (source_table, source_id)
);

-- Fallback tables for environments where sqlite-vec extension is unavailable.
CREATE TABLE IF NOT EXISTS memory_vec (
  id INTEGER PRIMARY KEY,
  embedding BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS notes_vec (
  id INTEGER PRIMARY KEY,
  embedding BLOB NOT NULL
);
