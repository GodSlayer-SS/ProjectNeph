/// ipc/groq_stt.rs — Groq Whisper cloud STT (Blueprint §3, §6, Phase 1 primary).
///
/// Blueprint §3 says: "STT: Groq Whisper API (<300ms) primary;
///                         faster-whisper local fallback".
///
/// This module calls Groq's `/openai/v1/audio/transcriptions` endpoint using
/// the `whisper-large-v3-turbo` model. At ~250ms TTFB it satisfies the
/// Blueprint's <1s end-to-end latency budget.
///
/// Usage in voice pipeline:
/// 1. Try `groq_stt::transcribe(audio_bytes, "en", groq_api_key)?`
/// 2. On failure (rate limit, offline, missing key), fall back to pyside STT.

use anyhow::{anyhow, bail, Result};

const GROQ_STT_URL: &str =
    "https://api.groq.com/openai/v1/audio/transcriptions";

/// Transcribe PCM audio via Groq Whisper API.
///
/// `audio_bytes` must be a valid WAV or FLAC byte slice (16-bit PCM, any sample rate).
/// `language` is a BCP-47 language code (e.g. "en"); pass "" for auto-detect.
///
/// Returns the transcribed text on success, or an error suitable for fallback.
pub fn transcribe(audio_bytes: &[u8], language: &str, api_key: &str) -> Result<String> {
    if api_key.trim().is_empty() {
        bail!("groq_stt: no API key — falling back to local STT");
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Groq's transcription endpoint expects multipart/form-data with the audio
    // file and optional metadata fields.
    let audio_part = reqwest::blocking::multipart::Part::bytes(audio_bytes.to_vec())
        .file_name("audio.wav")
        .mime_str("audio/wav")?;

    let mut form = reqwest::blocking::multipart::Form::new()
        .part("file", audio_part)
        .text("model", "whisper-large-v3-turbo")
        .text("response_format", "json");

    if !language.is_empty() {
        form = form.text("language", language.to_string());
    }

    let response = client
        .post(GROQ_STT_URL)
        .header("Authorization", format!("Bearer {api_key}"))
        .multipart(form)
        .send()
        .map_err(|e| anyhow!("groq_stt: request failed: {e}"))?;

    let status = response.status();
    let body = response.text().unwrap_or_default();

    if !status.is_success() {
        bail!("groq_stt: API error {status}: {body}");
    }

    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| anyhow!("groq_stt: JSON parse error: {e}"))?;

    let text = json["text"]
        .as_str()
        .ok_or_else(|| anyhow!("groq_stt: missing 'text' in response"))?
        .trim()
        .to_string();

    tracing::debug!(
        target: "neph_stt",
        chars = text.len(),
        "groq_stt: transcription complete"
    );

    Ok(text)
}

/// Cloud-first STT: try Groq API, fall back to pyside local Whisper on any error.
///
/// This is the **recommended call site** in the voice actor — it satisfies
/// Blueprint §6's "Groq Whisper API primary; faster-whisper local fallback" mandate.
pub fn transcribe_cloud_first(
    audio_bytes: &[u8],
    language: &str,
    groq_key: Option<&str>,
    pyside: &crate::ipc::pyside::PysideClient,
    sample_rate: u32,
) -> Result<String> {
    // Try cloud first.
    if let Some(key) = groq_key.filter(|k| !k.trim().is_empty()) {
        match transcribe(audio_bytes, language, key) {
            Ok(text) => {
                tracing::info!(target: "neph_stt", "STT: Groq cloud (primary)");
                return Ok(text);
            }
            Err(e) => {
                tracing::warn!(target: "neph_stt", error = %e, "STT: Groq failed, falling back to local");
            }
        }
    }

    // Local fallback via Python sidecar.
    tracing::info!(target: "neph_stt", "STT: faster-whisper local (fallback)");
    pyside.stt_transcribe(audio_bytes, sample_rate)
}
