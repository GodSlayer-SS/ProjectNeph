use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::models::HistoryItem;

use super::AppState;

impl AppState {
    pub fn history(&self) -> Result<Vec<HistoryItem>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT ch.id, ch.input, ch.intent, ch.tool_name, ch.success, ch.created_at,
                    ch.provenance, ch.lineage_json,
                    a.risk_level, a.state, a.result_summary, a.args_json
             FROM command_history ch
             LEFT JOIN actions a ON a.command_id = ch.id
             ORDER BY ch.id DESC
             LIMIT 100",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(HistoryItem {
                id: row.get(0)?,
                input: row.get(1)?,
                intent: row.get(2)?,
                tool_name: row.get(3)?,
                success: row.get::<_, Option<i64>>(4)?.map(|value| value == 1),
                created_at: row.get(5)?,
                provenance: row.get(6)?,
                lineage_json: row.get(7)?,
                risk_level: row.get(8)?,
                state: row.get(9)?,
                result_summary: row.get(10)?,
                args_json: row.get(11)?,
            })
        })?;
        Ok(rows.filter_map(|item| item.ok()).collect())
    }

    pub fn export_db_backup(&self) -> Result<String> {
        let source = self.db_path.clone();
        let backup_dir = source
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("backups");
        fs::create_dir_all(&backup_dir)?;
        let file = format!("neph-{}.db", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
        let target = backup_dir.join(file);
        fs::copy(&source, &target)?;
        Ok(target.to_string_lossy().to_string())
    }

    pub fn export_memory_json(&self) -> Result<String> {
        let items = self.list_memory(None)?;
        let source = self.db_path.clone();
        let backup_dir = source
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("backups");
        fs::create_dir_all(&backup_dir)?;
        let protect = crate::db::read_setting(&self.db_path, "dpapi_protect_exports")?
            .map(|v| v == "1")
            .unwrap_or(false);
        let body = serde_json::to_vec_pretty(&items)?;
        let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S");
        let target = if protect {
            #[cfg(windows)]
            {
                let enc = crate::dpapi_win::protect_user_bytes(&body)?;
                let round = crate::dpapi_win::unprotect_user_bytes(&enc)?;
                if round != body {
                    anyhow::bail!("DPAPI export verification failed (round-trip mismatch)");
                }
                let path = backup_dir.join(format!("memory-{ts}.json.dpapi"));
                fs::write(&path, enc)?;
                path
            }
            #[cfg(not(windows))]
            {
                anyhow::bail!("DPAPI-protected exports are only supported on Windows");
            }
        } else {
            let path = backup_dir.join(format!("memory-{ts}.json"));
            fs::write(&path, body)?;
            path
        };
        Ok(target.to_string_lossy().to_string())
    }

    pub fn token_usage(&self) -> (u64, u64) {
        self.token_usage.lock().map(|v| *v).unwrap_or((0, 0))
    }
}
