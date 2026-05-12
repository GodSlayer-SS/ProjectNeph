/// providers/anthropic.rs — Anthropic Claude Sonnet 4.5 (Blueprint §3, Phase 1).
///
/// Wraps `llm_anthropic::AnthropicProvider` implementing `traits::llm::LlmProvider`.

use anyhow::Result;

use crate::traits::llm::{ChatRequest, ChatResponse, LlmProvider, ProviderCapabilities};

pub struct AnthropicProviderV2;

impl LlmProvider for AnthropicProviderV2 {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: true,
            json_mode: false, // Anthropic uses prompt-level JSON, not a mode flag
            vision: true,
            tool_calls: true,
            max_context_tokens: 200_000,
        }
    }

    fn complete(&self, req: &ChatRequest, api_key: &str) -> Result<ChatResponse> {
        let compat = crate::llm_anthropic::AnthropicProvider;
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
        let compat = crate::llm_anthropic::AnthropicProvider;
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
