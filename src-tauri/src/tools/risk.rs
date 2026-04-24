use crate::models::RiskLevel;

pub fn tool_risk(tool_name: &str) -> RiskLevel {
    match tool_name {
        "delete_file" => RiskLevel::Red,
        "rename_file" | "move_file" | "save_memory" | "append_to_note" | "update_note"
        | "delete_note" => RiskLevel::Yellow,
        "scan_apps" | "scan_files" | "search_files" | "launch_app" | "undo_action"
        | "context_snapshot" | "web_search" | "web_fetch" | "code_explain" | "pdf_read"
        | "repo_index" | "system_info" | "list_skills" | "run_skill" | "toggle_voice"
        | "screenshot_analyze" => {
            RiskLevel::Green
        }
        "schedule_reminder" | "code_fix" | "patch_preview" => RiskLevel::Yellow,
        "run_project_tests" | "daily_brief" => RiskLevel::Green,
        _ => RiskLevel::Green,
    }
}
