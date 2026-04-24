use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Result};

fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::Prefix(p) => out.push(p.as_os_str()),
            Component::RootDir => out.push(comp.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = out.pop();
            }
            Component::Normal(p) => out.push(p),
        }
    }
    out
}

fn is_protected_windows_prefix(lower: &str) -> bool {
    lower.starts_with("c:\\windows")
        || lower.starts_with("c:\\program files\\")
        || lower.starts_with("c:\\program files (x86)\\")
        || lower.contains("\\windows\\system32\\")
}

fn is_under_base(candidate: &Path, base: &Path) -> bool {
    let cand = normalize_path(candidate);
    let base = normalize_path(base);
    cand.starts_with(&base)
}

fn roots_allowlist() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(h) = dirs_next::home_dir() {
        roots.push(h);
    }
    if let Some(d) = dirs_next::data_local_dir() {
        roots.push(d);
    }
    roots.push(std::env::temp_dir());
    if let Ok(cd) = std::env::current_dir() {
        roots.push(cd);
    }
    roots
}

fn allowed_under_roots(path: &Path) -> bool {
    let roots = roots_allowlist();
    roots.iter().any(|root| {
        dunce::canonicalize(root)
            .ok()
            .map(|r| is_under_base(path, &r))
            .unwrap_or_else(|| is_under_base(path, root))
    })
}

/// Resolve and enforce safe paths for mutating file operations.
pub struct SafePathPolicy;

impl SafePathPolicy {
    /// `leaf_must_exist`: set `true` for delete/source of move; set `false` when the leaf file may not exist yet (move/rename destination).
    pub fn validate_user_path(path: &str, leaf_must_exist: bool) -> Result<PathBuf> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            bail!("path is empty");
        }
        let raw = Path::new(trimmed);
        let absolute = if raw.is_absolute() {
            raw.to_path_buf()
        } else {
            std::env::current_dir()?.join(raw)
        };
        let normalized = normalize_path(&absolute);
        let lower = normalized.to_string_lossy().to_lowercase();
        if lower.contains("..\\") || lower.contains("/../") {
            bail!("path contains traversal segments");
        }
        if is_protected_windows_prefix(&lower) {
            bail!("path is under a protected system location");
        }

        if normalized.exists() {
            let canon = dunce::canonicalize(&normalized)?;
            let canon_lower = canon.to_string_lossy().to_lowercase();
            if is_protected_windows_prefix(&canon_lower) {
                bail!("resolved path is under a protected system location");
            }
            if !allowed_under_roots(&canon) {
                bail!("path is outside allowed user locations (home, app data, temp, or current directory)");
            }
            return Ok(canon);
        }

        if leaf_must_exist {
            bail!("path does not exist");
        }

        let Some(parent) = normalized.parent() else {
            bail!("path has no parent directory");
        };
        if !parent.exists() {
            bail!("parent directory does not exist");
        }
        let parent_canon = dunce::canonicalize(parent)?;
        let pl = parent_canon.to_string_lossy().to_lowercase();
        if is_protected_windows_prefix(&pl) {
            bail!("parent resolves under a protected system location");
        }
        if !allowed_under_roots(&parent_canon) {
            bail!("parent path is outside allowed user locations");
        }
        Ok(normalized)
    }

    pub fn validate_move_endpoints(from: &str, to: &str) -> Result<(PathBuf, PathBuf)> {
        let from_pb = Self::validate_user_path(from, true)?;
        let to_pb = Self::validate_user_path(to, false)?;
        Ok((from_pb, to_pb))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn rejects_windows_prefix() {
        let p = "C:\\Windows\\System32\\drivers\\etc\\hosts";
        assert!(SafePathPolicy::validate_user_path(p, true).is_err());
    }

    #[test]
    fn allows_temp_file_under_tempdir() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("neph-path-test.txt");
        fs::write(&f, b"x").unwrap();
        let validated = SafePathPolicy::validate_user_path(f.to_str().unwrap(), true);
        assert!(validated.is_ok());
    }

    #[test]
    fn bad_path_matrix_parent_outside_allowlist() {
        // Non-existent path under system root should fail before touching disk
        assert!(SafePathPolicy::validate_user_path(r"C:\Windows\newfile.txt", false).is_err());
    }

    #[test]
    fn unicode_path_under_tempdir_round_trip() {
        let dir = tempdir().unwrap();
        let name = "neph-测试-ファイル.txt";
        let f = dir.path().join(name);
        fs::write(&f, b"x").unwrap();
        let validated = SafePathPolicy::validate_user_path(f.to_str().expect("utf-8 path"), true);
        assert!(validated.is_ok(), "{validated:?}");
        let move_r = SafePathPolicy::validate_move_endpoints(
            f.to_str().unwrap(),
            dir.path().join("moved-测试.txt").to_str().unwrap(),
        );
        assert!(move_r.is_ok(), "{move_r:?}");
    }
}
