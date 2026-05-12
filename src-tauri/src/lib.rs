mod apps;
mod agent;
mod confirmation;
mod ctx;
mod db;
mod diagnostics;
mod dpapi_win;
mod embeddings;
mod execution;
mod files;
/// Hotkey parsing — canonical code is in `actors::hotkey`; this shim keeps
/// existing `hotkey::parse_hotkey` call sites compiling unchanged.
mod hotkey;
mod llm;
mod llm_anthropic;
mod model_router;
mod mcp;
mod models;
mod network_allowlist;
mod path_policy;
mod rate_limit;
mod redaction;
mod router;
mod secrets;
mod skills;
mod startup;
mod state;
mod telemetry;
mod tools;
mod webview2;
mod win_compat;

// ── New architecture modules (Blueprint v2) ───────────────────────────────────
/// The 8 stable interfaces — nothing outside `providers/` and `memory/`
/// may import a concrete type; they depend only on these traits.
pub mod traits;
/// Tokio-based actor shells (Blueprint §4 — hotkey, voice, planner, executor,
/// memory, automation, provider_router, ui_bridge).
pub mod actors;
/// Execution domain enforcement (filesystem, network, browser, shell).
pub mod domains;
/// IPC — named-pipe client to Python sidecar and typed Tauri event names.
pub mod ipc;
/// Memory tiers — Hot (session) + Warm (SQLite) + Cold (LanceDB embeddings, Phase 2).
pub mod memory;
/// In-process typed event bus — typed event name constants (Blueprint §4).
pub mod bus;
/// Concrete LLM provider implementations behind the trait wall (Blueprint §2, §4).
/// ONLY this module may construct GeminiProvider, AnthropicProvider, etc.
pub mod providers;
/// Trust kernel components per Blueprint §4: risk, confirmation, path_policy, capabilities.
pub mod safety;
/// SQLite data layer — re-exports `crate::db` under the Blueprint §4 mandated `store` namespace.
pub mod store;

use std::path::Path;

use dirs_next::data_local_dir;
use rusqlite::Connection;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use state::AppState;
use tauri::{Emitter, Manager, State};

fn init_tracing(data_root: &Path) {
    let log_dir = data_root.join("logs");
    let _ = std::fs::create_dir_all(&log_dir);
    let file_appender = tracing_appender::rolling::daily(&log_dir, "neph");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    std::mem::forget(guard);
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        "warn,tauri_app_lib=info"
            .parse()
            .expect("static env filter")
    });
    use tracing_subscriber::fmt;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(fmt::layer().with_ansi(false).with_writer(non_blocking))
        .try_init();
}

fn spawn_wal_checkpoint_thread(db_path: std::path::PathBuf) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(30));
        if let Ok(c) = Connection::open(&db_path) {
            let _ = c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        }
    });
}

