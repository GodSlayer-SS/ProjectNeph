use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub system: String,
    pub user: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub json_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub text: String,
    pub estimated_input_tokens: u32,
    pub estimated_output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResponse {
    pub intent: String,
    pub args: Value,
    pub confidence: f32,
}

pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    fn complete(&self, req: &CompletionRequest, api_key: &str) -> Result<CompletionResponse>;
    fn complete_stream(
        &self,
        req: &CompletionRequest,
        api_key: &str,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<CompletionResponse>;
}

pub struct GroqProvider;
pub struct GeminiProvider;
pub struct OpenRouterProvider;

fn estimate_tokens(text: &str) -> u32 {
    ((text.len() as f32) / 4.0).ceil() as u32
}

fn usage_tokens_from_openai_shape(value: &Value, fallback_in: u32, fallback_out: u32) -> (u32, u32) {
    if let Some(u) = value.get("usage") {
        let pi = u["prompt_tokens"]
            .as_u64()
            .or_else(|| u["input_tokens"].as_u64())
            .unwrap_or(0);
        let po = u["completion_tokens"]
            .as_u64()
            .or_else(|| u["output_tokens"].as_u64())
            .unwrap_or(0);
        if pi > 0 || po > 0 {
            return (pi as u32, po as u32);
        }
    }
    (fallback_in, fallback_out)
}

fn complete_chat(api_base: &str, req: &CompletionRequest, api_key: &str) -> Result<CompletionResponse> {
    let client = Client::new();
    let mut body = serde_json::json!({
      "model": req.model,
      "temperature": req.temperature,
      "max_tokens": req.max_tokens,
      "messages": [
        {"role": "system", "content": req.system},
        {"role": "user", "content": req.user}
      ]
    });
    if req.json_mode {
        body["response_format"] = serde_json::json!({ "type": "json_object" });
    }
    let value: Value = client
        .post(format!("{api_base}/chat/completions"))
        .bearer_auth(api_key)
        .json(&body)
        .send()?
        .error_for_status()?
        .json()?;
    let text = value["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow!("missing completion content"))?
        .to_string();
    let fb_in = estimate_tokens(&format!("{}{}", req.system, req.user));
    let fb_out = estimate_tokens(&text);
    let (estimated_input_tokens, estimated_output_tokens) =
        usage_tokens_from_openai_shape(&value, fb_in, fb_out);
    Ok(CompletionResponse {
        estimated_input_tokens,
        estimated_output_tokens,
        text,
    })
}

fn complete_chat_stream(
    api_base: &str,
    req: &CompletionRequest,
    api_key: &str,
    on_token: &mut dyn FnMut(&str),
) -> Result<CompletionResponse> {
    let client = Client::new();
    let mut body = serde_json::json!({
      "model": req.model,
      "temperature": req.temperature,
      "max_tokens": req.max_tokens,
      "stream": true,
      "messages": [
        {"role": "system", "content": req.system},
        {"role": "user", "content": req.user}
      ]
    });
    if req.json_mode {
        body["response_format"] = serde_json::json!({ "type": "json_object" });
    }
    let response = client
        .post(format!("{api_base}/chat/completions"))
        .bearer_auth(api_key)
        .json(&body)
        .send()?
        .error_for_status()?;
    let mut reader = BufReader::new(response);
    let mut line = String::new();
    let mut full_text = String::new();
    while reader.read_line(&mut line)? > 0 {
        let trimmed = line.trim();
        if let Some(payload) = trimmed.strip_prefix("data: ") {
            if payload == "[DONE]" {
                line.clear();
                continue;
            }
            if let Ok(value) = serde_json::from_str::<Value>(payload) {
                if let Some(chunk) = value["choices"][0]["delta"]["content"].as_str() {
                    on_token(chunk);
                    full_text.push_str(chunk);
                }
            }
        }
        line.clear();
    }
    let fb_in = estimate_tokens(&format!("{}{}", req.system, req.user));
    let fb_out = estimate_tokens(&full_text);
    Ok(CompletionResponse {
        text: full_text,
        estimated_input_tokens: fb_in,
        estimated_output_tokens: fb_out,
    })
}

fn complete_gemini_stream(
    req: &CompletionRequest,
    api_key: &str,
    on_token: &mut dyn FnMut(&str),
) -> Result<CompletionResponse> {
    let client = Client::new();
    let model = if req.model.is_empty() {
        "gemini-2.0-flash"
    } else {
        &req.model
    };
    let body = serde_json::json!({
        "system_instruction": { "parts": [{ "text": req.system }]},
        "contents": [{ "role": "user", "parts": [{ "text": req.user }] }],
        "generationConfig": {
          "temperature": req.temperature,
          "maxOutputTokens": req.max_tokens
        }
    });
    let response = client
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{model}:streamGenerateContent?alt=sse&key={api_key}"
        ))
        .json(&body)
        .send()?
        .error_for_status()?;
    let mut reader = BufReader::new(response);
    let mut line = String::new();
    let mut full_text = String::new();
    while reader.read_line(&mut line)? > 0 {
        let trimmed = line.trim();
        if let Some(payload) = trimmed.strip_prefix("data: ") {
            if let Ok(value) = serde_json::from_str::<Value>(payload) {
                if let Some(chunk) = value["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                    on_token(chunk);
                    full_text.push_str(chunk);
                }
            }
        }
        line.clear();
    }
    let fb_in = estimate_tokens(&format!("{}{}", req.system, req.user));
    let fb_out = estimate_tokens(&full_text);
    Ok(CompletionResponse {
        text: full_text,
        estimated_input_tokens: fb_in,
        estimated_output_tokens: fb_out,
    })
}

