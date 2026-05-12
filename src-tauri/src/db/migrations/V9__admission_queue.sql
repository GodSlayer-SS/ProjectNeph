-- V9: Memory admission queue
-- Post-session distill pass classifies each episode before promoting to warm/cold.
CREATE TABLE IF NOT EXISTS admission_queue (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    content    TEXT    NOT NULL,
    kind       TEXT,
    score      REAL,
    decision   TEXT    CHECK(decision IN ('keep','discard','pending')) DEFAULT 'pending',
    decided_at TEXT,
    created_at TEXT    NOT NULL DEFAULT (datetime('now'))
);
