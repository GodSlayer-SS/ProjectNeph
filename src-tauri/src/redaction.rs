use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

static SK_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)sk-[a-z0-9]{10,}").expect("regex"));
static BEARER_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)bearer\s+[a-z0-9._\-]{8,}").expect("regex"));
static OPENAI_KEY_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)sk-[a-z0-9]{20,}").expect("regex"));

/// Redact common secret patterns from free-form text (logs, tracing, previews).
pub fn redact_secrets(text: &str) -> String {
    let mut out = SK_PATTERN.replace_all(text, "[REDACTED:sk]").to_string();
    out = BEARER_PATTERN.replace_all(&out, "[REDACTED:bearer]").to_string();
    out = OPENAI_KEY_PATTERN.replace_all(&out, "[REDACTED:key]").to_string();
    out
}

/// Deep-redact string values inside JSON (keys preserved).
pub fn redact_json_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                out.insert(k.clone(), redact_json_value(v));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(redact_json_value).collect()),
        Value::String(s) => Value::String(redact_secrets(s)),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_sk_pattern() {
        let s = "key sk-1234567890abcdef end";
        assert!(redact_secrets(s).contains("[REDACTED"));
        assert!(!redact_secrets(s).contains("sk-1234567890abcdef"));
    }
}
