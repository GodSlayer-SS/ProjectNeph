CREATE TABLE IF NOT EXISTS scheduled_jobs (
  id INTEGER PRIMARY KEY,
  message TEXT NOT NULL,
  due_at TEXT NOT NULL,
  delivered_at TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_scheduled_jobs_due ON scheduled_jobs(due_at, delivered_at);
