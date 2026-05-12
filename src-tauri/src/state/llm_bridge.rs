use anyhow::Result;
use regex::Regex;

use crate::llm::{self, LlmProvider};
use crate::model_router::{route_task, TaskClass};
use crate::models::IntentProvenance;
use crate::redaction::redact_json_value;
use crate::router::RoutedIntent;
use crate::secrets;
use crate::telemetry;

use super::AppState;

type ProviderSelection = (Box<dyn LlmProvider>, String, String, String);

fn looks_sensitive(input: &str) -> bool {
    let patterns = [
        r"sk-[A-Za-z0-9]{10,}",
        r"gsk_[A-Za-z0-9]{10,}",
        r"ghp_[A-Za-z0-9]{10,}",
        r"(?i)password\s*[:=]",
        r"(?i)api[_-]?key\s*[:=]",
    ];
    patterns
        .iter()
        .any(|p| Regex::new(p).map(|re| re.is_match(input)).unwrap_or(false))
}

fn is_rate_limited_error(err: &anyhow::Error) -> bool {
    let text = err.to_string().to_lowercase();
    text.contains("429") || text.contains("rate limit") || text.contains("too many requests")
}

impl AppState {
    pub(crate) fn classify_intent(&self, input: &str) -> Result<RoutedIntent> {
        if looks_sensitive(input) {
            return Ok(RoutedIntent {
                intent: "unknown".to_string(),
                args: serde_json::json!({ "raw": input }),
                confidence: 0.0,
                source: IntentProvenance::LlmClassify,
                llm_payload: None,
            });
        }
        if let Some((_, cached)) = self
            .command_cache
            .lock()
            .ok()
            .and_then(|cache| cache.iter().find(|(k, _)| k == input).cloned())
        {
            return Ok(cached);
        }
        let candidates = self.provider_candidates(TaskClass::QuickClassify)?;
        for (provider, key, model, reason) in candidates {
            self.provider_quota
                .check_and_increment(provider.name())
                .map_err(anyhow::Error::msg)?;
            telemetry::log_router_decision("quick_classify", provider.name(), &model, &reason);
            match llm::classify_intent_with_retry(provider.as_ref(), &key, input) {
                Ok((intent, in_tok, out_tok)) => {
                    if let Ok(mut usage) = self.token_usage.lock() {
                        usage.0 += in_tok as u64;
                        usage.1 += out_tok as u64;
                    }
                    crate::startup::log_first_llm_completion_if_needed();
                    let llm_payload = serde_json::to_value(&intent).ok().map(|v| redact_json_value(&v));
                    let routed = RoutedIntent {
                        intent: intent.intent,
                        args: intent.args,
                        confidence: intent.confidence,
                        source: IntentProvenance::LlmClassify,
                        llm_payload,
                    };
                    self.cache_intent(input, &routed);
                    return Ok(routed);
                }
                Err(err) => {
                    if is_rate_limited_error(&err) {
                        std::thread::sleep(std::time::Duration::from_millis(250));
                        continue;
                    }
                }
            }
        }
        Ok(RoutedIntent {
            intent: "unknown".to_string(),
            args: serde_json::json!({ "raw": input }),
            confidence: 0.0,
            source: IntentProvenance::LlmClassify,
            llm_payload: None,
        })
    }

