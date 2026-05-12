/// domains/shell.rs — Shell execution domain (Blueprint §5).
///
/// Three sandbox tiers mandate different privilege levels:
///
/// | Tier       | Use                         | Implementation                  |
/// |------------|-----------------------------|---------------------------------|
/// | `safe`     | text utils, git read        | in-process Rust only            |
/// | `sandboxed`| code exec, builds, scripts  | Windows Job Object + Restricted |
/// | `native`   | system commands             | red-tier, manual + full audit   |
///
/// The LLM sees only a tier handle. Mapping tier → reality happens here.
///
/// Rule: the `native` tier always returns an error if called without a
/// one-shot confirmation token (enforced by ExecutorActor, not here).

use anyhow::{bail, Result};

use crate::traits::domain::{Capability, DomainId, ExecutionDomain};
use crate::traits::tool::PlannedAction;

// ── Tier enum ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellTier {
    /// In-process Rust only. No subprocess spawn allowed.
    Safe,
    /// Subprocess with Windows Job Object + restricted token (Phase 3).
    Sandboxed,
    /// Full native shell — red-tier, requires explicit confirmation token.
    Native,
}

// ── ShellDomain ───────────────────────────────────────────────────────────────

pub struct ShellDomain {
    id: DomainId,
    tier: ShellTier,
}

impl ShellDomain {
    pub fn safe() -> Self {
        Self { id: DomainId::new("shell-safe"), tier: ShellTier::Safe }
    }

    pub fn sandboxed() -> Self {
        Self { id: DomainId::new("shell-sandboxed"), tier: ShellTier::Sandboxed }
    }

    pub fn native() -> Self {
        Self { id: DomainId::new("shell-native"), tier: ShellTier::Native }
    }

    pub fn tier(&self) -> &ShellTier {
        &self.tier
    }
}

impl ExecutionDomain for ShellDomain {
    fn id(&self) -> &DomainId {
        &self.id
    }

    fn allowed_caps(&self) -> &[Capability] {
        match self.tier {
            ShellTier::Safe => &[Capability::LocalCompute],
            ShellTier::Sandboxed => &[Capability::LocalCompute, Capability::SubProcess],
            ShellTier::Native => &[Capability::LocalCompute, Capability::SubProcess, Capability::System],
        }
    }

    fn enforce(&self, action: &PlannedAction) -> Result<()> {
        match self.tier {
            ShellTier::Safe => {
                // Only allow safe Rust-internal operations.
                // Any tool that spawns a subprocess should use Sandboxed/Native.
                if action.args.get("command").is_some() {
                    bail!(
                        "shell.safe domain: tool '{}' attempted subprocess spawn — \
                         use shell.sandboxed or shell.native domain instead",
                        action.tool
                    );
                }
                Ok(())
            }
            ShellTier::Sandboxed => {
                // Phase 3: will validate Job Object attachment before proceeding.
                // Phase 1/2: permit with a warning logged.
                tracing::warn!(
                    target: "neph_domain",
                    tool = %action.tool,
                    "shell.sandboxed: Job Object isolation not yet implemented (Phase 3)"
                );
                Ok(())
            }
            ShellTier::Native => {
                // Native tier requires a confirmation token to have been validated
                // by the ExecutorActor BEFORE `enforce()` is called.
                // If we reach here without confirmation, that's a programmer error,
                // not a user error — fail loud.
                tracing::error!(
                    target: "neph_domain",
                    tool = %action.tool,
                    "shell.native domain enforce() called — \
                     ExecutorActor must validate confirmationToken before invoking native shell"
                );
                // Allow — the token check is in executor.rs, not here.
                // This domain records that native shell was accessed.
                Ok(())
            }
        }
    }
}

// ── Resolver ─────────────────────────────────────────────────────────────────

/// Map a domain string from `tools.toml` to a `ShellDomain`.
///
/// Domain strings used in tools.toml:
///   "shell-safe"      → `ShellDomain::safe()`
///   "shell-sandboxed" → `ShellDomain::sandboxed()`
///   "shell-native"    → `ShellDomain::native()`
pub fn resolve_shell_domain(domain: &str) -> Option<ShellDomain> {
    match domain {
        "shell-safe" => Some(ShellDomain::safe()),
        "shell-sandboxed" => Some(ShellDomain::sandboxed()),
        "shell-native" => Some(ShellDomain::native()),
        _ => None,
    }
}
