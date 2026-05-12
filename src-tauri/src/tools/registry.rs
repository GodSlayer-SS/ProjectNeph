#![allow(dead_code)]

use std::collections::HashMap;

use serde_json::Value;

use crate::models::RiskLevel;

use super::risk::tool_risk;

pub trait ToolDefinition: Send + Sync {
    fn name(&self) -> &'static str;
    fn risk(&self) -> RiskLevel;
    fn schema(&self) -> Value;
}

pub struct RegistryTool {
    pub tool_name: &'static str,
    pub tool_risk: RiskLevel,
    pub tool_schema: Value,
}

impl ToolDefinition for RegistryTool {
    fn name(&self) -> &'static str {
        self.tool_name
    }

    fn risk(&self) -> RiskLevel {
        self.tool_risk.clone()
    }

    fn schema(&self) -> Value {
        self.tool_schema.clone()
    }
}

pub fn build_registry() -> HashMap<&'static str, Box<dyn ToolDefinition>> {
    let mut map: HashMap<&'static str, Box<dyn ToolDefinition>> = HashMap::new();
    let register = |name: &'static str| -> Box<dyn ToolDefinition> {
        Box::new(RegistryTool {
            tool_name: name,
            tool_risk: tool_risk(name),
            tool_schema: serde_json::json!({"type":"object"}),
        })
    };
    for name in [
        "create_note",
        "search_notes",
        "save_memory",
        "retrieve_memory",
        "search_files",
        "scan_files",
        "scan_apps",
        "launch_app",
        "move_file",
        "rename_file",
        "delete_file",
        "undo_action",
        "summarize",
        "rewrite",
        "agent_dry_run",
        "context_snapshot",
        "web_search",
        "web_fetch",
        "code_explain",
        "code_fix",
        "pdf_read",
        "repo_index",
        "system_info",
        "schedule_reminder",
        "patch_preview",
        "run_project_tests",
        "daily_brief",
        "list_skills",
        "run_skill",
        "toggle_voice",
        "screenshot_analyze",
        "browser_read_page",
        "browser_search",
        "browser_read_page_personal",
        "browser_search_personal",
        "browser_click",
        "browser_fill_form",
        "focus_window",
        "type_in_active",
        "read_active",
        "organize_files_template",
        "code_companion_diff",
        "toggle_wake_word",
        "toggle_mcp_bridge",
        "toggle_orb_v2",
    ] {
        map.insert(name, register(name));
    }
    map
}
