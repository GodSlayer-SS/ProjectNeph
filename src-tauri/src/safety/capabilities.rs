/// safety/capabilities.rs — Capability token enforcement (Blueprint §4, §5, §10).
///
/// Blueprint §10: "Refactor risk levels → capability tokens (additive,
/// don't break risk levels)."
///
/// A capability token is an additive grant that allows a specific action class
/// within a session. Without the token, the Executor rejects the action.
///
/// Current capability tokens mirror the execution domain tiers:
///   - `green`  : Read-only, no network, no mutations — no token needed
///   - `yellow` : Mutations, limited network — requires `YellowCap` token
///   - `red`    : Full access (personal profile, system commands) — requires `RedCap` + explicit
///
/// Phase 1: Tokens are implicit (risk level from tools.toml).
/// Phase 2: Explicit capability token issuance and UI prompts per domain action.

use crate::models::RiskLevel;

/// The capability required to perform an action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityToken {
    /// No token needed — green-tier action.
    Green,
    /// Yellow token — user has confirmed a yellow-tier action in this session.
    Yellow,
    /// Red token — user has explicitly unlocked a red-tier action.
    Red,
}

impl CapabilityToken {
    /// Derive the minimum required capability from a tool risk level.
    pub fn required_for(risk: &RiskLevel) -> Self {
        match risk {
            RiskLevel::Green => CapabilityToken::Green,
            RiskLevel::Yellow => CapabilityToken::Yellow,
            RiskLevel::Red => CapabilityToken::Red,
        }
    }

    /// Is this capability sufficient to perform an action requiring `required`?
    pub fn satisfies(&self, required: &CapabilityToken) -> bool {
        match (self, required) {
            (_, CapabilityToken::Green) => true,
            (CapabilityToken::Red, _) => true,
            (CapabilityToken::Yellow, CapabilityToken::Yellow) => true,
            _ => false,
        }
    }
}

/// Session capability context — what tokens the user has granted in this session.
#[derive(Debug, Default, Clone)]
pub struct SessionCapabilities {
    /// Maximum capability tier granted in this session.
    pub max_tier: Option<CapabilityToken>,
}

impl SessionCapabilities {
    pub fn new() -> Self {
        Self::default()
    }

    /// Grant a capability for this session (additive — can only escalate, not demote).
    pub fn grant(&mut self, cap: CapabilityToken) {
        match (&self.max_tier, &cap) {
            (None, _) => self.max_tier = Some(cap),
            (Some(CapabilityToken::Green), CapabilityToken::Yellow)
            | (Some(CapabilityToken::Green), CapabilityToken::Red)
            | (Some(CapabilityToken::Yellow), CapabilityToken::Red) => {
                self.max_tier = Some(cap);
            }
            _ => {} // already at or above this level
        }
    }

    /// Check whether this session can perform an action requiring `cap`.
    pub fn can_perform(&self, cap: &CapabilityToken) -> bool {
        match (&self.max_tier, cap) {
            (_, CapabilityToken::Green) => true,
            (Some(granted), required) => granted.satisfies(required),
            (None, _) => false,
        }
    }
}