    pub(crate) fn run_text_tool(
        &self,
        tool: &str,
        text: &str,
        mut on_token: Option<&mut dyn FnMut(&str)>,
    ) -> Result<String> {
        let task = if tool == "rewrite" {
            TaskClass::Coding
        } else {
            TaskClass::GeneralChat
        };
        let candidates = self.provider_candidates(task)?;
        if candidates.is_empty() {
            return Ok("No provider key found. Add Groq, Gemini, or OpenRouter key in Settings.".to_string());
        }
        if looks_sensitive(text) {
            return Ok("Sensitive content detected. Cloud providers are blocked for this request.".into());
        }
        let prompt = match tool {
            "summarize" => {
                format!("Summarize the following text in concise bullet points:\n\n{text}")
            }
            "rewrite" => {
                format!("Rewrite the following text for clarity and brevity:\n\n{text}")
            }
            _ => text.to_string(),
        };
        let req = llm::CompletionRequest {
            system: "You are Neph, a concise productivity assistant.".to_string(),
            user: prompt,
            model: String::new(),
            temperature: 0.2,
            max_tokens: 500,
            json_mode: false,
        };
        let mut last_err: Option<anyhow::Error> = None;
        for (provider, key, model, reason) in candidates {
            self.provider_quota
                .check_and_increment(provider.name())
                .map_err(anyhow::Error::msg)?;
            telemetry::log_router_decision("text_tool", provider.name(), &model, &reason);
            let req = llm::CompletionRequest {
                model: model.clone(),
                ..req.clone()
            };
            let mut attempt = 0u32;
            loop {
                let mut noop = |_chunk: &str| {};
                let result = if let Some(cb) = on_token.as_deref_mut() {
                    provider.complete_stream(&req, &key, cb)
                } else {
                    provider.complete_stream(&req, &key, &mut noop)
                };
                match result {
                    Ok(response) => {
                        if let Ok(mut usage) = self.token_usage.lock() {
                            usage.0 += response.estimated_input_tokens as u64;
                            usage.1 += response.estimated_output_tokens as u64;
                        }
                        crate::startup::log_first_llm_completion_if_needed();
                        return Ok(format!(
                            "{}\n\n[usage ~in:{} out:{}]",
                            response.text, response.estimated_input_tokens, response.estimated_output_tokens
                        ));
                    }
                    Err(err) => {
                        if is_rate_limited_error(&err) && attempt < 2 {
                            let backoff_ms = 250u64 * (1u64 << attempt);
                            std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                            attempt += 1;
                            continue;
                        }
                        last_err = Some(err);
                        break;
                    }
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("all providers failed")))
    }

    fn provider_candidates(&self, task: TaskClass) -> Result<Vec<ProviderSelection>> {
        let route = route_task(task);
        let mut candidates = Vec::new();
        for provider in route.provider_chain {
            if let Some((client, key)) = provider_with_key(provider)? {
                candidates.push((client, key, route.model.to_string(), route.reason.clone()));
            }
        }
        Ok(candidates)
    }

    fn cache_intent(&self, input: &str, routed: &RoutedIntent) {
        if let Ok(mut cache) = self.command_cache.lock() {
            cache.retain(|(k, _)| k != input);
            cache.push((input.to_string(), routed.clone()));
            if cache.len() > 20 {
                let _ = cache.remove(0);
            }
        }
    }
}

fn provider_with_key(name: &str) -> Result<Option<(Box<dyn LlmProvider>, String)>> {
    match name {
        "groq" => {
            if let Some(key) = secrets::read_provider_key("groq")? {
                return Ok(Some((Box::new(llm::GroqProvider), key)));
            }
        }
        "gemini" => {
            if let Some(key) = secrets::read_provider_key("gemini")? {
                return Ok(Some((Box::new(llm::GeminiProvider), key)));
            }
        }
        "openrouter" => {
            if let Some(key) = secrets::read_provider_key("openrouter")? {
                return Ok(Some((Box::new(llm::OpenRouterProvider), key)));
            }
        }
        "anthropic" => {
            if let Some(key) = secrets::read_provider_key("anthropic")? {
                return Ok(Some((Box::new(crate::llm_anthropic::AnthropicProvider), key)));
            }
        }
        _ => {}
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::looks_sensitive;
    use crate::db;
    use crate::state::AppState;
    use tempfile::tempdir;

    #[test]
    fn sensitive_patterns_are_detected() {
        let cases = [
            "api_key=sk-abcdef1234567890",
            "password: hunter2",
            "gsk_testtoken123456789",
            "ghp_exampletoken123456789",
        ];
        for input in cases {
            assert!(looks_sensitive(input), "expected sensitive: {input}");
        }
    }

    #[test]
    fn sensitive_input_short_circuits_before_cache_or_provider() {
        let dir = tempdir().expect("tempdir");
        let (db_path, meta) = db::initialize_database(dir.path()).expect("db init");
        let state = AppState::new(db_path, meta);
        if let Ok(mut cache) = state.command_cache.lock() {
            cache.push((
                "api_key=sk-abcdef1234567890".into(),
                crate::router::RoutedIntent {
                    intent: "create_note".into(),
                    args: serde_json::json!({"body":"cached"}),
                    confidence: 1.0,
                    source: crate::models::IntentProvenance::LlmClassify,
                    llm_payload: None,
                },
            ));
        }
        let routed = state
            .classify_intent("api_key=sk-abcdef1234567890")
            .expect("classify");
        assert_eq!(routed.intent, "unknown");
    }
}
