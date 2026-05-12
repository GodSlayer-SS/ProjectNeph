/// memory/admission.rs — LLM-based admission control (Blueprint §7, §9 Phase 2).
///
/// "The difference between Jarvis (curates) and ChatGPT memory (hoards)."
///
/// The Admission Controller runs at the END of each voice session (or
/// periodically for long sessions). It takes the raw session Hot memory and
/// asks an LLM: "Which of these is worth keeping long-term?"
///
/// Blueprint §7: "Admission control after every session = the difference between
/// Jarvis and ChatGPT memory. Cheap distill pass classifies what's worth
/// keeping. Discard everything else."
///
/// Phase 1: Keyword-heuristic fallback (matches "remember", "my name is", etc.)
///          — this was the only implementation; now it's the offline fallback.
///
/// Phase 2 (this module): LLM-based distillation via `TaskClass::MemoryDistill`.
///          Returns a list of `AdmissionCandidate` with a score and reason.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::traits::memory::MemoryItem;

// ── Types ─────────────────────────────────────────────────────────────────────

/// One item the LLM evaluated from the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissionCandidate {
    pub id: i64,
    pub kind: String,
    pub content: String,
    /// Score in [0, 1] — how confidently worth storing.
    pub score: f32,
    /// LLM-provided reason, displayed in the UI admission queue.
    pub reason: String,
}

/// Outcome of an admission control pass.
#[derive(Debug, Default)]
pub struct AdmissionResult {
    /// Items approved for warm memory — save these.
    pub approved: Vec<MemoryItem>,
    /// Items rejected — discard.
    pub rejected: Vec<i64>,
    /// Items deferred to the user to decide (shown in the UI queue).
    pub queued_for_review: Vec<AdmissionCandidate>,
}

// ── AdmissionController ───────────────────────────────────────────────────────

pub struct AdmissionController;

impl AdmissionController {
    pub fn new() -> Self {
        Self
    }

    /// Phase 2 LLM-based admission pass.
    ///
    /// Sends the session items to Gemini Flash (`TaskClass::MemoryDistill`) in
    /// JSON mode. The model returns a scored list. Approved items are immediately
    /// stored; borderline items are queued for the UI review panel.
    ///
    /// Requires: a valid Gemini API key in the keyring.
    /// Falls back to `heuristic_filter` if the LLM is unavailable.
    pub fn run_distill_pass(
        &self,
        session_items: &[MemoryItem],
        api_key: &str,
    ) -> Result<AdmissionResult> {
        if session_items.is_empty() {
            return Ok(AdmissionResult::default());
        }

        // Try LLM distillation.
        match self.llm_distill(session_items, api_key) {
            Ok(result) => Ok(result),
            Err(e) => {
                tracing::warn!(
                    target: "neph_memory",
                    error = %e,
                    "admission: LLM distill failed, falling back to heuristic filter"
                );
                Ok(self.heuristic_filter(session_items))
            }
        }
    }

    /// LLM-based distillation using Gemini Flash via ProviderRouter (Blueprint §2 trait wall).
    fn llm_distill(
        &self,
        items: &[MemoryItem],
        api_key: &str,
    ) -> Result<AdmissionResult> {
        use crate::actors::provider_router::ProviderRouter;
        use crate::llm::CompletionRequest;
        use crate::model_router::TaskClass;

        let items_text = items
            .iter()
            .enumerate()
            .map(|(i, m)| format!("{}. [{}] {}", i + 1, m.kind, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let system = r#"You are a memory curator for a personal AI assistant.
Evaluate the following session memories and decide what is worth storing permanently.

Score each item 0.0 to 1.0:
- 1.0: Definitely keep (explicit preference, name, important fact, learned procedure)
- 0.7: Probably keep (useful context, recurring pattern)
- 0.4: Review (borderline, let user decide)
- 0.0: Discard (transient, not useful long-term)

Respond with a JSON array:
[{"index": 1, "score": 0.9, "reason": "explicit name preference"}]
Only include items with score > 0.0."#;

        let req = CompletionRequest {
            system: system.to_string(),
            user: format!("Session memories:\n{items_text}"),
            model: "gemini-2.0-flash".to_string(),
            max_tokens: 1024,
            temperature: 0.0,
            json_mode: true,
        };

        // Route through ProviderRouter — uses the configured Gemini key from keyring.
        let _ = api_key; // key is fetched inside ProviderRouter via secrets::read_provider_key
        let router = ProviderRouter::new();
        let mut buf = String::new();
        router.complete_stream(TaskClass::MemoryDistill, &req, &mut |chunk| {
            buf.push_str(chunk)
        })?;

        // Parse JSON array from LLM output.
        #[derive(Deserialize)]
        struct LlmItem {
            index: usize,
            score: f32,
            reason: String,
        }

        // Extract JSON array from the response.
        let json_start = buf.find('[').unwrap_or(0);
        let json_end = buf.rfind(']').map(|i| i + 1).unwrap_or(buf.len());
        let json_str = &buf[json_start..json_end];

        let scored: Vec<LlmItem> = serde_json::from_str(json_str).unwrap_or_default();

        let mut result = AdmissionResult::default();
        for entry in &scored {
            let idx = entry.index.saturating_sub(1);
            let Some(item) = items.get(idx) else { continue };

            if entry.score >= 0.7 {
                result.approved.push(item.clone());
            } else if entry.score >= 0.35 {
                let id = item.id.unwrap_or(-(idx as i64));
                result.queued_for_review.push(AdmissionCandidate {
                    id,
                    kind: item.kind.clone(),
                    content: item.content.clone(),
                    score: entry.score,
                    reason: entry.reason.clone(),
                });
            } else {
                if let Some(id) = item.id {
                    result.rejected.push(id);
                }
            }
        }

        tracing::info!(
            target: "neph_memory",
            approved = result.approved.len(),
            queued = result.queued_for_review.len(),
            rejected = result.rejected.len(),
            "admission: LLM distill pass complete"
        );
        Ok(result)
    }

    /// Heuristic fallback — matches known patterns (no LLM required).
    fn heuristic_filter(&self, items: &[MemoryItem]) -> AdmissionResult {
        const KEYWORDS: &[&str] = &[
            "remember",
            "my name is",
            "i prefer",
            "i always",
            "i hate",
            "i love",
            "don't forget",
            "important:",
            "password",
            "api key",
        ];

        let mut result = AdmissionResult::default();
        for item in items {
            let lower = item.content.to_lowercase();
            if KEYWORDS.iter().any(|kw| lower.contains(kw)) {
                result.approved.push(item.clone());
            } else {
                if let Some(id) = item.id {
                    result.rejected.push(id);
                }
            }
        }
        result
    }
}

impl Default for AdmissionController {
    fn default() -> Self {
        Self::new()
    }
}
