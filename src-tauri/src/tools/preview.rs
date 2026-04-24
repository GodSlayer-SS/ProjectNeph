use serde::Deserialize;
use serde_json::Value;

use crate::path_policy::SafePathPolicy;
use crate::redaction::redact_json_value;

#[derive(Debug, Deserialize)]
struct MoveFileArgs {
    from: String,
    to: String,
}

#[derive(Debug, Deserialize)]
struct RenameFileArgs {
    path: String,
    new_name: String,
}

#[derive(Debug, Deserialize)]
struct PathArg {
    path: String,
}

#[derive(Debug, Deserialize)]
struct NoteIdBody {
    id: i64,
    body: String,
}

#[derive(Debug, Deserialize)]
struct NoteIdOnly {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct MemorySave {
    kind: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct TextArg {
    text: String,
}

pub fn redact_args_for_tool(tool: &str, args: &Value) -> Value {
    let mut redacted = redact_json_value(args);
    if matches!(tool, "summarize" | "rewrite" | "save_memory" | "create_note") {
        if let Value::Object(ref mut map) = redacted {
            for key in ["text", "content", "body"] {
                if let Some(Value::String(s)) = map.get(key) {
                    if s.len() > 200 {
                        let preview = s.chars().take(200).collect::<String>();
                        map.insert(
                            key.to_string(),
                            Value::String(format!("{preview}… [truncated {} chars]", s.len() - 200)),
                        );
                    }
                }
            }
        }
    }
    redacted
}

pub fn dry_run_preview(tool: &str, args: &Value) -> Result<String, String> {
    match tool {
        "move_file" => {
            let a: MoveFileArgs = serde_json::from_value(args.clone()).map_err(|e| e.to_string())?;
            let (from, to) = SafePathPolicy::validate_move_endpoints(&a.from, &a.to).map_err(|e| e.to_string())?;
            Ok(format!("Move file (after policy check):\n  From: {}\n  To: {}", from.display(), to.display()))
        }
        "rename_file" => {
            let a: RenameFileArgs = serde_json::from_value(args.clone()).map_err(|e| e.to_string())?;
            let path = SafePathPolicy::validate_user_path(&a.path, true).map_err(|e| e.to_string())?;
            Ok(format!(
                "Rename within directory:\n  Path: {}\n  New name: {}",
                path.display(),
                a.new_name
            ))
        }
        "delete_file" => {
            let a: PathArg = serde_json::from_value(args.clone()).map_err(|e| e.to_string())?;
            let path = SafePathPolicy::validate_user_path(&a.path, true).map_err(|e| e.to_string())?;
            Ok(format!("Send to Recycle Bin:\n  {}", path.display()))
        }
        "save_memory" => {
            let a: MemorySave = serde_json::from_value(args.clone()).map_err(|e| e.to_string())?;
            Ok(format!(
                "Save memory [{}]: {}…",
                a.kind,
                a.content.chars().take(120).collect::<String>()
            ))
        }
        "update_note" => {
            let a: NoteIdBody = serde_json::from_value(args.clone()).map_err(|e| e.to_string())?;
            Ok(format!("Update note #{} (new body length {} chars)", a.id, a.body.len()))
        }
        "delete_note" => {
            let a: NoteIdOnly = serde_json::from_value(args.clone()).map_err(|e| e.to_string())?;
            Ok(format!("Soft-delete note #{}", a.id))
        }
        "summarize" | "rewrite" => {
            let a: TextArg = serde_json::from_value(args.clone()).map_err(|e| e.to_string())?;
            Ok(format!(
                "{} text ({} chars)",
                if tool == "summarize" { "Summarize" } else { "Rewrite" },
                a.text.len()
            ))
        }
        other => Ok(format!("Run tool {other} (no structured preview).")),
    }
}
