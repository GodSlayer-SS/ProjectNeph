use anyhow::Result;
use rusqlite::{params, Connection};

use crate::models::AdmissionItem;

use super::AppState;

fn conn(state: &AppState) -> Result<Connection> {
    state.connect()
}

impl AppState {
    pub fn list_admission_queue(&self) -> Result<Vec<AdmissionItem>> {
        let conn = conn(self)?;
        let mut stmt = conn.prepare(
            "SELECT id, content, kind, score, decision, created_at
             FROM admission_queue
             WHERE decision = 'pending'
             ORDER BY id DESC
             LIMIT 50",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AdmissionItem {
                id: row.get(0)?,
                content: row.get(1)?,
                kind: row.get(2).ok(),
                score: row.get(3).ok(),
                decision: row.get::<_, String>(4)?,
                created_at: row.get(5)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn enqueue_admission_candidate(&self, content: &str, kind: Option<&str>, score: Option<f32>) -> Result<()> {
        let conn = conn(self)?;
        conn.execute(
            "INSERT INTO admission_queue(content, kind, score) VALUES(?1, ?2, ?3)",
            params![content, kind, score],
        )?;
        Ok(())
    }

    pub fn decide_admission(&self, id: i64, decision: &str) -> Result<bool> {
        let conn = conn(self)?;
        let changed = conn.execute(
            "UPDATE admission_queue
             SET decision = ?2, decided_at = datetime('now')
             WHERE id = ?1 AND decision = 'pending'",
            params![id, decision],
        )?;
        Ok(changed > 0)
    }

    pub fn keep_admission_as_memory(&self, id: i64) -> Result<bool> {
        let conn = conn(self)?;
        let row = conn.query_row(
            "SELECT content, kind FROM admission_queue WHERE id = ?1 AND decision = 'pending'",
            params![id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
        );
        let Ok((content, kind)) = row else {
            return Ok(false);
        };

        // Persist to warm memory (existing embedding pipeline runs).
        let kind = kind.unwrap_or_else(|| "episode".into());
        self.save_memory(&kind, &content)?;

        // Mark decision.
        let _ = self.decide_admission(id, "keep")?;
        Ok(true)
    }

    pub fn discard_admission(&self, id: i64) -> Result<bool> {
        self.decide_admission(id, "discard")
    }

    pub fn log_voice_session(&self, transcript: Option<&str>, duration_ms: u128) -> Result<()> {
        let conn = conn(self)?;
        conn.execute(
            "INSERT INTO voice_sessions(transcript, intent, duration_ms) VALUES(?1, ?2, ?3)",
            params![transcript, "voice", duration_ms as i64],
        )?;
        Ok(())
    }
}

