use anyhow::Result;
use serde::{Deserialize, Serialize};

// Re-export from models so the trait API and the existing infrastructure
// use the same single type. Nothing outside this crate needs to know the origin.
pub use crate::models::RiskLevel;

// ── Tool Manifest ─────────────────────────────────────────────────────────────

/// Static description of a tool, declared in `tools.toml`.
/// The manifest is what the LLM planner sees; it is never constructed at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    pub name: String,
    pub description: String,
    pub risk: RiskLevel,
    /// Execution domain this tool operates in: "workspace" | "projects" |
    /// "personal" | "system" | "temp" | "browser-research" | "browser-tools" |
    /// "browser-personal" | "browser-throwaway" | "network" | "none"
    pub domain: String,
    /// Allowlisted egress hostnames for this tool (empty = no network).
    pub egress: Vec<String>,
    /// JSON Schema of the args object.
    pub args_schema: serde_json::Value,
}

// ── Execution types ───────────────────────────────────────────────────────────

/// Raw args from the planner — not yet validated.
pub type ToolArgs = serde_json::Value;

/// Type-state marker: args that have passed schema + domain validation.
#[derive(Debug, Clone)]
pub struct Validated(pub serde_json::Value);

/// Result returned from a successful tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub text: String,
    /// Optional structured data (used by browser tools, code tools, etc.)
    pub data: Option<serde_json::Value>,
    /// JSON-serialisable undo payload (stored in the `actions` audit table).
    pub undo_payload: Option<serde_json::Value>,
}

impl ToolOutput {
    pub fn text(s: impl Into<String>) -> Self {
        Self { text: s.into(), data: None, undo_payload: None }
    }
}

/// A single action inside a Plan, before execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAction {
    pub tool: String,
    pub args: ToolArgs,
    pub domain_handle: String,
}

// ── The Trait ─────────────────────────────────────────────────────────────────

/// Every executable capability implements `Tool`.
/// Tools are declared in `tools.toml`; the registry maps name → `Box<dyn Tool>`.
pub trait Tool: Send + Sync {
    fn manifest(&self) -> &ToolManifest;

    fn risk_level(&self) -> RiskLevel {
        self.manifest().risk.clone()
    }

    /// Schema-validate and domain-check args before execution.
    fn validate(&self, args: &ToolArgs) -> Result<Validated>;

    /// Execute with already-validated args. The `ExecCtx` carries the audit
    /// connection, domain reference, and cancellation token.
    fn execute(&self, args: Validated) -> Result<ToolOutput>;
}
