/// providers/groq.rs — Groq Llama 3.3 70B / 8B provider (Blueprint §3, Phase 1).
///
/// Wraps `llm::GroqProvider` implementing `traits::llm::LlmProvider`.

use anyhow::Result;

use crate::traits::llm::{ChatRequest, ChatResponse, LlmProvider, ProviderCapabilities};

pub struct GroqProviderV2;

impl LlmProvider for GroqProviderV2 {
    fn name(&self) -> &str {
        "groq"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: true,
            json_mode: true,
            vision: false,
            tool_calls: true,
            max_context_tokens: 128_000,
        }
    }

    fn complete(&self, req: &ChatRequest, api_key: &str) -> Result<ChatResponse> {
        let compat = crate::llm::GroqProvider;
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
        let compat = crate::llm::GroqProvider;
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
