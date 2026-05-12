/// actors/provider_router.rs — ProviderRouter actor (Blueprint §4, §8).
///
/// Routes `LlmProvider` calls based on `TaskClass`. Encapsulates the
/// provider-selection logic currently spread across `model_router.rs` and
/// individual callers.
///
/// Phase 1: Thin wrapper around `model_router::route_task` that instantiates
/// the correct concrete provider and calls `complete_stream`. This consolidates
/// the concrete-type instantiation into ONE place, so actors always depend on
/// `Box<dyn LlmProvider>` everywhere else.
///
/// Phase 2: Add cost-tracking, circuit-breaker logic (fall through the chain
/// if the primary is rate-limited), and latency telemetry per provider.

use anyhow::Result;

use crate::llm::{CompletionRequest, CompletionResponse, LlmProvider};
use crate::model_router::{route_task, TaskClass};
use crate::secrets;

/// The single provider router — all LLM calls should go through here.
pub struct ProviderRouter;

impl ProviderRouter {
    pub fn new() -> Self {
        Self
    }

    /// Run a streaming completion for the given task class.
    ///
    /// Walks the provider chain from `route_task()` and returns the first
    /// successful response. `on_token` is called for each streamed delta.
    pub fn complete_stream(
        &self,
        task: TaskClass,
        req: &CompletionRequest,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<CompletionResponse> {
        let route = route_task(task);
        let mut last_err: Option<anyhow::Error> = None;

        for provider_name in &route.provider_chain {
            let Some(key) = secrets::read_provider_key(provider_name)? else {
                continue;
            };
            let provider: Box<dyn LlmProvider> = match *provider_name {
                "groq" => Box::new(crate::llm::GroqProvider),
                "gemini" => Box::new(crate::llm::GeminiProvider),
                "openrouter" => Box::new(crate::llm::OpenRouterProvider),
                "anthropic" => Box::new(crate::llm_anthropic::AnthropicProvider),
                _ => continue,
            };
            match provider.complete_stream(req, &key, on_token) {
                Ok(resp) => {
                    tracing::debug!(
                        target: "neph_router",
                        provider = provider_name,
                        model = %route.model,
                        task = ?task,
                        "provider_router: completed"
                    );
                    return Ok(resp);
                }
                Err(e) => {
                    tracing::warn!(
                        target: "neph_router",
                        provider = provider_name,
                        error = %e,
                        "provider_router: provider failed, trying next"
                    );
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("no provider available for {:?}", task)))
    }
}

impl Default for ProviderRouter {
    fn default() -> Self {
        Self::new()
    }
}
