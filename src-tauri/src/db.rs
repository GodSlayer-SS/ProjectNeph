use std::path::{Path, PathBuf};

use anyhow::Result;
use rusqlite::{params, Connection};

mod embedded {
    refinery::embed_migrations!("src/db/migrations");
}

/// Outcome of opening the DB and probing optional native extensions.
#[derive(Debug, Clone)]
pub struct DbInitMeta {
    pub sqlite_vec_loaded: bool,
}

pub fn database_path(app_dir: &Path) -> PathBuf {
    app_dir.join("neph.db")
}

fn put_setting_conn(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings(key, value, updated_at) VALUES(?1, ?2, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
        params![key, value],
    )?;
    Ok(())
}

/// Mark in-flight tool executions as failed after an unclean shutdown.
pub fn recover_interrupted_executing(conn: &Connection) -> Result<u64> {
    let n = conn.execute(
        "UPDATE actions SET state = 'failed', finished_at = datetime('now'),
            error_message = 'Interrupted at app restart (was executing)'
         WHERE state = 'executing'",
        [],
    )?;
    Ok(n as u64)
}

fn backfill_file_index_fts(conn: &Connection) -> Result<()> {
    let fts: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='file_index_fts'",
            [],
            |_| Ok(1i64),
        )
        .ok();
    if fts.is_none() {
        return Ok(());
    }
    let count: i64 = conn.query_row("SELECT COUNT(1) FROM file_index_fts", [], |r| r.get(0))?;
    if count == 0 {
        conn.execute(
            "INSERT INTO file_index_fts(rowid, path, name, extension)
             SELECT rowid, path, name, extension FROM file_index",
            [],
        )?;
    }
    Ok(())
}

pub fn initialize_database(app_dir: &Path) -> Result<(PathBuf, DbInitMeta)> {
    std::fs::create_dir_all(app_dir)?;
    let path = database_path(app_dir);
    let mut conn = Connection::open(&path)?;
    // PRAGMA journal_mode=WAL cannot run inside refinery's per-migration transaction;
    // apply connection pragmas after migrations. `set_abort_divergent(false)` tolerates
    // a one-time checksum change on V1 for installs that already applied the old file.
    embedded::migrations::runner()
        .set_abort_divergent(false)
        .run(&mut conn)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA synchronous = NORMAL;",
    )?;
    recover_interrupted_executing(&conn)?;

    let mut sqlite_vec_loaded = false;
    unsafe {
        if conn.load_extension_enable().is_ok() {
            if conn.load_extension("sqlite_vec", None::<&str>).is_ok() {
                let mem = conn.execute(
                    "CREATE VIRTUAL TABLE IF NOT EXISTS memory_vec_vtab USING vec0(id INTEGER PRIMARY KEY, embedding FLOAT[384])",
                    [],
                );
                let notes = conn.execute(
                    "CREATE VIRTUAL TABLE IF NOT EXISTS notes_vec_vtab USING vec0(id INTEGER PRIMARY KEY, embedding FLOAT[384])",
                    [],
                );
                sqlite_vec_loaded = mem.is_ok() && notes.is_ok();
            }
            let _ = conn.load_extension_disable();
        }
    }

    put_setting_conn(
        &conn,
        "sqlite_vec_loaded",
        if sqlite_vec_loaded { "1" } else { "0" },
    )?;
    put_setting_conn(&conn, "embedding_mode", crate::embeddings::mode_name())?;
    put_setting_conn(&conn, "phase2_enabled", "0")?;
    put_setting_conn(&conn, "phase3_enabled", "0")?;
    put_setting_conn(&conn, "phase4_enabled", "0")?;

    let _ = backfill_file_index_fts(&conn);

    Ok((
        path,
        DbInitMeta {
            sqlite_vec_loaded,
        },
    ))
}

/// Read a row from `settings`, or `None` if missing.
pub fn read_setting(db_path: &Path, key: &str) -> Result<Option<String>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let row = stmt.query_row(params![key], |row| row.get::<_, String>(0));
    match row {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Upsert a `settings` row.
pub fn write_setting(db_path: &Path, key: &str, value: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    put_setting_conn(&conn, key, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn recover_marks_executing_failed() {
        let dir = tempdir().unwrap();
        let (path, _) = initialize_database(dir.path()).unwrap();
        let conn = Connection::open(&path).unwrap();
        conn.execute(
            "INSERT INTO command_history (input, intent, tool_name, tool_args, success) VALUES ('x','y','z','{}',NULL)",
            [],
        )
        .unwrap();
        let cid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO actions (command_id, tool_name, args_json, risk_level, state) VALUES (?1,'z','{}','green','executing')",
            params![cid],
        )
        .unwrap();
        drop(conn);
        let conn = Connection::open(&path).unwrap();
        let n = recover_interrupted_executing(&conn).unwrap();
        assert_eq!(n, 1u64);
        let state: String = conn
            .query_row("SELECT state FROM actions ORDER BY id DESC LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(state, "failed");
    }
}
