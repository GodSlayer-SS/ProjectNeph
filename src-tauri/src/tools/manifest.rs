/// tools/manifest.rs — Runtime loader for `apps/desktop/tools.toml`.
///
/// The manifest is loaded once at startup and cached in a global `OnceLock`.
/// Both `risk.rs` and `schema.rs` delegate to `Manifest::get()` instead of
/// maintaining their own hard-coded match tables.
///
/// The TOML is loaded from one of (in priority order):
///   1. `NEPH_TOOLS_TOML` env var  — useful for tests
///   2. Relative to the binary: `../apps/desktop/tools.toml`  — dev layout
///   3. Relative to the binary: `tools.toml`  — bundled layout (future)

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::models::RiskLevel;

// ── TOML schema types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RawManifest {
    tool: Vec<RawTool>,
}

#[derive(Debug, Deserialize)]
struct RawTool {
    name: String,
    risk: String,
    description: String,
    domain: String,
    egress: Vec<String>,
    phase: Option<u8>,
    args: Option<toml::Table>,
}

// ── Public entry ──────────────────────────────────────────────────────────────

/// A parsed tool entry from tools.toml.
#[derive(Debug, Clone)]
pub struct ToolEntry {
    pub name: String,
    pub risk: RiskLevel,
    pub description: String,
    pub domain: String,
    pub egress: Vec<String>,
    pub phase: u8,
    /// Arg names → (type, required).
    pub args: HashMap<String, ArgSpec>,
}

#[derive(Debug, Clone)]
pub struct ArgSpec {
    pub arg_type: String, // "string" | "integer" | "boolean"
    pub required: bool,
}

/// The loaded manifest.
pub struct Manifest {
    by_name: HashMap<String, ToolEntry>,
}

static MANIFEST: OnceLock<Manifest> = OnceLock::new();

impl Manifest {
    /// Return the global manifest, loading it on first call.
    pub fn get() -> &'static Manifest {
        MANIFEST.get_or_init(|| {
            Manifest::load().unwrap_or_else(|e| {
                tracing::warn!(error = %e, "tools.toml not loaded; using hard-coded fallback risk map");
                Manifest { by_name: HashMap::new() }
            })
        })
    }

    fn load() -> anyhow::Result<Manifest> {
        let path = locate_toml()?;
        let src = std::fs::read_to_string(&path)?;
        let raw: RawManifest = toml::from_str(&src)?;
        let mut by_name = HashMap::new();
        for t in raw.tool {
            let risk = parse_risk(&t.risk);
            let args = parse_args(t.args.as_ref());
            by_name.insert(
                t.name.clone(),
                ToolEntry {
                    name: t.name,
                    risk,
                    description: t.description,
                    domain: t.domain,
                    egress: t.egress,
                    phase: t.phase.unwrap_or(1),
                    args,
                },
            );
        }
        tracing::info!(tools = by_name.len(), path = %path.display(), "tools.toml loaded");
        Ok(Manifest { by_name })
    }

    /// Look up a tool by name.
    pub fn tool(&self, name: &str) -> Option<&ToolEntry> {
        self.by_name.get(name)
    }

    /// Return all tools in the manifest (cloned, stable order by name).
    pub fn all_tools(&self) -> Vec<ToolEntry> {
        let mut v = self.by_name.values().cloned().collect::<Vec<_>>();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }

    /// Whether the manifest was successfully loaded (non-empty).
    pub fn is_loaded(&self) -> bool {
        !self.by_name.is_empty()
    }

    /// Risk level for a tool name. Returns Green if unknown (safe default).
    pub fn risk(&self, name: &str) -> RiskLevel {
        self.tool(name)
            .map(|t| t.risk.clone())
            .unwrap_or(RiskLevel::Green)
    }

    /// Whether the `args` object for a given tool contains all required fields.
    pub fn validate_args(&self, tool_name: &str, args: &serde_json::Value) -> anyhow::Result<()> {
        let Some(entry) = self.tool(tool_name) else {
            // Unknown tool — tolerate in Phase 1.
            return Ok(());
        };
        for (name, spec) in &entry.args {
            if spec.required && args.get(name.as_str()).is_none() {
                anyhow::bail!("tool '{}' missing required arg '{}'", tool_name, name);
            }
        }
        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn locate_toml() -> anyhow::Result<std::path::PathBuf> {
    // 1) Env var override (tests, CI)
    if let Ok(p) = std::env::var("NEPH_TOOLS_TOML") {
        let path = std::path::PathBuf::from(p);
        if path.exists() {
            return Ok(path);
        }
    }

    // 2) Locate relative to the exe: ../apps/desktop/tools.toml (dev layout)
    if let Ok(exe) = std::env::current_exe() {
        let dev_path = exe
            .parent()
            .and_then(|p| p.parent())  // out of /target/debug or /target/release
            .and_then(|p| p.parent())  // out of src-tauri
            .map(|root| root.join("apps").join("desktop").join("tools.toml"));
        if let Some(p) = dev_path {
            if p.exists() {
                return Ok(p);
            }
        }

        // 3) Bundled: next to the exe
        let bundled = exe.parent().map(|p| p.join("tools.toml"));
        if let Some(p) = bundled {
            if p.exists() {
                return Ok(p);
            }
        }
    }

    anyhow::bail!("tools.toml not found; set NEPH_TOOLS_TOML env var or place it next to the binary")
}

fn parse_risk(s: &str) -> RiskLevel {
    match s.to_lowercase().as_str() {
        "red" => RiskLevel::Red,
        "yellow" => RiskLevel::Yellow,
        _ => RiskLevel::Green,
    }
}

fn parse_args(table: Option<&toml::Table>) -> HashMap<String, ArgSpec> {
    let mut out = HashMap::new();
    let Some(table) = table else { return out };
    for (name, val) in table {
        if let Some(spec_table) = val.as_table() {
            let arg_type = spec_table
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("string")
                .to_string();
            let required = spec_table
                .get("required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            out.insert(name.clone(), ArgSpec { arg_type, required });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_risk_works() {
        assert_eq!(parse_risk("red"), RiskLevel::Red);
        assert_eq!(parse_risk("Yellow"), RiskLevel::Yellow);
        assert_eq!(parse_risk("green"), RiskLevel::Green);
        assert_eq!(parse_risk("unknown"), RiskLevel::Green);
    }
}
