use anyhow::{bail, Result};

use crate::traits::domain::{Capability, DomainId, ExecutionDomain};
use crate::traits::tool::PlannedAction;

pub struct BrowserDomain {
    id: DomainId,
    caps: Vec<Capability>,
    profile: &'static str,
}

impl BrowserDomain {
    pub fn research() -> Self {
        Self {
            id: DomainId::new("browser-research"),
            caps: vec![Capability::Read, Capability::BrowserInteract, Capability::Network],
            profile: "nephis-research",
        }
    }

    pub fn tools() -> Self {
        Self {
            id: DomainId::new("browser-tools"),
            caps: vec![Capability::Read, Capability::BrowserInteract, Capability::Network],
            profile: "nephis-tools",
        }
    }

    pub fn personal() -> Self {
        Self {
            id: DomainId::new("browser-personal"),
            caps: vec![Capability::Read, Capability::BrowserInteract, Capability::Network],
            profile: "nephis-personal",
        }
    }

    pub fn throwaway() -> Self {
        Self {
            id: DomainId::new("browser-throwaway"),
            caps: vec![Capability::Read, Capability::BrowserInteract, Capability::Network],
            profile: "nephis-throwaway",
        }
    }
}

impl ExecutionDomain for BrowserDomain {
    fn id(&self) -> &DomainId {
        &self.id
    }

    fn allowed_caps(&self) -> &[Capability] {
        &self.caps
    }

    fn enforce(&self, action: &PlannedAction) -> Result<()> {
        // Ensure action profile matches domain profile (no cross-profile escalation).
        if let Some(profile) = action.args.get("profile").and_then(|v| v.as_str()) {
            if profile != self.profile {
                bail!("profile '{}' not allowed in domain '{}'", profile, self.id.0);
            }
        }
        // Personal profile actions must be explicit (blueprint red-tier directive).
        if self.profile == "nephis-personal" {
            let explicit = action
                .args
                .get("explicit_personal")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !explicit {
                bail!("personal browser domain requires explicit_personal=true");
            }
        }
        Ok(())
    }
}

pub fn resolve_browser_domain(handle: &str) -> Option<BrowserDomain> {
    match handle {
        "browser-research" => Some(BrowserDomain::research()),
        "browser-tools" => Some(BrowserDomain::tools()),
        "browser-personal" => Some(BrowserDomain::personal()),
        "browser-throwaway" => Some(BrowserDomain::throwaway()),
        _ => None,
    }
}

