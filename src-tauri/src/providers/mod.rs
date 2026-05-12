/// providers/ — Concrete LLM/TTS provider implementations (Blueprint §2, §3, §4).
///
/// These modules are the ONLY place concrete provider types may be constructed.
/// Everything outside `providers/` and `memory/` depends solely on the
/// `traits::llm::LlmProvider` trait.  Swapping Gemini for a new model is a
/// 20-line PR here, zero changes elsewhere.
///
/// Phase 1 providers (LLM):
///   gemini.rs      — Google Gemini 2.5 Flash / Pro
///   anthropic.rs   — Anthropic Claude Sonnet 4.5
///   groq.rs        — Groq Llama 3.3 70B / 8B
///   openrouter.rs  — OpenRouter (multi-model fallback)
///
/// Phase 2 providers (LLM):
///   local_llama.rs — llama.cpp/CUDA (Qwen3 4B Q4_K_M, offline)
///
/// Phase 4 providers (TTS):
///   elevenlabs.rs  — ElevenLabs Flash streaming TTS

pub mod gemini;
pub mod anthropic;
pub mod groq;
pub mod openrouter;
pub mod local_llama;
/// Phase 4: ElevenLabs Flash TTS — stub until API key configured.
pub mod elevenlabs;

// Re-export the concrete types for use inside `actors/provider_router.rs` only.
pub use gemini::GeminiProviderV2;
pub use anthropic::AnthropicProviderV2;
pub use groq::GroqProviderV2;
pub use openrouter::OpenRouterProviderV2;
pub use local_llama::LocalLlamaProvider;
pub use elevenlabs::ElevenLabsProvider;
