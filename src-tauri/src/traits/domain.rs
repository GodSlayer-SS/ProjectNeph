use anyhow::Result;
use serde::{Deserialize, Serialize};

// ── Domain identifiers ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DomainId(pub String);

impl DomainId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

// ── Capabilities ──────────────────────────────────────────────────────────────

/// Fine-grained capability tokens. Additive: a domain grants a set of these.
/// The LLM sees only a domain handle; Rust maps handle → `[Capability]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Read files / pages in this domain.
    Read,
    /// Write / create files in this domain.
    Write,
    /// Delete files / entries in this domain (always Yellow+).
    Delete,
    /// Execute shell commands in this domain's sandbox tier.
    Execute,
    /// Make network requests to the domain's egress allowlist.
    Network,
    /// Interact with browser profile (click, fill, navigate).
    BrowserInteract,
    /// In-process local computation (shell-safe tier).
    LocalCompute,
    /// Spawn a subprocess (shell-sandboxed tier, Phase 3: Job Object).
    SubProcess,
    /// Full system/native access (shell-native tier, always Red).
    System,
}

// ── The Trait ─────────────────────────────────────────────────────────────────

/// An execution domain enforces capability policy for a class of resources.
///
/// Filesystem domains: workspace | projects | personal | system | temp
/// Browser domains:    research | tools | personal | throwaway
/// Shell domains:      safe | sandboxed | native  (Phase 3)
/// Network:            per-tool allowlist
pub trait ExecutionDomain: Send + Sync {
    fn id(&self) -> &DomainId;

    /// The capabilities this domain grants.
    fn allowed_caps(&self) -> &[Capability];

    /// Returns `Ok(())` if the action is permitted, or an error describing the
    /// policy violation. Called by `ExecutorActor` before every tool execution.
    fn enforce(&self, action: &crate::traits::tool::PlannedAction) -> Result<()>;
}
