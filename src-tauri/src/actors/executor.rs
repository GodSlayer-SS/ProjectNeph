use crate::domains::{browser, filesystem, network};
use crate::execution::{privileged_mutation_risk, ExecutionPlan};
use crate::models::{IntentProvenance, PaletteRunResponse};
use crate::router::RoutedIntent;
use crate::state::AppState;
use crate::state::runner::execute_plan;
use crate::tools;
use crate::tools::manifest::Manifest;
use crate::traits::domain::ExecutionDomain;
use crate::traits::planner::Plan;
use crate::traits::tool::PlannedAction;

/// Execute a typed `Plan` (Phase 2).
///
/// This is a minimal bridge that supports:
/// - single-step plans
/// - manifest schema validation (`tools.toml`)
/// - domain enforcement (filesystem + network in Phase 1/2)
/// - existing trust-kernel confirmationToken/planHash/TTL gating
///
/// Multi-step plans and richer progress events land as this actor evolves.
pub fn execute_structured_plan(
    state: &AppState,
    user_input: &str,
    plan: &Plan,
    confirmation_token: Option<&str>,
    on_token: Option<&mut dyn FnMut(&str)>,
) -> Result<PaletteRunResponse, String> {
    if plan.steps.is_empty() {
        return Err("empty plan".to_string());
    }

    let manifest = Manifest::get();

    // Validate and compute max risk.
    let mut max_risk = crate::models::RiskLevel::Green;
    let mut preview_lines: Vec<String> = Vec::new();

    for (i, step) in plan.steps.iter().enumerate() {
        let entry = manifest
            .tool(&step.tool)
            .ok_or_else(|| format!("Unknown tool '{}' (not in tools.toml)", step.tool))?;

        tools::validate_tool_schema(&step.tool, &step.args).map_err(|e| e.to_string())?;

        let action = PlannedAction {
            tool: step.tool.clone(),
            args: step.args.clone(),
            domain_handle: entry.domain.clone(),
        };
        if let Some(fs_dom) = filesystem::resolve_filesystem_domain(entry.domain.as_str()) {
            fs_dom.enforce(&action).map_err(|e| e.to_string())?;
        }
        if let Some(br_dom) = browser::resolve_browser_domain(entry.domain.as_str()) {
            br_dom.enforce(&action).map_err(|e| e.to_string())?;
        }
        if !entry.egress.is_empty() {
            let net = network::NetworkDomain::new(entry.egress.clone());
            net.enforce(&action).map_err(|e| e.to_string())?;
        }

        let risk = tools::tool_risk(&step.tool);
        if matches!(risk, crate::models::RiskLevel::Red) {
            max_risk = crate::models::RiskLevel::Red;
        } else if matches!(risk, crate::models::RiskLevel::Yellow) && !matches!(max_risk, crate::models::RiskLevel::Red) {
            max_risk = crate::models::RiskLevel::Yellow;
        }

        let preview = tools::dry_run_preview(&step.tool, &step.args).unwrap_or_else(|_| step.tool.clone());
        preview_lines.push(format!("{}. {} — {}", i + 1, step.tool, preview));
    }

    let plan_hash = plan.plan_hash.clone();
    let needs_confirmation = privileged_mutation_risk(&max_risk);

    if needs_confirmation && confirmation_token.is_none() {
        let token = state
            .confirmations
            .lock()
            .map_err(|_| "confirmation store poisoned".to_string())?
            .issue(plan_hash.clone());
        return Ok(PaletteRunResponse::NeedConfirmation {
            plan_hash,
            preview: preview_lines.join("\n"),
            risk: max_risk.as_str().to_string(),
            token,
        });
    }

    if needs_confirmation {
        let tok = confirmation_token.ok_or_else(|| "missing confirmation token".to_string())?;
        state
            .confirmations
            .lock()
            .map_err(|_| "confirmation store poisoned".to_string())?
            .consume(tok, &plan_hash)
            .map_err(|e| e.to_string())?;
    }

    // Execute each step via existing executor path.
    let mut last_output: Option<String> = None;
    for step in &plan.steps {
        let routed = RoutedIntent {
            intent: step.tool.clone(),
            args: step.args.clone(),
            confidence: 1.0,
            source: IntentProvenance::UserPrefix,
            llm_payload: None,
        };
        let exec_plan = ExecutionPlan::from_routed(&routed).map_err(|e| e.to_string())?;
        let res = execute_plan(state, user_input, &routed, &exec_plan, on_token.as_deref_mut());
        match res? {
            PaletteRunResponse::Completed { output } => last_output = Some(output),
            PaletteRunResponse::NeedConfirmation { .. } => {
                // Should not happen because we consumed the token at plan level.
                return Err("unexpected confirmation request during plan execution".into());
            }
            PaletteRunResponse::Rejected { message } => return Ok(PaletteRunResponse::Rejected { message }),
        }
    }
    Ok(PaletteRunResponse::Completed {
        output: last_output.unwrap_or_else(|| "Plan executed.".into()),
    })
}

