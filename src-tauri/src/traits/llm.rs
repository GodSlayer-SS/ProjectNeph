use anyhow::Result;
use serde::{Deserialize, Serialize};

// ── Request / Response ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "system" | "user" | "assistant"
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    /// Request JSON-mode output from the model.
    pub json_mode: bool,
    /// Optional tool / function schemas to pass to the model.
    pub tools: Option<serde_json::Value>,
}

impl ChatRequest {
    pub fn simple(system: impl Into<String>, user: impl Into<String>) -> Self {
        Self {
            messages: vec![
                ChatMessage { role: "system".into(), content: system.into() },
                ChatMessage { role: "user".into(), content: user.into() },
            ],
            model: String::new(), // provider fills in the default
            temperature: 0.2,
            max_tokens: 2048,
            json_mode: false,
            tools: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub model_used: String,
}

/// A single streamed token delta.
#[derive(Debug, Clone)]
pub struct Token(pub String);

// ── Capabilities ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct ProviderCapabilities {
    pub streaming: bool,
    pub json_mode: bool,
    pub vision: bool,
    pub tool_calls: bool,
    pub max_context_tokens: u32,
}

// ── The Trait ─────────────────────────────────────────────────────────────────

/// Anything that can answer text/multimodal questions.
/// Implementations: `GeminiProvider`, `AnthropicProvider`, `GroqProvider`,
/// `OpenRouterProvider`, `LocalLlamaProvider` (Phase 2).
///
/// The blocking `complete` / `complete_stream` helpers on the existing `llm.rs`
/// implementations satisfy this trait via the `LlmProviderCompat` wrapper in
/// `providers/compat.rs` so that Phase-1 code keeps compiling while we migrate.
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> ProviderCapabilities;

    /// Non-streaming completion. Implementations may call the streaming path
    /// internally and collect — this default does exactly that.
    fn complete(&self, req: &ChatRequest, api_key: &str) -> Result<ChatResponse>;

    /// Streaming completion; `on_token` is called for each delta.
    fn complete_stream(
        &self,
        req: &ChatRequest,
        api_key: &str,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<ChatResponse>;
}
