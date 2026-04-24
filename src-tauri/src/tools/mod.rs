pub mod preview;
pub mod registry;
pub mod risk;
pub mod schema;

pub use preview::{dry_run_preview, redact_args_for_tool};
pub use registry::build_registry;
pub use risk::tool_risk;
pub use schema::{stable_json, validate_tool_schema};

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{redact_args_for_tool, validate_tool_schema};

    #[test]
    fn redact_tool_args_truncates_long_content() {
        let long = "x".repeat(300);
        let args = json!({ "content": long });
        let out = redact_args_for_tool("save_memory", &args);
        let s = out["content"].as_str().unwrap_or_default();
        assert!(s.contains("truncated"));
        assert!(s.len() < long.len());
    }

    #[test]
    fn schema_validation_accepts_known_shapes() {
        let cases = vec![
            ("create_note", json!({"body":"x"})),
            ("search_notes", json!({"query":"x"})),
            ("save_memory", json!({"kind":"fact","content":"x"})),
            ("retrieve_memory", json!({"query":"x"})),
            ("search_files", json!({"query":"x"})),
            ("launch_app", json!({"query":"x"})),
            ("move_file", json!({"from":"a","to":"b"})),
            ("rename_file", json!({"path":"a","new_name":"b"})),
            ("delete_file", json!({"path":"a"})),
            ("update_note", json!({"id":1,"body":"x"})),
            ("delete_note", json!({"id":1})),
            ("summarize", json!({"text":"x"})),
            ("rewrite", json!({"text":"x"})),
            ("agent_dry_run", json!({"steps_json":"[]"})),
            ("web_search", json!({"query":"rust tauri"})),
            ("web_fetch", json!({"url":"https://example.com"})),
            ("code_explain", json!({"text":"fn main(){}"})),
            ("code_fix", json!({"text":"panic!"})),
            ("pdf_read", json!({"path":"C:/tmp/a.pdf"})),
            ("repo_index", json!({"path":"C:/tmp/repo"})),
            ("schedule_reminder", json!({"raw":"10m standup"})),
            ("patch_preview", json!({"raw":"improve error handling"})),
        ];
        for (tool, args) in cases {
            assert!(validate_tool_schema(tool, &args).is_ok(), "schema failed for {tool}");
        }
    }
}
