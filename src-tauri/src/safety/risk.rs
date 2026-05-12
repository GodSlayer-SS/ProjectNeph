/// safety/risk.rs — Risk level classification (Blueprint §4 safety/ group).
///
/// Extracted from `models.rs` and re-exported here under `safety::risk`.
/// The `RiskLevel` enum remains in `models.rs` for backward compat with
/// the many callers using `crate::models::RiskLevel`.
///
/// New code should prefer `crate::safety::risk::RiskLevel`.
/// Existing callers using `crate::models::RiskLevel` continue to compile
/// unchanged — this is a non-breaking additive re-export.
///
/// Blueprint §4: `safety/risk.rs` is listed alongside `confirmation.rs`,
/// `path_policy.rs`, and `capabilities.rs` in the trust kernel group.
///
/// Blueprint §10: "LLM cannot start yellow/red without confirmation.
///                 Sacred rule."

/// Re-export `RiskLevel` from `models` under the `safety::risk` namespace.
pub use crate::models::RiskLevel;

/// Classify a plain-text risk string (from tools.toml or LLM output) into
/// a `RiskLevel`. Unrecognized strings default to `Green` (safe default).
pub fn classify(s: &str) -> RiskLevel {
    match s.trim().to_lowercase().as_str() {
        "red" => RiskLevel::Red,
        "yellow" => RiskLevel::Yellow,
        _ => RiskLevel::Green,
    }
}

/// Return the minimum `RiskLevel` given two levels.
/// (the less-privileged of the two — useful for domain intersection)
pub fn min_risk(a: &RiskLevel, b: &RiskLevel) -> RiskLevel {
    match (a, b) {
        (RiskLevel::Green, _) | (_, RiskLevel::Green) => RiskLevel::Green,
        (RiskLevel::Yellow, _) | (_, RiskLevel::Yellow) => RiskLevel::Yellow,
        _ => RiskLevel::Red,
    }
}

/// Return the maximum `RiskLevel` given two levels.
/// (the more-privileged of the two — used by the executor for compound plans)
pub fn max_risk(a: &RiskLevel, b: &RiskLevel) -> RiskLevel {
    match (a, b) {
        (RiskLevel::Red, _) | (_, RiskLevel::Red) => RiskLevel::Red,
        (RiskLevel::Yellow, _) | (_, RiskLevel::Yellow) => RiskLevel::Yellow,
        _ => RiskLevel::Green,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_works() {
        assert!(matches!(classify("red"), RiskLevel::Red));
        assert!(matches!(classify("Yellow"), RiskLevel::Yellow));
        assert!(matches!(classify("unknown"), RiskLevel::Green));
        assert!(matches!(classify(""), RiskLevel::Green));
    }

    #[test]
    fn max_risk_works() {
        assert!(matches!(
            max_risk(&RiskLevel::Green, &RiskLevel::Yellow),
            RiskLevel::Yellow
        ));
        assert!(matches!(
            max_risk(&RiskLevel::Yellow, &RiskLevel::Red),
            RiskLevel::Red
        ));
    }
}