impl LlmProvider for GroqProvider {
    fn name(&self) -> &str {
        "groq"
    }

    fn complete(&self, req: &CompletionRequest, api_key: &str) -> Result<CompletionResponse> {
        complete_chat("https://api.groq.com/openai/v1", req, api_key)
    }

    fn complete_stream(
        &self,
        req: &CompletionRequest,
        api_key: &str,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<CompletionResponse> {
        complete_chat_stream("https://api.groq.com/openai/v1", req, api_key, on_token)
    }
}

impl LlmProvider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    fn complete(&self, req: &CompletionRequest, api_key: &str) -> Result<CompletionResponse> {
        let client = Client::new();
        let body = serde_json::json!({
            "system_instruction": { "parts": [{ "text": req.system }]},
            "contents": [{ "role": "user", "parts": [{ "text": req.user }] }],
            "generationConfig": {
              "temperature": req.temperature,
              "maxOutputTokens": req.max_tokens
            }
        });
        let model = if req.model.is_empty() {
            "gemini-2.0-flash"
        } else {
            &req.model
        };
        let value: Value = client
            .post(format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"))
            .json(&body)
            .send()?
            .error_for_status()?
            .json()?;
        let text = value["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("missing completion content"))?
            .to_string();
        let fb_in = estimate_tokens(&format!("{}{}", req.system, req.user));
        let fb_out = estimate_tokens(&text);
        let (estimated_input_tokens, estimated_output_tokens) =
            usage_tokens_from_openai_shape(&value, fb_in, fb_out);
        Ok(CompletionResponse {
            estimated_input_tokens,
            estimated_output_tokens,
            text,
        })
    }

    fn complete_stream(
        &self,
        req: &CompletionRequest,
        api_key: &str,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<CompletionResponse> {
        complete_gemini_stream(req, api_key, on_token)
    }
}

impl LlmProvider for OpenRouterProvider {
    fn name(&self) -> &str {
        "openrouter"
    }

    fn complete(&self, req: &CompletionRequest, api_key: &str) -> Result<CompletionResponse> {
        complete_chat("https://openrouter.ai/api/v1", req, api_key)
    }

    fn complete_stream(
        &self,
        req: &CompletionRequest,
        api_key: &str,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<CompletionResponse> {
        complete_chat_stream("https://openrouter.ai/api/v1", req, api_key, on_token)
    }
}

pub fn classify_intent_with_retry(
    provider: &dyn LlmProvider,
    api_key: &str,
    input: &str,
) -> Result<(IntentResponse, u32, u32)> {
    let system = "You are Neph's intent router. Output ONLY strict JSON matching: {\"intent\":\"launch_app|search_files|create_note|save_memory|retrieve_memory|summarize|rewrite|translate|explain|unknown\",\"args\":{},\"confidence\":0}. Never output prose.";
    let mut user = format!("Input: {input}");
    let mut last_in = 0u32;
    let mut last_out = 0u32;
    for attempt in 0..2 {
        let model = if provider.name() == "groq" {
            "llama-3.1-8b-instant"
        } else if provider.name() == "gemini" {
            "gemini-2.0-flash"
        } else {
            "openrouter/auto"
        };
        let response = provider.complete(
            &CompletionRequest {
                system: system.to_string(),
                user: user.clone(),
                model: model.to_string(),
                temperature: 0.1,
                max_tokens: 300,
                json_mode: true,
            },
            api_key,
        )?;
        last_in = response.estimated_input_tokens;
        last_out = response.estimated_output_tokens;
        if let Ok(parsed) = serde_json::from_str::<IntentResponse>(&response.text) {
            return Ok((parsed, last_in, last_out));
        }
        if attempt == 0 {
            user = format!("{input}\nYour previous output was invalid JSON. Return only valid JSON.");
        }
    }
    Ok((
        IntentResponse {
            intent: "unknown".to_string(),
            args: serde_json::json!({ "raw": input }),
            confidence: 0.0,
        },
        last_in,
        last_out,
    ))
}
