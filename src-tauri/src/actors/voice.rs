/// VoiceActor — Phase 1 (full implementation with barge-in).
///
/// Flow:
///   hotkey_down  → open WASAPI mic → stream chunks to Python VAD
///   VAD detects speech → accumulate audio → forward to STT (Groq Whisper via sidecar)
///   STT partial → emit `stt:partial`  (orb: listening)
///   STT final   → emit `stt:final`   (orb: thinking → PlannerActor takes over)
///   hotkey_up   → stop mic → flush remaining audio → STT → finalize
///
/// Barge-in (Phase 1):
///   A shared `CancelToken` is set true whenever:
///     - hotkey_down fires while TTS is still playing
///     - VAD detects speech energy > threshold during the `Speaking` state
///   The `play_wav_bytes` loop in lib.rs polls `is_cancelled()` every frame.
///   After cancellation, the token is reset before the next capture loop starts.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter};

use crate::actors::cancel::CancelToken;
use crate::ipc::pyside::PysideClient;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceState {
    Idle,
    Listening,
    Transcribing,
    Thinking,
    Speaking,
    Error,
}

#[derive(Clone, serde::Serialize)]
pub struct VoiceAmplitude {
    pub value: f32,
}

#[derive(Clone, serde::Serialize)]
pub struct SttPartial {
    pub text: String,
}

#[derive(Clone, serde::Serialize)]
pub struct SttFinal {
    pub text: String,
}

/// Emitted to the frontend when we have end-to-end latency data.
#[derive(Clone, serde::Serialize)]
pub struct VoiceLatency {
    /// Hotkey-down → first TTS audio frame, in milliseconds.
    pub total_ms: u128,
    /// Just the STT part.
    pub stt_ms: u128,
    /// Just the LLM-first-token part.
    pub llm_ms: u128,
}

// ── Audio buffer ──────────────────────────────────────────────────────────────

#[derive(Default)]
struct AudioBuf {
    samples: Vec<i16>,
}

impl AudioBuf {
    fn push_f32(&mut self, data: &[f32]) {
        for &s in data {
            self.samples.push((s.clamp(-1.0, 1.0) * 32767.0) as i16);
        }
    }

    fn rms(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.samples.iter().map(|&s| (s as f64).powi(2)).sum();
        ((sum / self.samples.len() as f64).sqrt() / 32767.0) as f32
    }

    fn as_le_bytes(&self) -> Vec<u8> {
        self.samples.iter().flat_map(|s| s.to_le_bytes()).collect()
    }
}

// ── VoiceActor ────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct VoiceActor {
    app: AppHandle,
    /// True while the hotkey is held (push-to-talk).
    recording: Arc<AtomicBool>,
    /// Shared barge-in cancellation token.
    /// Cancelling this stops any ongoing TTS playback.
    pub barge_cancel: CancelToken,
    /// Timestamp of hotkey-down, for end-to-end latency measurement.
    voice_start: Arc<Mutex<Option<std::time::Instant>>>,
    /// Last STT latency (ms) for this voice session.
    last_stt_ms: Arc<Mutex<Option<u128>>>,
    /// Last LLM first-token latency (ms) for this voice session.
    last_llm_ms: Arc<Mutex<Option<u128>>>,
    /// Last final transcript for this voice session.
    last_transcript: Arc<Mutex<Option<String>>>,
}

