use anyhow::Result;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::models::{IntentProvenance, RiskLevel};
use crate::router::RoutedIntent;
use crate::redaction::redact_json_value;
use crate::tools::{self, tool_risk};

#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub tool: String,
    pub args: Value,
    pub risk: RiskLevel,
    pub provenance: IntentProvenance,
}

impl ExecutionPlan {
    pub fn from_routed(routed: &RoutedIntent) -> Result<Self> {
        let tool = routed.intent.clone();
        tools::validate_tool_schema(&tool, &routed.args)?;
        let risk = tool_risk(&tool);
        Ok(Self {
            tool,
            args: routed.args.clone(),
            risk,
            provenance: routed.source.clone(),
        })
    }

    pub fn stable_args_json(&self) -> Value {
        tools::stable_json(&self.args)
    }

    pub fn plan_hash(&self) -> String {
        let payload = serde_json::json!({
            "tool": self.tool,
            "args": self.stable_args_json(),
        });
        let canon = payload.to_string();
        let digest = Sha256::digest(canon.as_bytes());
        hex::encode(digest)
    }

    pub fn redacted_args_json(&self) -> Value {
        tools::redact_args_for_tool(&self.tool, &self.args)
    }

    pub fn lineage_value(&self, user_input: &str, llm_raw: Option<&Value>) -> Value {
        let base = serde_json::json!({
            "user_input": user_input,
            "provenance": self.provenance,
            "tool": self.tool,
            "args_redacted": self.redacted_args_json(),
        });
        if let Some(raw) = llm_raw {
            serde_json::json!({
                "user_input": user_input,
                "provenance": self.provenance,
                "llm_route_redacted": redact_json_value(raw),
                "tool": self.tool,
                "args_redacted": self.redacted_args_json(),
            })
        } else {
            base
        }
    }
}

pub fn privileged_mutation_risk(risk: &RiskLevel) -> bool {
    matches!(risk, RiskLevel::Yellow | RiskLevel::Red)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::IntentProvenance;
    use crate::router::RoutedIntent;
    use serde_json::json;

    #[test]
    fn plan_hash_stable_for_key_order() {
        let a = RoutedIntent {
            intent: "move_file".into(),
            args: json!({"to": "b", "from": "a"}),
            confidence: 1.0,
            source: IntentProvenance::UserPrefix,
            llm_payload: None,
        };
        let b = RoutedIntent {
            intent: "move_file".into(),
            args: json!({"from": "a", "to": "b"}),
            confidence: 1.0,
            source: IntentProvenance::UserPrefix,
            llm_payload: None,
        };
        let pa = ExecutionPlan::from_routed(&a).unwrap();
        let pb = ExecutionPlan::from_routed(&b).unwrap();
        assert_eq!(pa.plan_hash(), pb.plan_hash());
    }
}
