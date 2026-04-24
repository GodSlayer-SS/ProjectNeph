//! Local diagnostic bundle (no telemetry): redacted counts + optional log tail paths.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::Serialize;
use zip::write::FileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

#[derive(Debug, Serialize)]
pub struct DiagnosticReport {
    pub generated_at_utc: String,
    pub sqlite_user_version: i64,
    pub settings_keys: Vec<String>,
    pub sqlite_vec_loaded: Option<String>,
    pub embedding_mode: Option<String>,
    pub dpapi_protect_exports: Option<String>,
    pub counts: DiagnosticCounts,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticCounts {
    pub memory_rows: i64,
    pub notes_rows: i64,
    pub command_history_rows: i64,
    pub file_index_rows: i64,
    pub actions_executing: i64,
}

fn setting_or_empty(conn: &Connection, key: &str) -> anyhow::Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let row = stmt.query_row(params![key], |r| r.get::<_, String>(0));
    match row {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn collect_report(db_path: &Path) -> Result<DiagnosticReport> {
    let conn = Connection::open(db_path)?;
    let sqlite_user_version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let mut stmt = conn.prepare("SELECT key FROM settings ORDER BY key")?;
    let settings_keys: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .filter_map(|v| v.ok())
        .collect();
    let counts = DiagnosticCounts {
        memory_rows: conn.query_row("SELECT COUNT(1) FROM memory WHERE deleted_at IS NULL", [], |r| r.get(0))?,
        notes_rows: conn.query_row("SELECT COUNT(1) FROM notes WHERE deleted_at IS NULL", [], |r| r.get(0))?,
        command_history_rows: conn.query_row("SELECT COUNT(1) FROM command_history", [], |r| r.get(0))?,
        file_index_rows: conn.query_row("SELECT COUNT(1) FROM file_index", [], |r| r.get(0))?,
        actions_executing: conn.query_row(
            "SELECT COUNT(1) FROM actions WHERE state = 'executing'",
            [],
            |r| r.get(0),
        )?,
    };
    Ok(DiagnosticReport {
        generated_at_utc: chrono::Utc::now().to_rfc3339(),
        sqlite_user_version,
        settings_keys,
        sqlite_vec_loaded: setting_or_empty(&conn, "sqlite_vec_loaded")?,
        embedding_mode: setting_or_empty(&conn, "embedding_mode")?,
        dpapi_protect_exports: setting_or_empty(&conn, "dpapi_protect_exports")?,
        counts,
    })
}

/// Write `neph-diagnostics-{timestamp}.zip` next to the database with `report.json` and recent log files.
pub fn write_diagnostic_zip(db_path: &Path, logs_dir: &Path) -> Result<PathBuf> {
    let report = collect_report(db_path)?;
    let parent = db_path.parent().unwrap_or_else(|| Path::new("."));
    let name = format!(
        "neph-diagnostics-{}.zip",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    );
    let out = parent.join(&name);
    let file = fs::File::create(&out)?;
    let mut zip = ZipWriter::new(file);
    let opts: FileOptions<()> = FileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file("report.json", opts)?;
    zip.write_all(serde_json::to_string_pretty(&report)?.as_bytes())?;
    if logs_dir.is_dir() {
        for entry in fs::read_dir(logs_dir)? {
            let entry = entry?;
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            let fname = p.file_name().and_then(|n| n.to_str()).unwrap_or("log.bin");
            if !fname.starts_with("neph") {
                continue;
            }
            let data = fs::read(&p)?;
            zip.start_file(format!("logs/{fname}"), opts)?;
            zip.write_all(&data)?;
        }
    }
    zip.finish()?;
    Ok(out)
}
