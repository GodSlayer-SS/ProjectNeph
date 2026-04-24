use anyhow::Result;

use crate::apps;
use crate::files;

use super::AppState;

impl AppState {
    pub(crate) fn initial_scan_apps(&self) -> Result<()> {
        let conn = self.connect()?;
        let count: i64 = conn.query_row("SELECT COUNT(1) FROM app_index", [], |row| row.get(0))?;
        if count == 0 {
            let _ = apps::scan_start_menu_apps(&conn)?;
        }
        Ok(())
    }

    pub(crate) fn scan_apps(&self) -> Result<usize> {
        let conn = self.connect()?;
        apps::scan_start_menu_apps(&conn)
    }

    pub(crate) fn launch_app(&self, query: &str) -> Result<Option<String>> {
        let conn = self.connect()?;
        apps::launch_indexed_app(&conn, query)
    }

    pub(crate) fn scan_files(&self) -> Result<usize> {
        let conn = self.connect()?;
        files::scan_default_files(&conn, 5000)
    }

    pub(crate) fn search_files(&self, query: &str) -> Result<Vec<String>> {
        let conn = self.connect()?;
        files::search_files(&conn, query, 5)
    }
}
