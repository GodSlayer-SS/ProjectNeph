use anyhow::Result;
use serde::{Deserialize, Serialize};

// ── Intent ────────────────────────────────────────────────────────────────────

/// A classified user intent, before planning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    /// The intent label (e.g. "create_note", "general_chat", "browse_web").
    pub label: String,
    /// Raw user text that produced this intent.
    pub raw_input: String,
    /// Confidence in [0, 1].
    pub confidence: f32,
}

// ── Plan ─────────────────────────────────────────────────────────────────────

/// A single step in an execution plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedStep {
    /// Tool to invoke (matches a name in `tools.toml`).
    pub tool: String,
    /// Arguments for the tool (not yet validated).
    pub args: serde_json::Value,
    /// Domain handle the tool will run in.
    pub domain: String,
}

/// An ordered list of steps that the `ExecutorActor` will walk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// Unique content-hash of the plan (SHA-256 of stable JSON).
    pub plan_hash: String,
    pub steps: Vec<PlannedStep>,
    /// Raw LLM output that generated this plan (for audit / lineage).
    pub raw_llm_output: String,
}

impl Plan {
    /// A degenerate plan with a single LLM-chat step (Phase 1 default).
    pub fn chat_only(raw_llm_output: impl Into<String>) -> Self {
        let output = raw_llm_output.into();
        let hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut h = DefaultHasher::new();
            output.hash(&mut h);
            format!("{:x}", h.finish())
        };
        Self { plan_hash: hash, steps: vec![], raw_llm_output: output }
    }
}

// ── Planner context ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PlannerCtx {
    /// Active session memories injected into the system prompt.
    pub memory_snippets: Vec<String>,
    /// The tools the planner is allowed to use (from tools.toml).
    pub available_tools: Vec<String>,
}

// ── The Trait ─────────────────────────────────────────────────────────────────

/// The only component allowed to call an `LlmProvider` for orchestration.
///
/// Phase 1: `SimplePlanner` — just streams a Gemini Flash reply, returns a
///           `Plan::chat_only()` with no steps.
/// Phase 2: `StructuredPlanner` — produces typed multi-step Plans from JSON
///           mode + tools.toml validation.
pub trait Planner: Send + Sync {
    /// Classify the raw input into an `Intent`.
    fn classify(&self, raw: &str) -> Result<Intent>;

    /// Produce an execution plan for the given intent.
    /// `on_token` is called for each streamed token from the LLM.
    fn plan(
        &self,
        intent: &Intent,
        ctx: &PlannerCtx,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<Plan>;
}
