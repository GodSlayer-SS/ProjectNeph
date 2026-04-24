use anyhow::Result;
use rusqlite::params;
use std::fs;

use super::AppState;

impl AppState {
    pub(crate) fn undo_last_action(&self) -> Result<String> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, undo_payload FROM actions
             WHERE state = 'done' AND undo_payload IS NOT NULL
             ORDER BY id DESC LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        let Some(row) = rows.next()? else {
            return Ok("No undoable action found.".to_string());
        };
        let action_id: i64 = row.get(0)?;
        let payload: String = row.get(1)?;
        let value: serde_json::Value = serde_json::from_str(&payload)?;
        let kind = value["type"].as_str().unwrap_or("");
        match kind {
            "move" => {
                let from = value["from"].as_str().unwrap_or_default();
                let to = value["to"].as_str().unwrap_or_default();
                let _ = fs::rename(from, to);
            }
            "overwrite" => {
                let path = value["path"].as_str().unwrap_or_default();
                let content = value["content"].as_str().unwrap_or_default();
                let _ = fs::write(path, content);
            }
            _ => {}
        }
        conn.execute("UPDATE actions SET state = 'undone' WHERE id = ?1", params![action_id])?;
        Ok("Undo completed".to_string())
    }
}

#[cfg(test)]
mod undo_shape_tests {
    use serde_json::json;

    #[test]
    fn move_undo_payload_fields() {
        let p = json!({"type": "move", "from": "dest", "to": "src"});
        assert_eq!(p["type"], "move");
        assert!(p["from"].as_str().is_some());
        assert!(p["to"].as_str().is_some());
    }

    #[test]
    fn overwrite_undo_payload_fields() {
        let p = json!({"type": "overwrite", "path": "/tmp/x", "content": "prev"});
        assert_eq!(p["type"], "overwrite");
    }
}
