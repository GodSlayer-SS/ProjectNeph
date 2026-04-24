#![allow(dead_code)]

use anyhow::anyhow;
use serde::Deserialize;
use serde_json::{Map, Value};

pub fn stable_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut out = Map::new();
            for k in keys {
                if let Some(v) = map.get(&k) {
                    out.insert(k, stable_json(v));
                }
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(stable_json).collect()),
        other => other.clone(),
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MoveFileArgs {
    from: String,
    to: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RenameFileArgs {
    path: String,
    new_name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PathArg {
    path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NoteIdBody {
    id: i64,
    body: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NoteIdOnly {
    id: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MemorySave {
    kind: String,
    content: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct QueryArg {
    query: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TextArg {
    text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CreateNote {
    body: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AgentDryRunArgs {
    steps_json: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct UrlArg {
    url: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RawArg {
    raw: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NameArg {
    name: String,
}

pub fn validate_tool_schema(tool: &str, args: &Value) -> anyhow::Result<()> {
    let err = |msg: &str| anyhow!(msg.to_string());
    match tool {
        "move_file" => {
            serde_json::from_value::<MoveFileArgs>(args.clone()).map_err(|_| err("invalid move_file args"))?;
        }
        "rename_file" => {
            serde_json::from_value::<RenameFileArgs>(args.clone()).map_err(|_| err("invalid rename_file args"))?;
        }
        "delete_file" | "pdf_read" | "repo_index" => {
            serde_json::from_value::<PathArg>(args.clone()).map_err(|_| err("invalid path args"))?;
        }
        "undo_action" | "context_snapshot" | "scan_apps" | "scan_files" | "system_info" | "list_skills"
        | "toggle_voice" | "screenshot_analyze" => {}
        "update_note" => {
            serde_json::from_value::<NoteIdBody>(args.clone()).map_err(|_| err("invalid update_note args"))?;
        }
        "delete_note" => {
            serde_json::from_value::<NoteIdOnly>(args.clone()).map_err(|_| err("invalid delete_note args"))?;
        }
        "save_memory" => {
            serde_json::from_value::<MemorySave>(args.clone()).map_err(|_| err("invalid save_memory args"))?;
        }
        "retrieve_memory" | "search_notes" | "search_files" | "launch_app" | "web_search" => {
            serde_json::from_value::<QueryArg>(args.clone()).map_err(|_| err("invalid query args"))?;
        }
        "web_fetch" => {
            serde_json::from_value::<UrlArg>(args.clone()).map_err(|_| err("invalid url args"))?;
        }
        "summarize" | "rewrite" | "code_explain" | "code_fix" => {
            serde_json::from_value::<TextArg>(args.clone()).map_err(|_| err("invalid text tool args"))?;
        }
        "schedule_reminder" | "patch_preview" => {
            serde_json::from_value::<RawArg>(args.clone()).map_err(|_| err("invalid reminder args"))?;
        }
        "run_skill" => {
            serde_json::from_value::<NameArg>(args.clone()).map_err(|_| err("invalid skill args"))?;
        }
        "create_note" => {
            serde_json::from_value::<CreateNote>(args.clone()).map_err(|_| err("invalid create_note args"))?;
        }
        "agent_dry_run" => {
            serde_json::from_value::<AgentDryRunArgs>(args.clone())
                .map_err(|_| err("invalid agent_dry_run args"))?;
        }
        _ => {}
    }
    Ok(())
}
