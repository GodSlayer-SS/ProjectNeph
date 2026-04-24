#![allow(dead_code)]

use std::time::{Duration, Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::RiskLevel;
use crate::tools::{tool_risk, validate_tool_schema};

const MAX_STEPS: usize = 5;
const MAX_WALL_CLOCK: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStepProposal {
    pub thought: String,
    pub tool: String,
    pub args: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStepRecord {
    pub proposal: AgentStepProposal,
    pub risk: String,
    pub allowed: bool,
    pub observation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunResult {
    pub steps: Vec<AgentStepRecord>,
    pub timed_out: bool,
}

pub fn run_bounded_agent(proposals: &[AgentStepProposal]) -> Result<AgentRunResult> {
    let start = Instant::now();
    let mut records = Vec::new();
    for proposal in proposals.iter().take(MAX_STEPS) {
        if start.elapsed() > MAX_WALL_CLOCK {
            return Ok(AgentRunResult {
                steps: records,
                timed_out: true,
            });
        }
        validate_tool_schema(&proposal.tool, &proposal.args)?;
        let risk = tool_risk(&proposal.tool);
        let allowed = matches!(risk, RiskLevel::Green);
        records.push(AgentStepRecord {
            proposal: proposal.clone(),
            risk: risk.as_str().to_string(),
            allowed,
            observation: if allowed {
                "validated".into()
            } else {
                "requires confirmation token".into()
            },
        });
    }
    Ok(AgentRunResult {
        steps: records,
        timed_out: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_non_green_from_auto_execution() {
        let proposals = vec![AgentStepProposal {
            thought: "delete old file".into(),
            tool: "delete_file".into(),
            args: serde_json::json!({"path":"C:/tmp/a.txt"}),
        }];
        let result = run_bounded_agent(&proposals).unwrap();
        assert_eq!(result.steps.len(), 1);
        assert!(!result.steps[0].allowed);
    }
}