impl VoiceActor {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            recording: Arc::new(AtomicBool::new(false)),
            barge_cancel: CancelToken::new(),
            voice_start: Arc::new(Mutex::new(None)),
            last_stt_ms: Arc::new(Mutex::new(None)),
            last_llm_ms: Arc::new(Mutex::new(None)),
            last_transcript: Arc::new(Mutex::new(None)),
        }
    }

    // ── Hotkey events ─────────────────────────────────────────────────────────

    pub fn on_hotkey_down(&self) {
        if self.recording.load(Ordering::SeqCst) {
            return;
        }
        // Stamp latency start.
        if let Ok(mut t) = self.voice_start.lock() {
            *t = Some(std::time::Instant::now());
        }
        // Barge-in: cancel any ongoing TTS before starting to listen.
        self.barge_cancel.cancel();
        self.recording.store(true, Ordering::SeqCst);

        let actor = self.clone();
        std::thread::spawn(move || {
            // Small delay so TTS thread sees the cancel before we emit Listening.
            std::thread::sleep(std::time::Duration::from_millis(80));
            actor.barge_cancel.reset();

            if let Err(e) = actor.capture_loop() {
                tracing::error!(target: "neph_voice", "capture_loop error: {e}");
                let _ = actor.app.emit("voice:state", VoiceState::Error);
                let _ = actor.app.emit("llm:error", e.to_string());
            }
        });
    }

    pub fn on_hotkey_up(&self) {
        self.recording.store(false, Ordering::SeqCst);
    }

    /// Instant of the most recent hotkey-down (if any).
    pub fn voice_start_instant(&self) -> Option<std::time::Instant> {
        self.voice_start.lock().ok().and_then(|g| *g)
    }

    pub fn set_llm_first_token_ms(&self, ms: u128) {
        if let Ok(mut g) = self.last_llm_ms.lock() {
            *g = Some(ms);
        }
        let _ = self.app.emit("voice:latency_llm", ms);
    }

    pub fn snapshot_latencies(&self) -> (Option<u128>, Option<u128>) {
        let stt = self.last_stt_ms.lock().ok().and_then(|g| *g);
        let llm = self.last_llm_ms.lock().ok().and_then(|g| *g);
        (stt, llm)
    }

    pub fn last_transcript(&self) -> Option<String> {
        self.last_transcript.lock().ok().and_then(|g| g.clone())
    }

    // ── Capture loop ──────────────────────────────────────────────────────────

    fn capture_loop(&self) -> anyhow::Result<()> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

        self.emit_state(VoiceState::Listening)?;
        tracing::info!(target: "neph_voice", "capture_loop: started");

        // Connect to sidecar once per capture session if available.
        // If not available, we fall back to local RMS/silence heuristics.
        let sidecar = PysideClient::connect().ok();

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("no input device found"))?;

        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(16000),
            buffer_size: cpal::BufferSize::Fixed(512),
        };

        let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<f32>>(64);
        let err_flag = Arc::new(AtomicBool::new(false));
        let err_flag2 = err_flag.clone();

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _| {
                let _ = tx.try_send(data.to_vec());
            },
            move |e| {
                tracing::error!(target: "neph_voice", "cpal input error: {e}");
                err_flag2.store(true, Ordering::SeqCst);
            },
            None,
        )?;
        stream.play()?;

        let mut buf = AudioBuf::default();
        let mut speech_seen = false;
        let silence_chunks = 40_usize; // ~1.3s @ 32ms chunks
        let mut silence_count = 0usize;

        loop {
            let mut last_chunk_pcm: Option<Vec<u8>> = None;
            while let Ok(samples) = rx.try_recv() {
                buf.push_f32(&samples);

                // Convert just this chunk to i16 PCM bytes for VAD.
                let mut chunk_i16: Vec<i16> = Vec::with_capacity(samples.len());
                for &s in &samples {
                    chunk_i16.push((s.clamp(-1.0, 1.0) * 32767.0) as i16);
                }
                let mut bytes = Vec::with_capacity(chunk_i16.len() * 2);
                for s in chunk_i16 {
                    bytes.extend_from_slice(&s.to_le_bytes());
                }
                last_chunk_pcm = Some(bytes);
            }

            let amp = buf.rms();
            let _ = self.app.emit("voice:amplitude", VoiceAmplitude { value: amp });

            let is_recording = self.recording.load(Ordering::SeqCst);
            let stream_err = err_flag.load(Ordering::SeqCst);

            // Determine speech vs silence.
            let speech_detected = if let (Some(client), Some(chunk)) = (&sidecar, &last_chunk_pcm) {
                client.vad_process(chunk, 16000).unwrap_or(false)
            } else {
                // RMS fallback (no sidecar).
                amp >= 0.005
            };

            if speech_detected {
                speech_seen = true;
                silence_count = 0;
            } else if speech_seen {
                silence_count += 1;
            }

            let auto_stop = silence_count > silence_chunks;
            if (!is_recording || stream_err || auto_stop) && !buf.samples.is_empty() {
                tracing::info!(
                    target: "neph_voice",
                    samples = buf.samples.len(),
                    reason = if !is_recording {"hotkey_up"} else if auto_stop {"silence"} else {"error"},
                    "capture_loop: finalizing"
                );
                break;
            }
            if !is_recording && buf.samples.is_empty() {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(32));
        }

        drop(stream);
        self.emit_state(VoiceState::Transcribing)?;

        let t0 = std::time::Instant::now();
        let transcript = self.transcribe(buf)?;
        let stt_ms = t0.elapsed().as_millis();
        tracing::info!(target: "neph_voice", stt_ms = stt_ms, "STT latency");
        // Emit partial latency so the UI can display it.
        let _ = self.app.emit("voice:latency_stt", stt_ms);
        if let Ok(mut g) = self.last_stt_ms.lock() {
            *g = Some(stt_ms);
        }

        if transcript.trim().is_empty() {
            let _ = self.app.emit("voice:amplitude", VoiceAmplitude { value: 0.0 });
            self.emit_state(VoiceState::Idle)?;
            return Ok(());
        }

        tracing::info!(target: "neph_voice", text = %transcript, "stt:final");
        if let Ok(mut g) = self.last_transcript.lock() {
            *g = Some(transcript.clone());
        }
        let _ = self.app.emit("stt:final", SttFinal { text: transcript });
        self.emit_state(VoiceState::Thinking)?;
        Ok(())
    }

    // ── STT: Groq cloud primary → pyside local fallback (Blueprint §3/§6) ────

    fn transcribe(&self, buf: AudioBuf) -> anyhow::Result<String> {
        let pcm_bytes = buf.as_le_bytes();
        if pcm_bytes.is_empty() {
            return Ok(String::new());
        }

        // Build a minimal WAV header around the raw PCM so Groq's API
        // can identify the format (16-bit LE, mono, 16 kHz).
        let wav_bytes = pcm_to_wav(&pcm_bytes, 16000);

        // Read Groq key from OS credential store (non-blocking — already in
        // cache after first call).
        let groq_key = crate::secrets::read_provider_key("groq")
            .ok()
            .flatten();

        // Try Groq cloud STT first; fall back to pyside local Whisper.
        match crate::ipc::groq_stt::transcribe(
            &wav_bytes,
            "en",
            groq_key.as_deref().unwrap_or(""),
        ) {
            Ok(text) => {
                tracing::info!(target: "neph_voice", "STT: Groq cloud (primary)");
                // No partials from cloud API — emit as a single partial so
                // the UI shows something while the planner thinks.
                if !text.trim().is_empty() {
                    let _ = self.app.emit("stt:partial", SttPartial { text: text.clone() });
                }
                return Ok(text);
            }
            Err(e) => {
                tracing::warn!(
                    target: "neph_voice",
                    error = %e,
                    "STT: Groq failed, falling back to local faster-whisper"
                );
            }
        }

        // Local fallback via Python sidecar.
        let client = match PysideClient::connect() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    target: "neph_voice",
                    "Python sidecar unavailable ({}); skipping STT. \
                     Run `python -m nephis_pyside.pipe_server` to enable voice.",
                    e
                );
                return Ok(String::new());
            }
        };

        tracing::info!(target: "neph_voice", "STT: faster-whisper local (fallback)");
        let res = client.stt_transcribe_with_partials(&pcm_bytes, 16000)?;
        for p in &res.partials {
            if !p.trim().is_empty() {
                let _ = self.app.emit("stt:partial", SttPartial { text: p.clone() });
            }
        }
        Ok(res.text)
    }

    fn emit_state(&self, state: VoiceState) -> anyhow::Result<()> {
        self.app.emit("voice:state", state)?;
        Ok(())
    }
}

// ── WAV encoding helper ───────────────────────────────────────────────────────

/// Wrap raw 16-bit LE PCM bytes in a minimal RIFF WAV header.
/// Required because Groq's audio API needs a valid container format.
fn pcm_to_wav(pcm: &[u8], sample_rate: u32) -> Vec<u8> {
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    let block_align: u16 = channels * bits_per_sample / 8;
    let data_len = pcm.len() as u32;
    let chunk_size = 36 + data_len;

    let mut wav = Vec::with_capacity(44 + pcm.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&chunk_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());    // subchunk1 size
    wav.extend_from_slice(&1u16.to_le_bytes());     // PCM = 1
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    wav.extend_from_slice(pcm);
    wav
}
