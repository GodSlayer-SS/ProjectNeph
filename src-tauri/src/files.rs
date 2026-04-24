use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

fn default_index_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(path) = dirs_next::document_dir() {
        roots.push(path);
    }
    if let Some(path) = dirs_next::desktop_dir() {
        roots.push(path);
    }
    if let Some(path) = dirs_next::download_dir() {
        roots.push(path);
    }
    roots
}

fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(items) => items,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

pub fn scan_default_files(conn: &Connection, max_files: usize) -> Result<usize> {
    let mut files = Vec::new();
    for root in default_index_roots() {
        walk_files(&root, &mut files);
        if files.len() >= max_files {
            break;
        }
    }
    files.truncate(max_files);

    for file in &files {
        let name = file
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("unknown");
        let extension = file.extension().and_then(|v| v.to_str()).unwrap_or("");
        let path = file.to_string_lossy();
        let metadata = fs::metadata(file).ok();
        let size = metadata.as_ref().map(|v| v.len() as i64);
        let modified_at = metadata
            .and_then(|v| v.modified().ok())
            .and_then(|v| v.elapsed().ok())
            .map(|v| format!("{}s_ago", v.as_secs()));

        conn.execute(
            "INSERT OR REPLACE INTO file_index (path, name, extension, size_bytes, modified_at, indexed_at) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
            params![path.as_ref(), name, extension, size, modified_at],
        )?;
    }
    Ok(files.len())
}

fn fts_match_query(query: &str) -> Option<String> {
    let parts: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() >= 2)
        .take(8)
        .map(|w| format!("\"{w}\"*"))
        .collect();
    if parts.is_empty() {
        return None;
    }
    Some(format!("name : ({})", parts.join(" OR ")))
}

fn fts_available(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='file_index_fts'",
        [],
        |_| Ok(1i32),
    )
    .optional()
    .ok()
    .flatten()
    .is_some()
}

pub fn search_files(conn: &Connection, query: &str, limit: usize) -> Result<Vec<String>> {
    if fts_available(conn) {
        if let Some(m) = fts_match_query(query) {
            let sql = "SELECT path FROM file_index_fts WHERE file_index_fts MATCH ?1 LIMIT ?2";
            if let Ok(mut stmt) = conn.prepare(sql) {
                let rows = stmt.query_map(params![m, limit as i64], |row| row.get::<_, String>(0));
                if let Ok(rows) = rows {
                    let v: Vec<String> = rows.filter_map(|r| r.ok()).collect();
                    if !v.is_empty() {
                        return Ok(v);
                    }
                }
            }
        }
    }

    let mut stmt = conn.prepare(
        "SELECT path FROM file_index WHERE name LIKE ?1 ORDER BY indexed_at DESC LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![format!("%{query}%"), limit as i64], |row| row.get(0))?;
    Ok(rows.filter_map(|row| row.ok()).collect())
}
