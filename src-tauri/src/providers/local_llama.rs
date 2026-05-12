/// providers/local_llama.rs — Local LLM via llama.cpp/CUDA (Blueprint §3, Phase 2).
///
/// Runs Qwen3 4B Q4_K_M locally via a llama.cpp OpenAI-compatible server.
/// The server is expected to listen on `http://localhost:8080` (configurable).
///
/// Phase 1/2: Stub returns an informative error until the model is downloaded
///            and the server is running. Blueprint §3 says: "Add local Qwen3 4B
///            in Phase 2 ONLY when measured to be necessary."
///
/// Blueprint §13 Mantra: "Cloud-primary cognition. Reflexes only when measured."

use anyhow::{bail, Result};

use crate::traits::llm::{ChatRequest, ChatResponse, LlmProvider, ProviderCapabilities};

/// Local llama.cpp OpenAI-compatible inference server.
pub struct LocalLlamaProvider {
    /// Base URL of the llama.cpp server (default: http://localhost:8080).
    pub base_url: String,
    /// Model name reported in responses.
    pub model_name: String,
}

impl LocalLlamaProvider {
    /// Default Phase 2 configuration: Qwen3 4B at localhost:8080.
    pub fn qwen3_4b() -> Self {
        Self {
            base_url: "http://localhost:8080".into(),
            model_name: "qwen3-4b-q4_k_m".into(),
        }
    }

    fn is_server_available(&self) -> bool {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(500))
            .build()
            .ok();
        if let Some(c) = client {
            c.get(format!("{}/health", self.base_url)).send().is_ok()
        } else {
            false
        }
    }
}

impl LlmProvider for LocalLlamaProvider {
    fn name(&self) -> &str {
        "local_llama"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: true,
            json_mode: true,
            vision: false,
            tool_calls: false, // grammar-constrained in Phase 2
            max_context_tokens: 8_192,
        }
    }

    fn complete(&self, req: &ChatRequest, _api_key: &str) -> Result<ChatResponse> {
        if !self.is_server_available() {
            bail!(
                "Local LLM server not available at {}. \
                 Start llama.cpp server with Qwen3 4B (Phase 2). \
                 Falling back to cloud provider.",
                self.base_url
            );
        }
        // Use OpenAI-compatible endpoint from llama.cpp.
        let legacy_req = crate::llm::CompletionRequest::from_chat_request(req);
        let client = reqwest::blocking::Client::new();
        let body = serde_json::json!({
            "model": self.model_name,
            "temperature": legacy_req.temperature,
            "max_tokens": legacy_req.max_tokens,
            "messages": [
                {"role": "system", "content": legacy_req.system},
                {"role": "user",   "content": legacy_req.user}
            ]
        });
        let value: serde_json::Value = client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&body)
            .send()?
            .error_for_status()?
            .json()?;
        let text = value["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        Ok(ChatResponse {
            content: text,
            input_tokens: 0,
            output_tokens: 0,
            model_used: self.model_name.clone(),
        })
    }

    fn complete_stream(
        &self,
        req: &ChatRequest,
        api_key: &str,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<ChatResponse> {
        // Phase 2: SSE streaming from llama.cpp. For now delegate to non-streaming.
        let resp = self.complete(req, api_key)?;
        on_token(&resp.content);
        Ok(resp)
    }
}
