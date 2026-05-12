-- V8: Voice session audit table
-- Records each voice interaction for latency tracking and debugging.
CREATE TABLE IF NOT EXISTS voice_sessions (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    transcript   TEXT,
    intent       TEXT,
    duration_ms  INTEGER,
    stt_provider TEXT    NOT NULL DEFAULT 'groq_whisper',
    tts_provider TEXT    NOT NULL DEFAULT 'edge_tts',
    created_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);
