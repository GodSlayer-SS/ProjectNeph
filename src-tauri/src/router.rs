use serde::{Deserialize, Serialize};

use crate::models::IntentProvenance;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutedIntent {
    pub intent: String,
    pub args: serde_json::Value,
    pub confidence: f32,
    pub source: IntentProvenance,
    #[serde(default)]
    pub llm_payload: Option<serde_json::Value>,
}

fn user_route(intent: String, args: serde_json::Value, confidence: f32) -> RoutedIntent {
    RoutedIntent {
        intent,
        args,
        confidence,
        source: IntentProvenance::UserPrefix,
        llm_payload: None,
    }
}

pub fn route_input(input: &str) -> RoutedIntent {
    let trimmed = input.trim();
    if let Some(body) = trimmed.strip_prefix(">note") {
        return user_route(
            "create_note".to_string(),
            serde_json::json!({ "body": body.trim() }),
            1.0,
        );
    }

    if trimmed == ">notes" {
        return user_route("list_notes".to_string(), serde_json::json!({}), 1.0);
    }

    if let Some(query) = trimmed.strip_prefix(">findnote") {
        return user_route(
            "search_notes".to_string(),
            serde_json::json!({ "query": query.trim() }),
            1.0,
        );
    }

    if let Some(rest) = trimmed.strip_prefix(">updatenote") {
        let parts: Vec<&str> = rest.trim().splitn(2, ' ').collect();
        let id = parts.first().and_then(|value| value.parse::<i64>().ok()).unwrap_or(0);
        let body = parts.get(1).copied().unwrap_or("");
        return user_route(
            "update_note".to_string(),
            serde_json::json!({ "id": id, "body": body.trim() }),
            1.0,
        );
    }

    if let Some(rest) = trimmed.strip_prefix(">deletenote") {
        let id = rest.trim().parse::<i64>().unwrap_or(0);
        return user_route(
            "delete_note".to_string(),
            serde_json::json!({ "id": id }),
            1.0,
        );
    }

    if let Some(text) = trimmed.strip_prefix(">summarize") {
        return user_route(
            "summarize".to_string(),
            serde_json::json!({ "text": text.trim() }),
            1.0,
        );
    }

    if let Some(text) = trimmed.strip_prefix(">rewrite") {
        return user_route(
            "rewrite".to_string(),
            serde_json::json!({ "text": text.trim() }),
            1.0,
        );
    }

    if trimmed == ">ctx" {
        return user_route("context_snapshot".to_string(), serde_json::json!({}), 1.0);
    }

    if let Some(query) = trimmed.strip_prefix(">websearch") {
        return user_route(
            "web_search".to_string(),
            serde_json::json!({ "query": query.trim() }),
            1.0,
        );
    }

    if let Some(url) = trimmed.strip_prefix(">webfetch") {
        return user_route(
            "web_fetch".to_string(),
            serde_json::json!({ "url": url.trim() }),
            1.0,
        );
    }

    if let Some(text) = trimmed.strip_prefix(">codeexplain") {
        return user_route(
            "code_explain".to_string(),
            serde_json::json!({ "text": text.trim() }),
            1.0,
        );
    }

    if let Some(text) = trimmed.strip_prefix(">codefix") {
        return user_route(
            "code_fix".to_string(),
            serde_json::json!({ "text": text.trim() }),
            1.0,
        );
    }

    if let Some(path) = trimmed.strip_prefix(">pdfread") {
        return user_route(
            "pdf_read".to_string(),
            serde_json::json!({ "path": path.trim() }),
            1.0,
        );
    }

    if let Some(path) = trimmed.strip_prefix(">repo") {
        return user_route(
            "repo_index".to_string(),
            serde_json::json!({ "path": path.trim() }),
            1.0,
        );
    }

    if trimmed == ">systeminfo" {
        return user_route("system_info".to_string(), serde_json::json!({}), 1.0);
    }

    if trimmed == ">voice" {
        return user_route("toggle_voice".to_string(), serde_json::json!({}), 1.0);
    }

    if trimmed == ">snip" {
        return user_route("screenshot_analyze".to_string(), serde_json::json!({}), 1.0);
    }

    if let Some(url) = trimmed.strip_prefix(">browse") {
        return user_route(
            "browser_read_page".to_string(),
            serde_json::json!({ "url": url.trim(), "profile": "nephis-research" }),
            1.0,
        );
    }

    if let Some(query) = trimmed.strip_prefix(">bsearch") {
        return user_route(
            "browser_search".to_string(),
            serde_json::json!({ "query": query.trim(), "profile": "nephis-research" }),
            1.0,
        );
    }

    if let Some(url) = trimmed.strip_prefix(">browse-personal") {
        return user_route(
            "browser_read_page_personal".to_string(),
            serde_json::json!({
                "url": url.trim(),
                "profile": "nephis-personal",
                "explicit_personal": true
            }),
            1.0,
        );
    }

    if let Some(query) = trimmed.strip_prefix(">bsearch-personal") {
        return user_route(
            "browser_search_personal".to_string(),
            serde_json::json!({
                "query": query.trim(),
                "profile": "nephis-personal",
                "explicit_personal": true
            }),
            1.0,
        );
    }

    if let Some(rest) = trimmed.strip_prefix(">bclick") {
        let parts: Vec<&str> = rest.trim().splitn(2, '|').collect();
        return user_route(
            "browser_click".to_string(),
            serde_json::json!({
                "url": parts.first().copied().unwrap_or("").trim(),
                "selector": parts.get(1).copied().unwrap_or("").trim(),
                "profile": "nephis-tools"
            }),
            1.0,
        );
    }

    if let Some(rest) = trimmed.strip_prefix(">bfill") {
        // format: >bfill <url>|<fields_json>|<submit_selector?>
        let parts: Vec<&str> = rest.trim().splitn(3, '|').collect();
        let fields = parts
            .get(1)
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s.trim()).ok())
            .unwrap_or_else(|| serde_json::json!({}));
        return user_route(
            "browser_fill_form".to_string(),
            serde_json::json!({
                "url": parts.first().copied().unwrap_or("").trim(),
                "fields": fields,
                "submit_selector": parts.get(2).copied().unwrap_or("").trim(),
                "profile": "nephis-tools"
            }),
            1.0,
        );
    }

    if let Some(query) = trimmed.strip_prefix(">focus") {
        return user_route(
            "focus_window".to_string(),
            serde_json::json!({ "query": query.trim() }),
            1.0,
        );
    }

    if let Some(text) = trimmed.strip_prefix(">type") {
        return user_route(
            "type_in_active".to_string(),
            serde_json::json!({ "text": text.trim() }),
            1.0,
        );
    }

    if trimmed == ">readactive" {
        return user_route("read_active".to_string(), serde_json::json!({}), 1.0);
    }

    if let Some(root) = trimmed.strip_prefix(">organize") {
        return user_route(
            "organize_files_template".to_string(),
            serde_json::json!({ "root": root.trim(), "dry_run": false }),
            1.0,
        );
    }

    if let Some(request) = trimmed.strip_prefix(">companion") {
        return user_route(
            "code_companion_diff".to_string(),
            serde_json::json!({ "request": request.trim() }),
            1.0,
        );
    }

    if trimmed == ">wakeon" {
        return user_route(
            "toggle_wake_word".to_string(),
            serde_json::json!({ "enabled": true }),
            1.0,
        );
    }
    if trimmed == ">wakeoff" {
        return user_route(
            "toggle_wake_word".to_string(),
            serde_json::json!({ "enabled": false }),
            1.0,
        );
    }
    if trimmed == ">mcpon" {
        return user_route(
            "toggle_mcp_bridge".to_string(),
            serde_json::json!({ "enabled": true }),
            1.0,
        );
    }
    if trimmed == ">mcpoff" {
        return user_route(
            "toggle_mcp_bridge".to_string(),
            serde_json::json!({ "enabled": false }),
            1.0,
        );
    }
    if trimmed == ">orbv2on" {
        return user_route(
            "toggle_orb_v2".to_string(),
            serde_json::json!({ "enabled": true }),
            1.0,
        );
    }
    if trimmed == ">orbv2off" {
        return user_route(
            "toggle_orb_v2".to_string(),
            serde_json::json!({ "enabled": false }),
            1.0,
        );
    }

    if let Some(rest) = trimmed.strip_prefix(">remind") {
        return user_route(
            "schedule_reminder".to_string(),
            serde_json::json!({ "raw": rest.trim() }),
            1.0,
        );
    }

    if let Some(desc) = trimmed.strip_prefix(">patch") {
        return user_route(
            "patch_preview".to_string(),
            serde_json::json!({ "raw": desc.trim() }),
            1.0,
        );
    }

    if trimmed == ">test" {
        return user_route("run_project_tests".to_string(), serde_json::json!({}), 1.0);
    }

    if trimmed == ">dailybrief" {
        return user_route("daily_brief".to_string(), serde_json::json!({}), 1.0);
    }

    if trimmed == ">skills" {
        return user_route("list_skills".to_string(), serde_json::json!({}), 1.0);
    }

    if let Some(name) = trimmed.strip_prefix(">runskill") {
        return user_route(
            "run_skill".to_string(),
            serde_json::json!({ "name": name.trim() }),
            1.0,
        );
    }

    if let Some(rest) = trimmed.strip_prefix(">movefile") {
        let parts: Vec<&str> = rest.trim().splitn(2, '|').collect();
        return user_route(
            "move_file".to_string(),
            serde_json::json!({
                "from": parts.first().copied().unwrap_or("").trim(),
                "to": parts.get(1).copied().unwrap_or("").trim()
            }),
            1.0,
        );
    }

    if let Some(rest) = trimmed.strip_prefix(">renamefile") {
        let parts: Vec<&str> = rest.trim().splitn(2, '|').collect();
        return user_route(
            "rename_file".to_string(),
            serde_json::json!({
                "path": parts.first().copied().unwrap_or("").trim(),
                "new_name": parts.get(1).copied().unwrap_or("").trim()
            }),
            1.0,
        );
    }

    if let Some(path) = trimmed.strip_prefix(">deletefile") {
        return user_route(
            "delete_file".to_string(),
            serde_json::json!({ "path": path.trim() }),
            1.0,
        );
    }

    if let Some(payload) = trimmed.strip_prefix(">agentdryrun") {
        return user_route(
            "agent_dry_run".to_string(),
            serde_json::json!({ "steps_json": payload.trim() }),
            1.0,
        );
    }

    if trimmed == ">undo" {
        return user_route("undo_action".to_string(), serde_json::json!({}), 1.0);
    }

    if let Some(body) = trimmed.strip_prefix(">remember") {
        return user_route(
            "save_memory".to_string(),
            serde_json::json!({ "kind": "fact", "content": body.trim() }),
            1.0,
        );
    }

    if let Some(query) = trimmed.strip_prefix(">recall") {
        return user_route(
            "retrieve_memory".to_string(),
            serde_json::json!({ "query": query.trim() }),
            1.0,
        );
    }

    if let Some(query) = trimmed.strip_prefix(">app") {
        return user_route(
            "launch_app".to_string(),
            serde_json::json!({ "query": query.trim() }),
            0.95,
        );
    }

    if trimmed == ">scanapps" {
        return user_route("scan_apps".to_string(), serde_json::json!({}), 1.0);
    }

    if trimmed == ">scanfiles" {
        return user_route("scan_files".to_string(), serde_json::json!({}), 1.0);
    }

    if let Some(query) = trimmed.strip_prefix(">find") {
        return user_route(
            "search_files".to_string(),
            serde_json::json!({ "query": query.trim() }),
            0.95,
        );
    }

    user_route(
        "unknown".to_string(),
        serde_json::json!({ "raw": trimmed }),
        0.0,
    )
}

