/// PlannerActor — Phase 1.
///
/// Phase 1 behaviour:
///   - Receives a text input (from palette or STT final transcript)
///   - Delegates to `AppState::run_palette_command` which handles:
///       classify → trust-gate → tool dispatch → LLM streaming
///   - Streams reply tokens via `llm:token` Tauri events
///   - Returns the `PaletteRunResponse` for the caller to forward
///
/// The `Planner` trait impl (`SimplePlanner`) is a Phase 2 concern.
/// In Phase 1 we expose only the `run_plan` free function which is called
/// from both the palette Tauri command and the `stt:final` listener.

use anyhow::Result;
use tauri::{AppHandle, Emitter};

use crate::state::AppState;
use crate::tools::manifest::Manifest;
use crate::tools::schema::{stable_json, validate_tool_schema};
use crate::traits::planner::{Intent, Plan, Planner, PlannerCtx, PlannedStep};
use crate::{llm, model_router::TaskClass, secrets};

// ── PlannerActor (Tauri command wrapper) ──────────────────────────────────────

/// Wire an input (typed or voiced) through the full run_palette_command pipeline.
///
/// Streams tokens via `llm:token`.  When done, the caller emits `llm:done`.
/// Called from:
///   1. The `stt:final` listener in `lib.rs` (voice path)
///   2. The `run_palette_command` Tauri command (keyboard path) — indirectly
pub fn run_plan(
    app: &AppHandle,
    state: &AppState,
    input: &str,
    confirmation_token: Option<String>,
) -> anyhow::Result<crate::models::PaletteRunResponse> {
    let mut token_emitter = |chunk: &str| {
        let _ = app.emit("llm:token", chunk);
    };
    let response = state
        .run_palette_command(input, confirmation_token, Some(&mut token_emitter))
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(response)
}

// ── Phase 2: Structured planner (minimal) ────────────────────────────────────

/// Minimal `Planner` implementation that turns explicit commands into 1-step Plans.
///
/// This is the Phase 2 bridge: it produces a typed `Plan` with steps that reference
/// `tools.toml` tool names, and validates args against the manifest schema.
///
/// Notes:
/// - This does **not** yet use an LLM for multi-step planning (ExecutorActor wires next).
/// - Natural-language (no prefix) requests fall back to `Plan::chat_only`.
pub struct StructuredPlanner;

impl StructuredPlanner {
    pub fn new() -> Self {
        Self
    }

    fn plan_hash(steps: &[PlannedStep]) -> String {
        use sha2::{Digest, Sha256};
        // Phase 2 bridge: for now we only support 1-step plans, and we align the
        // hash with the existing trust kernel: SHA-256 over stable {tool,args}.
        let first = steps.first();
        let v = if let Some(s) = first {
            serde_json::json!({ "tool": s.tool, "args": stable_json(&s.args) })
        } else {
            serde_json::json!({ "tool": "chat", "args": {} })
        };
        let bytes = serde_json::to_vec(&v).unwrap_or_default();
        let mut h = Sha256::new();
        h.update(bytes);
        format!("{:x}", h.finalize())
    }
}

impl Default for StructuredPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Planner for StructuredPlanner {
    fn classify(&self, raw: &str) -> Result<Intent> {
        let routed = crate::router::route_input(raw);
        if routed.intent != "unknown" {
            return Ok(Intent {
                label: routed.intent,
                raw_input: raw.to_string(),
                confidence: 1.0,
            });
        }
        Ok(Intent { label: "nl_plan".into(), raw_input: raw.to_string(), confidence: 0.5 })
    }

