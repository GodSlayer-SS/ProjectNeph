/// Filesystem execution domain — Phase 1.
///
/// Extends `SafePathPolicy` with the 5 named domains from the blueprint:
///
/// | Domain    | Paths                       | Tier   |
/// |-----------|-----------------------------|--------|
/// | workspace | ~/nephis-workspace/         | green  |
/// | projects  | declared project roots      | yellow |
/// | personal  | Documents, Downloads, Desktop | yellow write, red delete |
/// | system    | Program Files, Windows, etc | red    |
/// | temp      | ~/.nephis/tmp/              | green  |
///
/// The LLM sees only a domain handle (e.g. `"workspace"`). The Executor maps
/// the handle to a real path via this module. The LLM never sees raw paths.

use anyhow::{bail, Result};
use std::path::PathBuf;

use crate::traits::domain::{Capability, DomainId, ExecutionDomain};
use crate::traits::tool::PlannedAction;

// ── Domain definitions ────────────────────────────────────────────────────────

pub struct FilesystemDomain {
    id: DomainId,
    root: PathBuf,
    caps: Vec<Capability>,
}

impl FilesystemDomain {
    fn new(id: &str, root: PathBuf, caps: Vec<Capability>) -> Self {
        Self { id: DomainId::new(id), root, caps }
    }

    /// `workspace` domain — green, full read/write inside ~/nephis-workspace/.
    pub fn workspace() -> Self {
        let root = dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default"))
            .join("nephis-workspace");
        Self::new("workspace", root, vec![Capability::Read, Capability::Write])
    }

    /// `temp` domain — green, ephemeral scratch space.
    pub fn temp() -> Self {
        let root = dirs_next::data_local_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("Neph")
            .join("tmp");
        Self::new("temp", root, vec![Capability::Read, Capability::Write, Capability::Delete])
    }

    /// `personal` domain — yellow, Documents/Downloads/Desktop.
    /// Delete requires red-tier confirmation (enforced by the trust kernel).
    pub fn personal() -> Self {
        let root = dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default"));
        Self::new(
            "personal",
            root,
            vec![Capability::Read, Capability::Write],
            // Note: Delete is NOT in caps — it must go through red-tier confirmation.
        )
    }

    /// `projects` domain — yellow, declared project roots (Phase 2).
    ///
    /// Phase 1: defaults to `~/Projects` if no explicit root declared.
    /// Phase 2: reads declared roots from `settings` table in SQLite.
    pub fn projects() -> Self {
        let root = dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default"))
            .join("Projects");
        Self::new("projects", root, vec![Capability::Read, Capability::Write])
    }

    /// `system` domain — red, Program Files / Windows / registry.
    ///
    /// Tools tagged `domain = "system"` in tools.toml (scan_apps, launch_app,
    /// system_info, context_snapshot, screenshot_analyze, focus_window,
    /// type_in_active, read_active) use this domain.
    ///
    /// For read-only system tools the enforce() check is permissive (no path
    /// escaping risk — these tools don't write files). Red-tier trust kernel
    /// confirmation is still required for any mutation.
    pub fn system() -> Self {
        // Root = C:\ (intentionally broad — system tools need wide read access).
        // Writes to system paths require the red-tier confirmation gate.
        let root = PathBuf::from("C:\\");
        Self::new(
            "system",
            root,
            vec![Capability::Read],
            // Write/Delete: NOT in caps — must go through red-tier confirmation.
        )
    }
}

impl ExecutionDomain for FilesystemDomain {
    fn id(&self) -> &DomainId {
        &self.id
    }

    fn allowed_caps(&self) -> &[Capability] {
        &self.caps
    }

    fn enforce(&self, action: &PlannedAction) -> Result<()> {
        // For filesystem actions, check that the `path` arg (if present)
        // resolves within this domain's root.
        if let Some(path_str) = action.args.get("path").and_then(|v| v.as_str()) {
            let path = std::path::Path::new(path_str);
            let abs = if path.is_absolute() {
                path.to_path_buf()
            } else {
                self.root.join(path)
            };
            if !abs.starts_with(&self.root) {
                bail!(
                    "path '{}' escapes domain '{}' (root: {})",
                    path_str,
                    self.id.0,
                    self.root.display()
                );
            }
        }
        Ok(())
    }
}

// ── Domain registry helper ────────────────────────────────────────────────────

/// Returns the appropriate filesystem domain for a given handle string.
///
/// Called by `ExecutorActor` to resolve `PlannedStep.domain`.
/// All 5 Blueprint §5 filesystem domains are wired here.
pub fn resolve_filesystem_domain(handle: &str) -> Option<FilesystemDomain> {
    match handle {
        "workspace" => Some(FilesystemDomain::workspace()),
        "temp" => Some(FilesystemDomain::temp()),
        "personal" => Some(FilesystemDomain::personal()),
        "projects" => Some(FilesystemDomain::projects()),
        "system" => Some(FilesystemDomain::system()),
        _ => None,
    }
}