/// Auto-launch the Python ML sidecar (Blueprint §9 Phase 1, BUG-6 fix).
///
/// Spawns `python -m nephis_pyside.pipe_server` as a detached background process.
/// Waits up to 2s for the named-pipe to become connectable, then sends a ping.
/// Non-fatal: if the sidecar fails to start, voice will log a warning per call.
fn launch_pyside_sidecar() {
    use std::process::{Command, Stdio};
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    // If the pipe already exists (sidecar already running), skip re-launch.
    // We check by trying to connect briefly — if successful, it's already up.
    let already_running = ipc::pyside::PysideClient::ping_quick();
    if already_running {
        tracing::info!(target: "neph_sidecar", "pyside already running — skip launch");
        return;
    }

    tracing::info!(target: "neph_sidecar", "launching Python ML sidecar...");

    // On Windows, spawn with CREATE_NO_WINDOW so no console pops up.
    #[cfg(windows)]
    let result = {
        use std::os::windows::process::CommandExt;
        Command::new("python")
            .args(["-m", "nephis_pyside.pipe_server"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
            .spawn()
    };
    #[cfg(not(windows))]
    let result = Command::new("python")
        .args(["-m", "nephis_pyside.pipe_server"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    match result {
        Ok(child) => {
            tracing::info!(
                target: "neph_sidecar",
                pid = child.id(),
                "pyside sidecar spawned"
            );
            // Give it time to start the pipe server before the first voice call.
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(1500));
                match ipc::pyside::PysideClient::ping_quick() {
                    true => tracing::info!(target: "neph_sidecar", "pyside ping OK"),
                    false => tracing::warn!(
                        target: "neph_sidecar",
                        "pyside ping failed after 1.5s — voice will not work. \
                         Ensure Python is in PATH and nephis-pyside is installed: \
                         `pip install -e apps/pyside`"
                    ),
                }
            });
        }
        Err(e) => {
            tracing::warn!(
                target: "neph_sidecar",
                error = %e,
                "could not spawn pyside sidecar. Voice will not work. \
                 Ensure Python is in PATH: `python -m nephis_pyside.pipe_server`"
            );
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            startup::record_setup_start();
            let data_root = data_local_dir()
                .unwrap_or_else(std::env::temp_dir)
                .join("Neph");
            let _ = std::fs::create_dir_all(&data_root);
            init_tracing(&data_root);

            let (db_path, db_meta) = db::initialize_database(&data_root)
                .map_err(|err| std::io::Error::other(err.to_string()))?;
            spawn_wal_checkpoint_thread(db_path.clone());

            // ── Auto-launch Python ML sidecar (BUG-6 fix) ────────────────────
            // Blueprint §9 Phase 1: "Spin up Python sidecar with named-pipe IPC."
            // The sidecar must be running before voice can work. We spawn it
            // as a detached background process and do a best-effort ping check.
            // A failure is logged as a warning — the rest of the app continues.
            launch_pyside_sidecar();

            #[cfg(windows)]
            {
                if let Some(v) = webview2::evergreen_runtime_version() {
                    if !webview2::version_ge(&v, webview2::MIN_RECOMMENDED_VERSION) {
                        tracing::warn!(
                            installed = %v,
                            min = webview2::MIN_RECOMMENDED_VERSION,
                            "WebView2 runtime is below the recommended minimum; update from {}",
                            webview2::INSTALL_URL
                        );
                    }
                } else {
                    tracing::warn!(
                        "WebView2 Evergreen runtime not found in registry; install from {}",
                        webview2::INSTALL_URL
                    );
                }
            }

            let hk_spec = db::read_setting(&db_path, "palette_hotkey")
                .map_err(|e| std::io::Error::other(e.to_string()))?
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "ctrl+space".into());
            let hotkey = hotkey::parse_hotkey(&hk_spec).unwrap_or_else(|e| {
                tracing::warn!(error = %e, spec = %hk_spec, "invalid palette_hotkey; falling back to ctrl+space");
                hotkey::parse_hotkey("ctrl+space").expect("default hotkey parses")
            });
            let hotkey_id = hotkey.id();

            // Voice push-to-talk hotkey: Alt+V
            // Down = start listening, Up = stop and transcribe.
            let voice_hotkey = {
                use global_hotkey::hotkey::{Code, HotKey, Modifiers};
                HotKey::new(Some(Modifiers::ALT), Code::KeyV)
            };
            let voice_hotkey_id = voice_hotkey.id();

            let app_handle = app.handle().clone();
            let voice_app_handle = app.handle().clone();
            let hotkey_manager = GlobalHotKeyManager::new()?;
            hotkey_manager.register(hotkey)?;
            hotkey_manager.register(voice_hotkey)?;

            app.manage(AppState::new(db_path, db_meta));
            std::mem::forget(hotkey_manager);
            startup::log_palette_infra_ready("hotkey_registered");

            if let Some(window) = app_handle.get_webview_window("main") {
                window.hide()?;
            }

            // Create the VoiceActor once; it is Clone so the thread captures it.
            let voice_actor = actors::voice::VoiceActor::new(voice_app_handle);
            // Clone the barge-in token so the TTS listener can check it.
            let barge_cancel = voice_actor.barge_cancel.clone();
            let voice_actor_for_metrics = voice_actor.clone();

            std::thread::spawn(move || {
                let receiver = GlobalHotKeyEvent::receiver();

                while let Ok(event) = receiver.recv() {
                    if event.id == hotkey_id {
                        // Palette toggle (existing behaviour — unchanged).
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            if is_visible {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    } else if event.id == voice_hotkey_id {
                        // Push-to-talk: KeyState::Pressed → down, Released → up.
                        use global_hotkey::HotKeyState;
                        match event.state {
                            HotKeyState::Pressed  => voice_actor.on_hotkey_down(),
                            HotKeyState::Released => voice_actor.on_hotkey_up(),
                        }
                    }
                }
            });

            // Wire stt:final → PlannerActor.
            // When the VoiceActor emits a final transcript, run it through the
            // existing run_palette_command path so the response streams back to
            // the UI via llm:token / llm:done events.
            {
                let stt_app = app.handle().clone();
                let metrics_voice = voice_actor_for_metrics.clone();
                stt_app.listen("stt:final", move |event| {
                    #[derive(serde::Deserialize)]
                    struct Payload { text: String }
                    let Ok(payload) = serde_json::from_str::<Payload>(event.payload()) else {
                        return;
                    };
                    let transcript = payload.text;
                    if transcript.trim().is_empty() {
                        return;
                    }
                    let plan_app = stt_app.clone();
                    std::thread::spawn(move || {
                        let state = plan_app.state::<AppState>();
                        let llm_t0 = std::time::Instant::now();
                        let mut first_token: Option<std::time::Instant> = None;
                        let mut token_emitter = |chunk: &str| {
                            if first_token.is_none() {
                                first_token = Some(std::time::Instant::now());
                                let llm_ms = llm_t0.elapsed().as_millis();
                                metrics_voice.set_llm_first_token_ms(llm_ms);
                            }
                            let _ = plan_app.emit("llm:token", chunk);
                        };

                        let result = state.run_palette_command(&transcript, None, Some(&mut token_emitter));
                        match result {
                            Ok(models::PaletteRunResponse::Completed { output }) => {
                                let _ = plan_app.emit("llm:done", output);
                            }
                            Ok(models::PaletteRunResponse::NeedConfirmation { preview, .. }) => {
                                let _ = plan_app.emit("llm:done",
                                    format!("[Confirmation needed] {preview}"));
                            }
                            Ok(models::PaletteRunResponse::Rejected { message }) => {
                                let _ = plan_app.emit("llm:error", message);
                            }
                            Err(e) => {
                                let _ = plan_app.emit("llm:error", e.to_string());
                            }
                        }
                    });
                });
            }

            // Wire llm:done → TTS via Python sidecar (barge-in aware).
            // Falls back silently if sidecar is not running.
            {
                let tts_app = app.handle().clone();
                let tts_cancel = barge_cancel.clone();
                let metrics_voice = voice_actor.clone();
                tts_app.listen("llm:done", move |event| {
                    let raw = event.payload().to_string();
                    let text = serde_json::from_str::<String>(&raw)
                        .unwrap_or_else(|_| raw.trim_matches('"').to_string());
                    if text.is_empty() || text.starts_with('[') {
                        return;
                    }
                    let _ = tts_app.emit("voice:state", actors::voice::VoiceState::Speaking);
                    let speak_app = tts_app.clone();
                    let cancel_tok = tts_cancel.clone();
                    let voice_for_latency = metrics_voice.clone();
                    std::thread::spawn(move || {
                        use crate::ipc::pyside::PysideClient;
                        let tts_t0 = std::time::Instant::now();
                        let client = match PysideClient::connect() {
                            Ok(c) => c,
                            Err(_) => {
                                let _ = speak_app.emit("voice:state",
                                    actors::voice::VoiceState::Idle);
                                return;
                            }
                        };
                        let speak_text: String = text.chars().take(600).collect();
                        match client.tts_speak(&speak_text) {
                            Ok(bytes) if !bytes.is_empty() => {
                                let tts_first_audio_ms = tts_t0.elapsed().as_millis();
                                let _ = speak_app.emit("voice:latency_tts_first_audio", tts_first_audio_ms);
                                let total_ms = voice_for_latency
                                    .voice_start_instant()
                                    .map(|t| t.elapsed().as_millis())
                                    .unwrap_or(0);
                                let (stt_ms, llm_ms) = voice_for_latency.snapshot_latencies();
                                let payload = actors::voice::VoiceLatency {
                                    total_ms,
                                    stt_ms: stt_ms.unwrap_or(0),
                                    llm_ms: llm_ms.unwrap_or(0),
                                };
                                let _ = speak_app.emit("voice:latency", payload);

                                // Phase 2: voice session audit + LLM-based admission control.
                                // AdmissionController tries Gemini Flash first, falls back
                                // to keyword heuristic if no key is available (Blueprint §7).
                                if let Some(transcript) = voice_for_latency.last_transcript() {
                                    let state = speak_app.state::<AppState>();
                                    let _ = state.log_voice_session(Some(&transcript), total_ms);
                                    // Run admission control off the hot path.
                                    let admit_app = speak_app.clone();
                                    let admit_transcript = transcript.clone();
                                    std::thread::spawn(move || {
                                        let state = admit_app.state::<AppState>();
                                        run_post_session_admission(&state, &admit_transcript);
                                    });
                                }

                                // play_audio_bytes handles MP3 and WAV via rodio (BUG-1 fix).
                                if let Err(e) = play_audio_bytes(&bytes, &cancel_tok) {
                                    tracing::warn!(target: "neph_tts", "playback: {e}");
                                }
                            }
                            Ok(_) => {}
                            Err(e) => tracing::warn!(target: "neph_tts", "tts_speak: {e}"),
                        }
                        let _ = speak_app.emit("voice:state", actors::voice::VoiceState::Idle);
                    });
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            hide_palette,
            run_palette_command,
            get_history,
            save_provider_key,
            test_provider,
            set_active_provider,
            get_token_stats,
            report_issue_link,
            get_memory,
            get_admission_queue,
            keep_admission,
            discard_admission,
            update_memory_item,
            toggle_memory_pin,
            delete_memory_item,
            export_db_backup,
            export_memory_json,
            get_palette_hotkey,
            set_palette_hotkey,
            get_startup_diagnostics,
            export_diagnostic_bundle,
            get_dpapi_protect_exports,
            set_dpapi_protect_exports,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn hide_palette(window: tauri::Window) -> Result<(), String> {
    window.hide().map_err(|err| err.to_string())
}

#[tauri::command]
fn run_palette_command(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    payload: models::RunPalettePayload,
) -> Result<models::PaletteRunResponse, String> {
    let mut token_emitter = |chunk: &str| {
        let _ = app.emit("llm:token", chunk);
    };
    let response = state
        .run_palette_command(&payload.input, payload.confirmation_token, Some(&mut token_emitter))
        .inspect_err(|err| {
            if let Some(root) = data_local_dir() {
                let _ =
                    telemetry::write_crash_log(&root.join("Neph").join("logs"), &err.to_string());
            }
        })?;
    match &response {
        models::PaletteRunResponse::Completed { output } => {
            let _ = app.emit("llm:done", output);
        }
        models::PaletteRunResponse::Rejected { message } => {
            let _ = app.emit("llm:error", message);
        }
        models::PaletteRunResponse::NeedConfirmation { preview, .. } => {
            let _ = app.emit("llm:done", preview);
        }
    }
    Ok(response)
}

#[tauri::command]
fn get_history(state: State<'_, AppState>) -> Result<Vec<models::HistoryItem>, String> {
    state.history().map_err(|err| err.to_string())
}

#[tauri::command]
fn save_provider_key(provider: String, api_key: String) -> Result<(), String> {
    secrets::save_provider_key(&provider, &api_key).map_err(|err| err.to_string())
}

#[tauri::command]
fn set_active_provider(state: State<'_, AppState>, provider: String) -> Result<(), String> {
    if let Ok(mut current) = state.provider.lock() {
        *current = provider;
    }
    Ok(())
}

#[tauri::command]
fn test_provider(provider: String) -> Result<String, String> {
    let key = secrets::read_provider_key(&provider).map_err(|err| err.to_string())?;
    match key {
        Some(_) => Ok(format!("{provider} key is configured")),
        None => Ok(format!("{provider} key not found")),
    }
}

#[tauri::command]
fn get_token_stats(state: State<'_, AppState>) -> Result<(u64, u64), String> {
    Ok(state.token_usage())
}

#[tauri::command]
fn report_issue_link() -> Result<String, String> {
    Ok("https://github.com/mohit/Project-Neph/issues/new?template=bug_report.md".to_string())
}

#[tauri::command]
fn get_memory(
    state: State<'_, AppState>,
    query: Option<String>,
) -> Result<Vec<models::MemoryItem>, String> {
    state
        .list_memory(query.as_deref())
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn get_admission_queue(state: State<'_, AppState>) -> Result<Vec<models::AdmissionItem>, String> {
    state.list_admission_queue().map_err(|e| e.to_string())
}

#[tauri::command]
fn keep_admission(state: State<'_, AppState>, id: i64) -> Result<bool, String> {
    state.keep_admission_as_memory(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn discard_admission(state: State<'_, AppState>, id: i64) -> Result<bool, String> {
    state.discard_admission(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_memory_item(state: State<'_, AppState>, id: i64, content: String) -> Result<bool, String> {
    state.update_memory(id, &content).map_err(|err| err.to_string())
}

#[tauri::command]
fn toggle_memory_pin(state: State<'_, AppState>, id: i64, pinned: bool) -> Result<bool, String> {
    state
        .set_memory_pin(id, pinned)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn delete_memory_item(state: State<'_, AppState>, id: i64) -> Result<bool, String> {
    state.delete_memory(id).map_err(|err| err.to_string())
}

#[tauri::command]
fn export_db_backup(state: State<'_, AppState>) -> Result<String, String> {
    state.export_db_backup().map_err(|err| err.to_string())
}

#[tauri::command]
fn export_memory_json(state: State<'_, AppState>) -> Result<String, String> {
    state.export_memory_json().map_err(|err| err.to_string())
}

#[tauri::command]
fn get_palette_hotkey(state: State<'_, AppState>) -> Result<String, String> {
    db::read_setting(&state.db_path, "palette_hotkey")
        .map_err(|e| e.to_string())?
        .filter(|s| !s.trim().is_empty())
        .map(Ok)
        .unwrap_or_else(|| Ok("ctrl+space".into()))
}

#[tauri::command]
fn set_palette_hotkey(state: State<'_, AppState>, spec: String) -> Result<String, String> {
    let normalized: String = spec.chars().filter(|c| !c.is_whitespace()).map(|c| c.to_ascii_lowercase()).collect();
    if !hotkey::PRESET_SPECS.contains(&normalized.as_str()) {
        return Err(format!(
            "Unsupported hotkey. Choose one of: {}",
            hotkey::PRESET_SPECS.join(", ")
        ));
    }
    hotkey::parse_hotkey(&normalized).map_err(|e| e.to_string())?;
    db::write_setting(&state.db_path, "palette_hotkey", &normalized).map_err(|e| e.to_string())?;
    Ok("Saved. Restart Neph for the change to take effect.".into())
}

#[tauri::command]
fn get_startup_diagnostics(state: State<'_, AppState>) -> Result<models::StartupDiagnostics, String> {
    let palette_hotkey = db::read_setting(&state.db_path, "palette_hotkey")
        .map_err(|e| e.to_string())?
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "ctrl+space".into());
    let webview2_version = webview2::evergreen_runtime_version();
    let webview2_meets_minimum = if cfg!(windows) {
        webview2_version
            .as_deref()
            .map(|v| webview2::version_ge(v, webview2::MIN_RECOMMENDED_VERSION))
            .unwrap_or(false)
    } else {
        true
    };
    let sqlite_vec_loaded = db::read_setting(&state.db_path, "sqlite_vec_loaded")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let vector_search_enabled = sqlite_vec_loaded == "1";
    let embedding_mode = db::read_setting(&state.db_path, "embedding_mode").map_err(|e| e.to_string())?;
    let dpapi_protect_exports = db::read_setting(&state.db_path, "dpapi_protect_exports")
        .map_err(|e| e.to_string())?
        .map(|v| v == "1")
        .unwrap_or(false);
    let wake_word_enabled = db::read_setting(&state.db_path, "wake_word_enabled")
        .map_err(|e| e.to_string())?
        .map(|v| v == "1")
        .unwrap_or(false);
    let mcp_enabled = db::read_setting(&state.db_path, "mcp_enabled")
        .map_err(|e| e.to_string())?
        .map(|v| v == "1")
        .unwrap_or(false);
    let orb_v2_enabled = db::read_setting(&state.db_path, "orb_v2_enabled")
        .map_err(|e| e.to_string())?
        .map(|v| v == "1")
        .unwrap_or(false);
    Ok(models::StartupDiagnostics {
        palette_hotkey,
        webview2_version,
        webview2_meets_minimum,
        webview2_minimum: webview2::MIN_RECOMMENDED_VERSION.to_string(),
        webview2_install_url: webview2::INSTALL_URL.to_string(),
        ime_hotkey_tip: "Ctrl+Space is used by some East Asian IMEs to switch input modes. If the palette does not open, try Ctrl+Shift+Space or Alt+Space in Settings > General, then restart Neph.".into(),
        vector_search_enabled,
        embedding_mode,
        dpapi_protect_exports,
        wake_word_enabled,
        mcp_enabled,
        orb_v2_enabled,
    })
}

#[tauri::command]
fn export_diagnostic_bundle(state: State<'_, AppState>) -> Result<String, String> {
    let logs_dir = data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("Neph")
        .join("logs");
    diagnostics::write_diagnostic_zip(&state.db_path, &logs_dir)
        .map_err(|e| e.to_string())
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn get_dpapi_protect_exports(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(db::read_setting(&state.db_path, "dpapi_protect_exports")
        .map_err(|e| e.to_string())?
        .map(|v| v == "1")
        .unwrap_or(false))
}

#[tauri::command]
fn set_dpapi_protect_exports(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    db::write_setting(
        &state.db_path,
        "dpapi_protect_exports",
        if enabled { "1" } else { "0" },
    )
    .map_err(|e| e.to_string())
}

/// Play audio bytes (WAV or MP3) through the default output device.
///
/// **BUG-1 FIX**: The previous implementation manually parsed a RIFF WAV
/// header from bytes[22..27]. EdgeTTS streams MP3 (not WAV), so this silently
/// produced garbage/silence on every TTS response. `rodio` auto-detects the
/// format from magic bytes and handles both WAV and MP3.
///
/// Cancellable: the caller passes a `CancelToken`; if it is set (barge-in)
/// the playback stops within one poll tick (~20ms).
fn play_audio_bytes(bytes: &[u8], cancel: &actors::cancel::CancelToken) -> anyhow::Result<()> {
    use rodio::{Decoder, OutputStream, Sink};
    use std::io::Cursor;

    if bytes.is_empty() {
        return Ok(());
    }

    // `OutputStream` must stay alive for the duration of playback.
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| anyhow::anyhow!("audio output unavailable: {e}"))?;

    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| anyhow::anyhow!("could not create audio sink: {e}"))?;

    // rodio auto-detects format from magic bytes (MP3 ID3/sync, RIFF WAV, OGG, etc.)
    let cursor = Cursor::new(bytes.to_vec());
    let source = Decoder::new(cursor)
        .map_err(|e| anyhow::anyhow!("audio decode failed: {e} (bytes len={})", bytes.len()))?;

    sink.append(source);

    // Poll until done or cancelled (barge-in).
    while !sink.empty() && !cancel.is_cancelled() {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    if cancel.is_cancelled() {
        sink.stop();
    }
    // Dropping `sink` and `_stream` ends playback.
    Ok(())
}


/// Post-session admission control (Blueprint §7).
///
/// Runs after each voice exchange: feeds the transcript through the LLM-based
/// `AdmissionController`, which scores items and decides:
///   approved   → immediately saved to warm memory
///   for_review → enqueued to the UI admission_queue (user decides)
///   rejected   → discarded
///
/// Falls back to keyword heuristic if no Gemini key is configured.
fn run_post_session_admission(state: &AppState, transcript: &str) {
    use crate::memory::admission::AdmissionController;
    use crate::traits::memory::{MemoryItem, MemoryTier};

    if transcript.trim().is_empty() {
        return;
    }

    // Build a single-item list from the transcript.
    let item = MemoryItem {
        id: None,
        kind: "episode".into(),
        content: transcript.to_string(),
        score: None,
        tier: MemoryTier::Hot,
        pinned: false,
    };

    // Try LLM distillation; fall back to heuristic automatically.
    let api_key = secrets::read_provider_key("gemini")
        .ok()
        .flatten()
        .unwrap_or_default();

    let controller = AdmissionController::new();
    let result = controller.run_distill_pass(&[item], &api_key);

    match result {
        Ok(admission) => {
            // Approved → persist directly to warm memory.
            for mem in &admission.approved {
                let _ = state.save_memory(&mem.kind, &mem.content);
                tracing::debug!(
                    target: "neph_memory",
                    kind = %mem.kind,
                    "admission: auto-approved to warm memory"
                );
            }
            // For-review → enqueue in UI admission_queue.
            for candidate in &admission.queued_for_review {
                let _ = state.enqueue_admission_candidate(
                    &candidate.content,
                    Some(&candidate.kind),
                    Some(candidate.score),
                );
                tracing::debug!(
                    target: "neph_memory",
                    score = candidate.score,
                    reason = %candidate.reason,
                    "admission: queued for user review"
                );
            }
            if !admission.rejected.is_empty() {
                tracing::debug!(
                    target: "neph_memory",
                    count = admission.rejected.len(),
                    "admission: discarded items"
                );
            }
        }
        Err(e) => {
            tracing::warn!(
                target: "neph_memory",
                error = %e,
                "post-session admission failed — transcript not admitted"
            );
        }
    }
}
