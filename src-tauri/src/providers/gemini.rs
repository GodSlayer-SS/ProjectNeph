/// providers/gemini.rs — Gemini 2.5 Flash/Pro provider (Blueprint §3, Phase 1).
///
/// Wraps the existing `llm::GeminiProvider` and implements `traits::llm::LlmProvider`.
/// Actors MUST NOT import `llm::GeminiProvider` directly — use this wrapper.

use anyhow::Result;

use crate::traits::llm::{ChatRequest, ChatResponse, LlmProvider, ProviderCapabilities};

/// Google Gemini provider (Flash primary, Pro fallback).
/// Implements `traits::llm::LlmProvider` — the stable interface.
pub struct GeminiProviderV2;

impl LlmProvider for GeminiProviderV2 {
    fn name(&self) -> &str {
        "gemini"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: true,
            json_mode: true,
            vision: true,
            tool_calls: true,
            max_context_tokens: 1_000_000,
        }
    }

    fn complete(&self, req: &ChatRequest, api_key: &str) -> Result<ChatResponse> {
        // Delegate to the legacy llm::GeminiProvider via compat shim.
        let compat = crate::llm::GeminiProvider;
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
        let compat = crate::llm::GeminiProvider;
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
