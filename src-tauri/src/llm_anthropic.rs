/// Anthropic Claude provider — implements the existing `LlmProvider` trait.
///
/// Supports:
///   - Claude Sonnet 4.5 (code, reasoning, complex plans)
///   - SSE streaming (Anthropic's format differs from OpenAI)
///
/// Model routing: the `model_router` selects this provider for
/// `TaskClass::Coding` and `TaskClass::Reasoning`.

use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde_json::Value;
use std::io::{BufRead, BufReader};

use crate::llm::{CompletionRequest, CompletionResponse, LlmProvider};

pub struct AnthropicProvider;

const API_BASE: &str = "https://api.anthropic.com/v1";
const API_VERSION: &str = "2023-06-01";

impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn complete(&self, req: &CompletionRequest, api_key: &str) -> Result<CompletionResponse> {
        let client = Client::new();
        let model = if req.model.is_empty() {
            "claude-sonnet-4-5"
        } else {
            &req.model
        };
        let body = build_request_body(req, model, false);
        let value: Value = client
            .post(format!("{API_BASE}/messages"))
            .header("x-api-key", api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()?
            .error_for_status()?
            .json()?;
        let text = value["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("missing Anthropic completion content"))?
            .to_string();
        let (ei, eo) = usage_from_anthropic(&value);
        Ok(CompletionResponse {
            text,
            estimated_input_tokens: ei,
            estimated_output_tokens: eo,
        })
    }

    fn complete_stream(
        &self,
        req: &CompletionRequest,
        api_key: &str,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<CompletionResponse> {
        let client = Client::new();
        let model = if req.model.is_empty() {
            "claude-sonnet-4-5"
        } else {
            &req.model
        };
        let body = build_request_body(req, model, true);
        let response = client
            .post(format!("{API_BASE}/messages"))
            .header("x-api-key", api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()?
            .error_for_status()?;

        let mut reader = BufReader::new(response);
        let mut line = String::new();
        let mut full_text = String::new();
        let mut input_tokens = 0u32;
        let mut output_tokens = 0u32;

        while reader.read_line(&mut line)? > 0 {
            let trimmed = line.trim();
            if let Some(payload) = trimmed.strip_prefix("data: ") {
                if payload == "[DONE]" {
                    line.clear();
                    continue;
                }
                if let Ok(value) = serde_json::from_str::<Value>(payload) {
                    // Anthropic SSE event types
                    match value["type"].as_str() {
                        Some("content_block_delta") => {
                            if let Some(chunk) =
                                value["delta"]["text"].as_str()
                            {
                                on_token(chunk);
                                full_text.push_str(chunk);
                            }
                        }
                        Some("message_start") => {
                            input_tokens = value["message"]["usage"]["input_tokens"]
                                .as_u64()
                                .unwrap_or(0) as u32;
                        }
                        Some("message_delta") => {
                            output_tokens = value["usage"]["output_tokens"]
                                .as_u64()
                                .unwrap_or(0) as u32;
                        }
                        _ => {}
                    }
                }
            }
            line.clear();
        }

        Ok(CompletionResponse {
            text: full_text,
            estimated_input_tokens: input_tokens,
            estimated_output_tokens: output_tokens,
        })
    }
}

fn build_request_body(req: &CompletionRequest, model: &str, stream: bool) -> Value {
    let mut body = serde_json::json!({
        "model": model,
        "max_tokens": req.max_tokens,
        "system": req.system,
        "messages": [{ "role": "user", "content": req.user }],
        "stream": stream,
    });
    // Anthropic uses a different temperature range but same [0,1] semantics.
    body["temperature"] = serde_json::json!(req.temperature);
    body
}

fn usage_from_anthropic(value: &Value) -> (u32, u32) {
    let usage = &value["usage"];
    (
        usage["input_tokens"].as_u64().unwrap_or(0) as u32,
        usage["output_tokens"].as_u64().unwrap_or(0) as u32,
    )
}
