/// providers/openrouter.rs — OpenRouter multi-model fallback (Blueprint §3, Phase 1).
///
/// Wraps `llm::OpenRouterProvider` implementing `traits::llm::LlmProvider`.

use anyhow::Result;

use crate::traits::llm::{ChatRequest, ChatResponse, LlmProvider, ProviderCapabilities};

pub struct OpenRouterProviderV2;

impl LlmProvider for OpenRouterProviderV2 {
    fn name(&self) -> &str {
        "openrouter"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: true,
            json_mode: true,
            vision: true,  // model-dependent
            tool_calls: true,
            max_context_tokens: 128_000,
        }
    }

    fn complete(&self, req: &ChatRequest, api_key: &str) -> Result<ChatResponse> {
        let compat = crate::llm::OpenRouterProvider;
        let legacy_req = crate::llm::CompletionRequest::from_chat_request(req);
        let resp = crate::llm::LlmProvider::complete(&compat, &legacy_req, api_key)?;
        Ok(ChatResponse {
            content: resp.text,
            input_tokens: resp.estimated_input_tokens,
            output_tokens: resp.estimated_output_tokens,
            model_used: req.model.clone(),
        })
    }

    fn complete_stream(
        &self,
        req: &ChatRequest,
        api_key: &str,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<ChatResponse> {
        let compat = crate::llm::OpenRouterProvider;
        let legacy_req = crate::llm::CompletionRequest::from_chat_request(req);
        let resp = crate::llm::LlmProvider::complete_stream(&compat, &legacy_req, api_key, on_token)?;
        Ok(ChatResponse {
            content: resp.text,
            input_tokens: resp.estimated_input_tokens,
            output_tokens: resp.estimated_output_tokens,
            model_used: req.model.clone(),
        })
    }
}
