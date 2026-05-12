/// Rust client for the Windows named-pipe JSON-RPC connection to the Python sidecar.
///
/// The sidecar process (`apps/pyside/nephis_pyside/pipe_server.py`) must be
/// started before this client connects.  In Phase 1 the sidecar is started
/// manually.  In Phase 2 it will be launched as a child process by `startup.rs`.
///
/// Usage:
/// ```rust
/// let client = PysideClient::connect()?;
/// let text = client.stt_transcribe(&audio_bytes, 16000)?;
/// let audio = client.tts_speak("Hello from Nephis")?;
/// ```

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};

#[cfg(windows)]
use std::fs::OpenOptions;

const PIPE_NAME: &str = r"\\.\pipe\NephPyside";

// ── JSON-RPC types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'a str,
    id: String,
    method: &'a str,
    params: Value,
}

#[derive(Deserialize)]
struct RpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: Option<String>,
    result: Option<Value>,
    error: Option<Value>,
}

// ── Client ────────────────────────────────────────────────────────────────────

pub struct PysideClient {
    #[cfg(windows)]
    pipe: Arc<Mutex<std::fs::File>>,
}

#[derive(Debug, Clone)]
pub struct SttResult {
    pub partials: Vec<String>,
    pub text: String,
}

impl PysideClient {
    /// Connect to the sidecar named pipe. Retries up to 3 times with 200ms delay.
    pub fn connect() -> Result<Self> {
        #[cfg(windows)]
        {
            let mut last_err = anyhow!("pipe not available");
            for attempt in 0..3 {
                match OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(PIPE_NAME)
                {
                    Ok(f) => {
                        tracing::info!(target: "neph_ipc", "pyside pipe connected (attempt {})", attempt + 1);
                        return Ok(Self { pipe: Arc::new(Mutex::new(f)) });
                    }
                    Err(e) => {
                        last_err = e.into();
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                }
            }
            Err(last_err)
        }
        #[cfg(not(windows))]
        {
            anyhow::bail!("PysideClient is Windows-only in Phase 1")
        }
    }

    /// Quick non-retrying ping — returns true if the sidecar is already running.
    ///
    /// Used by `launch_pyside_sidecar()` to:
    /// 1. Skip re-launch if the pipe is already connectable.
    /// 2. Verify readiness ~1.5s after spawning the process.
    ///
    /// Timeout: single attempt, no retry delay.
    pub fn ping_quick() -> bool {
        #[cfg(windows)]
        {
            let Ok(f) = OpenOptions::new().read(true).write(true).open(PIPE_NAME) else {
                return false;
            };
            let client = Self { pipe: Arc::new(Mutex::new(f)) };
            client.ping().is_ok()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    fn call(&self, method: &str, params: Value) -> Result<Value> {
        let req = RpcRequest {
            jsonrpc: "2.0",
            id: uuid_v4(),
            method,
            params,
        };
        let mut line = serde_json::to_string(&req)? + "\n";

        #[cfg(windows)]
        {
            let mut pipe = self.pipe.lock().map_err(|_| anyhow!("pipe mutex poisoned"))?;
            pipe.write_all(line.as_bytes())?;
            pipe.flush()?;

            let mut reader = BufReader::new(&*pipe);
            let mut resp_line = String::new();
            reader.read_line(&mut resp_line)?;

            let resp: RpcResponse = serde_json::from_str(&resp_line)?;
            if let Some(err) = resp.error {
                anyhow::bail!("sidecar error: {}", err);
            }
            resp.result.ok_or_else(|| anyhow!("empty sidecar result"))
        }
        #[cfg(not(windows))]
        {
            anyhow::bail!("not on Windows")
        }
    }

    // ── STT ──────────────────────────────────────────────────────────────────

    pub fn stt_transcribe(&self, audio_bytes: &[u8], sample_rate: u32) -> Result<String> {
        use base64::Engine;
        let audio_b64 = base64::engine::general_purpose::STANDARD.encode(audio_bytes);
        let result = self.call(
            "stt.transcribe",
            serde_json::json!({ "audio_b64": audio_b64, "sample_rate": sample_rate }),
        )?;
        Ok(result
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string())
    }

    /// STT with optional partial segments (when the sidecar supports it).
    ///
    /// Backward-compatible: if `partials` is missing, returns an empty vec.
    pub fn stt_transcribe_with_partials(&self, audio_bytes: &[u8], sample_rate: u32) -> Result<SttResult> {
        use base64::Engine;
        let audio_b64 = base64::engine::general_purpose::STANDARD.encode(audio_bytes);
        let result = self.call(
            "stt.transcribe",
            serde_json::json!({ "audio_b64": audio_b64, "sample_rate": sample_rate }),
        )?;

        let partials = result
            .get("partials")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let text = result
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        Ok(SttResult { partials, text })
    }

    // ── TTS ──────────────────────────────────────────────────────────────────

    pub fn tts_speak(&self, text: &str) -> Result<Vec<u8>> {
        use base64::Engine;
        let result = self.call("tts.speak", serde_json::json!({ "text": text }))?;
        let b64 = result["audio_b64"].as_str().unwrap_or_default();
        Ok(base64::engine::general_purpose::STANDARD.decode(b64)?)
    }

    // ── VAD ──────────────────────────────────────────────────────────────────

    pub fn vad_process(&self, audio_bytes: &[u8], sample_rate: u32) -> Result<bool> {
        use base64::Engine;
        let audio_b64 = base64::engine::general_purpose::STANDARD.encode(audio_bytes);
        let result = self.call(
            "vad.process",
            serde_json::json!({ "audio_b64": audio_b64, "sample_rate": sample_rate }),
        )?;
        Ok(result["speech_detected"].as_bool().unwrap_or(false))
    }

    // ── Ping ─────────────────────────────────────────────────────────────────

    pub fn ping(&self) -> Result<()> {
        self.call("ping", serde_json::json!({}))?;
        Ok(())
    }
}

fn uuid_v4() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        rng.gen::<u32>(),
        rng.gen::<u16>() & 0xffff,
        rng.gen::<u16>() & 0x0fff,
        (rng.gen::<u16>() & 0x3fff) | 0x8000,
        rng.gen::<u64>() & 0xffffffffffff,
    )
}
