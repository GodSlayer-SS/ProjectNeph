#![allow(dead_code)]

#[derive(Debug, Clone, Copy)]
pub enum TaskClass {
    QuickClassify,
    GeneralChat,
    Coding,
    Vision,
    LongContext,
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
        TaskClass::QuickClassify => ProviderRoute {
            provider_chain: vec!["groq", "gemini", "openrouter"],
            model: "llama-3.1-8b-instant",
            reason: "quick_classify prioritizes low-latency responses".into(),
        },
        TaskClass::GeneralChat => ProviderRoute {
            provider_chain: vec!["groq", "gemini", "openrouter"],
            model: "llama-3.3-70b-versatile",
            reason: "general_chat defaults to fast high-quality cloud path".into(),
        },
        TaskClass::Coding => ProviderRoute {
            provider_chain: vec!["openrouter", "groq", "gemini"],
            model: "openrouter/auto",
            reason: "coding prefers specialist OpenRouter models first".into(),
        },
        TaskClass::Vision => ProviderRoute {
            provider_chain: vec!["gemini", "groq", "openrouter"],
            model: "gemini-2.0-flash",
            reason: "vision routes to Gemini-first for multimodal support".into(),
        },
        TaskClass::LongContext => ProviderRoute {
            provider_chain: vec!["gemini", "openrouter", "groq"],
            model: "gemini-2.0-flash",
            reason: "long_context requires large-context provider chain".into(),
        },
        TaskClass::Offline => ProviderRoute {
            provider_chain: vec!["groq"],
            model: "llama-3.1-8b-instant",
            reason: "offline fallback currently maps to local fast-path placeholder".into(),
        },
    }
}