    fn plan(
        &self,
        intent: &Intent,
        ctx: &PlannerCtx,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<Plan> {
        if intent.label == "nl_plan" {
            return plan_with_llm(&intent.raw_input, ctx, on_token);
        }

        if !ctx.available_tools.iter().any(|t| t == &intent.label) {
            return Ok(Plan::chat_only(format!(
                "Tool '{}' not available in this context.",
                intent.label
            )));
        }

        // Resolve tool from manifest (preferred) to get domain + schema.
        let manifest = Manifest::get();
        let Some(entry) = manifest.tool(&intent.label) else {
            // If manifest isn't loaded, fall back to chat-only rather than planning unsafe steps.
            return Ok(Plan::chat_only("tools.toml unavailable; cannot plan.".to_string()));
        };

        // Re-route raw input to derive args (explicit prefix path).
        let routed = crate::router::route_input(&intent.raw_input);
        let args = routed.args;

        // Validate shape against manifest schema.
        validate_tool_schema(&intent.label, &args)?;

        let steps = vec![PlannedStep {
            tool: intent.label.clone(),
            args,
            domain: entry.domain.clone(),
        }];

        Ok(Plan {
            plan_hash: Self::plan_hash(&steps),
            steps,
            raw_llm_output: String::new(),
        })
    }
}

#[derive(serde::Deserialize)]
struct LlmPlanOut {
    steps: Vec<PlannedStep>,
}

fn plan_with_llm(raw: &str, ctx: &PlannerCtx, on_token: &mut dyn FnMut(&str)) -> Result<Plan> {
    let manifest = Manifest::get();
    if !manifest.is_loaded() {
        return Ok(Plan::chat_only("tools.toml unavailable; cannot plan.".to_string()));
    }

    let tools = manifest
        .all_tools()
        .into_iter()
        .filter(|t| ctx.available_tools.iter().any(|a| a == &t.name))
        .map(|t| {
            let args = t
                .args
                .iter()
                .map(|(k, v)| format!("{k}:{},required={}", v.arg_type, v.required))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "- name: {}\n  risk: {}\n  domain: {}\n  desc: {}\n  args: {}\n",
                t.name,
                t.risk.as_str(),
                t.domain,
                t.description,
                if args.is_empty() { "(none)".into() } else { args }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let system = format!(
        "You are Neph's planner. Output ONLY strict JSON of shape: {{\"steps\":[{{\"tool\":\"<tools.toml name>\",\"args\":{{}},\"domain\":\"<domain handle>\"}}]}}.\n\
Rules:\n\
- Only use tools listed below.\n\
- Each step must include tool, args, and domain.\n\
- Prefer 1-3 steps.\n\
- If user request is pure chat, output {{\"steps\":[]}}.\n\
\n\
Available tools:\n{tools}\n\
\n\
Context snippets (may be empty):\n{}",
        ctx.memory_snippets.join("\n")
    );

    // Use the existing provider routing stack (Blueprint §8): reasoning tasks → Claude first.
    let route = crate::model_router::route_task(TaskClass::Reasoning);
    let mut last_err: Option<anyhow::Error> = None;

    for provider_name in route.provider_chain {
        let Some(key) = secrets::read_provider_key(provider_name)? else { continue };
        let provider: Box<dyn llm::LlmProvider> = match provider_name {
            "groq" => Box::new(llm::GroqProvider),
            "gemini" => Box::new(llm::GeminiProvider),
            "openrouter" => Box::new(llm::OpenRouterProvider),
            "anthropic" => Box::new(crate::llm_anthropic::AnthropicProvider),
            _ => continue,
        };

        let mut user = raw.to_string();
        for attempt in 0..2 {
            let req = llm::CompletionRequest {
                system: system.clone(),
                user: user.clone(),
                model: route.model.to_string(),
                temperature: 0.2,
                max_tokens: 900,
                json_mode: true,
            };
            let resp = provider.complete_stream(&req, &key, on_token);
            match resp {
                Ok(r) => {
                    if let Ok(parsed) = serde_json::from_str::<LlmPlanOut>(&r.text) {
                        // Validate steps against schema.
                        for s in &parsed.steps {
                            validate_tool_schema(&s.tool, &s.args)?;
                        }
                        let plan_hash = {
                            use sha2::{Digest, Sha256};
                            let stable = stable_json(&serde_json::json!({ "steps": parsed.steps }));
                            let bytes = serde_json::to_vec(&stable).unwrap_or_default();
                            let mut h = Sha256::new();
                            h.update(bytes);
                            format!("{:x}", h.finalize())
                        };
                        return Ok(Plan { plan_hash, steps: parsed.steps, raw_llm_output: r.text });
                    }
                    if attempt == 0 {
                        user = format!("{raw}\nYour previous output was invalid JSON. Return only valid JSON.");
                        continue;
                    }
                    return Ok(Plan::chat_only(r.text));
                }
                Err(e) => {
                    last_err = Some(e);
                    break;
                }
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("no provider available for planning")))
}
