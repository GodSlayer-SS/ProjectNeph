/// safety/ — Trust kernel components (Blueprint §4, §10).
///
/// Groups the sacred trust kernel pieces per Blueprint's folder mandate:
///   risk.rs          — RiskLevel enum + classify/min/max helpers
///   confirmation.rs  — planHash + token + 60s TTL
///   path_policy.rs   — filesystem path allowlist
///   capabilities.rs  — capability token enforcement (additive, yellow/red)
///
/// Blueprint §10: "LLM cannot start yellow/red without confirmation. Sacred rule."
///
/// ⚠️ These modules are NOT moved — they stay as flat modules for backward compat
///    with the many callers across the codebase. This module re-exports them under
///    the `safety::` namespace so new code can use `crate::safety::confirmation`
///    while existing code continues using `crate::confirmation`.

/// Risk level classification — Green / Yellow / Red.
pub mod risk;

/// Capability enforcement — checks whether a planned action is permitted
/// given the current capability tokens in the session.
pub mod capabilities;