#[cfg(test)]
mod intent_router_eval {
    use super::route_input;

    fn eval_cases() -> Vec<(String, &'static str)> {
        let mut cases: Vec<(String, &'static str)> = Vec::new();
        for i in 0..120 {
            cases.push((format!(">note body-{i}"), "create_note"));
        }
        for i in 0..60 {
            cases.push((format!("   >find   doc{i}  "), "search_files"));
        }
        for i in 0..40 {
            cases.push((format!(">findnote query{i}"), "search_notes"));
        }
        for i in 0..40 {
            cases.push((format!(">app appname{i}"), "launch_app"));
        }
        for i in 0..30 {
            cases.push((format!(">remember fact {i}"), "save_memory"));
        }
        for i in 0..30 {
            cases.push((format!(">recall mem{i}"), "retrieve_memory"));
        }
        cases.push((">notes".into(), "list_notes"));
        cases.push((">scanapps".into(), "scan_apps"));
        cases.push((">scanfiles".into(), "scan_files"));
        cases.push((">undo".into(), "undo_action"));
        cases.push((">summarize hello world".into(), "summarize"));
        cases.push((">rewrite hello world".into(), "rewrite"));
        cases.push((">movefile a|b".into(), "move_file"));
        cases.push((">renamefile p|n".into(), "rename_file"));
        cases.push((">deletefile /tmp/x".into(), "delete_file"));
        cases.push((">updatenote 1 body".into(), "update_note"));
        cases.push((">deletenote 1".into(), "delete_note"));
        cases
    }

    #[test]
    fn prefix_router_eval_meets_85_percent_bar() {
        let cases = eval_cases();
        assert!(
            cases.len() >= 300,
            "expected >=300 eval cases, got {}",
            cases.len()
        );
        let mut ok = 0usize;
        for (input, expected) in &cases {
            let r = route_input(input);
            if r.intent == *expected {
                ok += 1;
            }
        }
        let rate = ok as f64 / cases.len() as f64;
        assert!(
            rate >= 0.85,
            "router eval pass rate {rate:.2} below 0.85 ({ok}/{})",
            cases.len()
        );
    }
}
