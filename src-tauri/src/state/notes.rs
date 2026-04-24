use anyhow::Result;
use rusqlite::params;

use super::AppState;

impl AppState {
    pub fn create_note(&self, body: &str) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO notes (title, body, updated_at) VALUES (?1, ?2, datetime('now'))",
            params!["Quick note", body],
        )?;
        Ok(())
    }

    pub fn list_notes(&self) -> Result<Vec<String>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT id, title FROM notes WHERE deleted_at IS NULL ORDER BY updated_at DESC LIMIT 10",
        )?;
        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            Ok(format!("#{id} {title}"))
        })?;
        Ok(rows.filter_map(|row| row.ok()).collect())
    }

    pub fn search_notes(&self, query: &str) -> Result<Vec<String>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT n.id, n.title
             FROM notes_fts f
             JOIN notes n ON n.id = f.rowid
             WHERE notes_fts MATCH ?1 AND n.deleted_at IS NULL
             ORDER BY bm25(notes_fts), n.updated_at DESC
             LIMIT 10",
        )?;
        let rows = stmt.query_map(params![query], |row| {
            let id: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            Ok(format!("#{id} {title}"))
        })?;
        Ok(rows.filter_map(|row| row.ok()).collect())
    }

    pub fn update_note(&self, id: i64, body: &str) -> Result<bool> {
        let conn = self.connect()?;
        let changed = conn.execute(
            "UPDATE notes SET body = ?1, updated_at = datetime('now') WHERE id = ?2 AND deleted_at IS NULL",
            params![body, id],
        )?;
        Ok(changed > 0)
    }

    pub fn delete_note(&self, id: i64) -> Result<bool> {
        let conn = self.connect()?;
        let changed = conn.execute(
            "UPDATE notes SET deleted_at = datetime('now'), updated_at = datetime('now') WHERE id = ?1 AND deleted_at IS NULL",
            params![id],
        )?;
        Ok(changed > 0)
    }
}
