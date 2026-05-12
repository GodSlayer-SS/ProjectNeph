use crate::models::RiskLevel;
use crate::tools::manifest::Manifest;

/// Return the risk level for a tool name.
///
/// Priority:
///   1. `tools.toml` manifest (loaded once at startup)
///   2. Hard-coded fallback table (keeps tests working when TOML is absent)
pub fn tool_risk(tool_name: &str) -> RiskLevel {
    let manifest = Manifest::get();
    if manifest.is_loaded() {
        return manifest.risk(tool_name);
    }
    // Fallback — mirrors the original hard-coded table.
    hardcoded_risk(tool_name)
}

fn hardcoded_risk(tool_name: &str) -> RiskLevel {
    match tool_name {
        "delete_file" => RiskLevel::Red,
        "rename_file" | "move_file" | "save_memory" | "append_to_note" | "update_note"
        | "delete_note" | "schedule_reminder" | "code_fix" | "patch_preview" => {
            RiskLevel::Yellow
        }
        _ => RiskLevel::Green,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_file_is_red() {
        // Works via fallback even if TOML is absent.
        assert_eq!(hardcoded_risk("delete_file"), RiskLevel::Red);
    }

    #[test]
    fn unknown_tool_is_green() {
        assert_eq!(hardcoded_risk("totally_unknown_tool"), RiskLevel::Green);
    }
}

