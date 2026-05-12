/// Network execution domain — Phase 1.
///
/// Extends `network_allowlist.rs` with per-tool egress checking.
/// In Phase 1 the allowlist is still global (loaded from the tool registry).
/// In Phase 2 it will read per-tool egress arrays from `tools.toml`.

use anyhow::{bail, Result};

use crate::traits::domain::{Capability, DomainId, ExecutionDomain};
use crate::traits::tool::PlannedAction;

pub struct NetworkDomain {
    id: DomainId,
    /// Allowlisted hostnames for this domain instance.
    allowed_hosts: Vec<String>,
}

impl NetworkDomain {
    pub fn new(allowed_hosts: Vec<String>) -> Self {
        Self {
            id: DomainId::new("network"),
            allowed_hosts,
        }
    }

    /// Build a domain from the global allowlist (Phase 1 fallback).
    pub fn global() -> Self {
        Self::new(vec![
            "api.groq.com".into(),
            "generativelanguage.googleapis.com".into(),
            "openrouter.ai".into(),
            "api.anthropic.com".into(),
            "api.openai.com".into(),
            "duckduckgo.com".into(),
            "html.duckduckgo.com".into(),
            "github.com".into(),
            "docs.rs".into(),
            "stackoverflow.com".into(),
        ])
    }

    pub fn host_allowed(&self, url: &str) -> bool {
        let Ok(parsed) = url::Url::parse(url) else { return false };
        if !matches!(parsed.scheme(), "http" | "https") {
            return false;
        }
        let Some(host) = parsed.host_str() else { return false };
        self.allowed_hosts
            .iter()
            .any(|a| host == a.as_str() || host.ends_with(&format!(".{a}")))
    }
}

impl ExecutionDomain for NetworkDomain {
    fn id(&self) -> &DomainId {
        &self.id
    }

    fn allowed_caps(&self) -> &[Capability] {
        &[Capability::Network]
    }

    fn enforce(&self, action: &PlannedAction) -> Result<()> {
        if let Some(url) = action.args.get("url").and_then(|v| v.as_str()) {
            if !self.host_allowed(url) {
                bail!("network egress to '{}' not allowed for tool '{}'", url, action.tool);
            }
        }
        Ok(())
    }
}
