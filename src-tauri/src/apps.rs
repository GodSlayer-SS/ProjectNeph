use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use rusqlite::{params, Connection};

fn start_menu_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(program_data) = std::env::var("ProgramData") {
        roots.push(
            Path::new(&program_data)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs"),
        );
    }
    if let Ok(app_data) = std::env::var("APPDATA") {
        roots.push(
            Path::new(&app_data)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs"),
        );
    }
    roots
}

fn walk_links(dir: &Path, out: &mut Vec<PathBuf>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_links(&path, out);
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("lnk"))
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
}

pub fn scan_start_menu_apps(conn: &Connection) -> Result<usize> {
    let mut links = Vec::new();
    for root in start_menu_roots() {
        walk_links(&root, &mut links);
    }

    for link in &links {
        let name = link
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("Unknown App");
        let exec_path = link.to_string_lossy();
        conn.execute(
            "INSERT OR IGNORE INTO app_index (name, exec_path) VALUES (?1, ?2)",
            params![name, exec_path.as_ref()],
        )?;
    }
    Ok(links.len())
}

pub fn launch_indexed_app(conn: &Connection, query: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, exec_path
         FROM app_index
         WHERE lower(name) LIKE lower(?1)
         ORDER BY
           (
             CASE
               WHEN lower(name) = lower(?2) THEN 1000
               WHEN lower(name) LIKE lower(?3) THEN 500
               ELSE 100
             END
             + min(launch_count * 12, 360)
             + CASE
                 WHEN last_used IS NULL THEN 0
                 ELSE max(0, 120 - CAST((julianday('now') - julianday(last_used)) * 24 AS INTEGER))
               END
           ) DESC,
           name ASC
         LIMIT 1",
    )?;
    let contains_match = format!("%{query}%");
    let exact_match = query.to_string();
    let prefix_match = format!("{query}%");
    let mut rows = stmt.query(params![contains_match, exact_match, prefix_match])?;
    if let Some(row) = rows.next()? {
        let app_id: i64 = row.get(0)?;
        let app_name: String = row.get(1)?;
        let exec_path: String = row.get(2)?;

        let _ = Command::new("cmd")
            .args(["/C", "start", "", &exec_path])
            .spawn();

        conn.execute(
            "UPDATE app_index SET launch_count = launch_count + 1, last_used = datetime('now') WHERE id = ?1",
            params![app_id],
        )?;
        return Ok(Some(app_name));
    }
    Ok(None)
}
