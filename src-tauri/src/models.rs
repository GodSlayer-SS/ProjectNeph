use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: i64,
    pub kind: String,
    pub content: String,
    pub pinned: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntentProvenance {
    UserPrefix,
    LlmClassify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum PaletteRunResponse {
    #[serde(rename = "completed")]
    Completed { output: String },
    #[serde(rename = "needConfirmation")]
    NeedConfirmation {
        #[serde(rename = "planHash")]
        plan_hash: String,
        preview: String,
        risk: String,
        token: String,
    },
    #[serde(rename = "rejected")]
    Rejected { message: String },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunPalettePayload {
    pub input: String,
    pub confirmation_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: i64,
    pub input: String,
    pub intent: String,
    pub tool_name: Option<String>,
    pub success: Option<bool>,
    pub risk_level: Option<String>,
    pub state: Option<String>,
    pub result_summary: Option<String>,
    pub args_json: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub provenance: Option<String>,
    #[serde(default)]
    pub lineage_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Green,
    Yellow,
    Red,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Red => "red",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupDiagnostics {
    pub palette_hotkey: String,
    pub webview2_version: Option<String>,
    pub webview2_meets_minimum: bool,
    pub webview2_minimum: String,
    pub webview2_install_url: String,
    pub ime_hotkey_tip: String,
    /// True when sqlite-vec loaded at startup; stub semantic similarity is only meaningful when this is true.
    pub vector_search_enabled: bool,
    pub embedding_mode: Option<String>,
    pub dpapi_protect_exports: bool,
}
