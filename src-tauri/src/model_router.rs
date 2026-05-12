#![allow(dead_code)]

/// Task classes map to provider/model selections per blueprint §8.
#[derive(Debug, Clone, Copy)]
pub enum TaskClass {
    /// Rule-based + LLM fallback intent classify (fast, cheap).
    QuickClassify,
    /// General conversational queries — Gemini 2.5 Flash.
    GeneralChat,
    /// Code generation / review / debugging — Claude Sonnet 4.5.
    Coding,
    /// Complex multi-step reasoning — Claude Sonnet 4.5.
    Reasoning,
    /// Vision / GUI grounding — Gemini 2.5 Flash.
    Vision,
    /// Long context (summarise large docs) — Gemini 2.5 Flash.
    LongContext,
    /// Latency-critical one-shot replies — Groq Llama 3.3 70B.
    LatencyCritical,
    /// Memory distillation pass — Gemini Flash (cheap).
    MemoryDistill,
    /// Offline fallback (no cloud). Not used in Phase 1.
    Offline,
}

#[derive(Debug, Clone)]
pub struct ProviderRoute {
    pub provider_chain: Vec<&'static str>,
    pub model: &'static str,
    pub reason: String,
}

pub fn route_task(task: TaskClass) -> ProviderRoute {
    match task {
        // Fast, cheap — Groq Llama small first, Gemini Flash fallback
        TaskClass::QuickClassify => ProviderRoute {
            provider_chain: vec!["groq", "gemini", "openrouter"],
            model: "llama-3.1-8b-instant",
            reason: "quick_classify: low-latency intent router".into(),
        },
        // General conversation — Gemini 2.5 Flash (free, fast, multimodal)
        TaskClass::GeneralChat => ProviderRoute {
            provider_chain: vec!["gemini", "groq", "openrouter"],
            model: "gemini-2.5-flash-preview-05-20",
            reason: "general_chat: Gemini 2.5 Flash primary".into(),
        },
        // Code gen / review — Claude Sonnet 4.5
        TaskClass::Coding => ProviderRoute {
            provider_chain: vec!["anthropic", "openrouter", "gemini"],
            model: "claude-sonnet-4-5",
            reason: "coding: Claude Sonnet 4.5 primary".into(),
        },
        // Complex reasoning / multi-step plans — Claude Sonnet 4.5
        TaskClass::Reasoning => ProviderRoute {
            provider_chain: vec!["anthropic", "gemini", "openrouter"],
            model: "claude-sonnet-4-5",
            reason: "reasoning: Claude Sonnet 4.5 primary".into(),
        },
        // Vision / GUI grounding — Gemini 2.5 Flash
        TaskClass::Vision => ProviderRoute {
            provider_chain: vec!["gemini", "openrouter"],
            model: "gemini-2.5-flash-preview-05-20",
            reason: "vision: Gemini multimodal primary".into(),
        },
        // Long context (large docs) — Gemini 2.5 Flash (1M context)
        TaskClass::LongContext => ProviderRoute {
            provider_chain: vec!["gemini", "openrouter"],
            model: "gemini-2.5-flash-preview-05-20",
            reason: "long_context: Gemini 1M context window".into(),
        },
        // Latency-critical one-liners — Groq Llama 3.3 70B (~100ms TTFB)
        TaskClass::LatencyCritical => ProviderRoute {
            provider_chain: vec!["groq", "gemini"],
            model: "llama-3.3-70b-versatile",
            reason: "latency_critical: Groq ultra-low latency".into(),
        },
        // Memory distill — cheap Gemini Flash pass
        TaskClass::MemoryDistill => ProviderRoute {
            provider_chain: vec!["gemini", "groq"],
            model: "gemini-2.5-flash-preview-05-20",
            reason: "memory_distill: cheap classification pass".into(),
        },
        // Offline — not used in Phase 1, placeholder
        TaskClass::Offline => ProviderRoute {
            provider_chain: vec!["groq"],
            model: "llama-3.1-8b-instant",
            reason: "offline: local fast-path placeholder".into(),
        },
    }
}
