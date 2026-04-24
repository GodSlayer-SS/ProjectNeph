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
mod hotkey;
mod llm;
mod model_router;
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

            let app_handle = app.handle().clone();
            let hotkey_manager = GlobalHotKeyManager::new()?;
            hotkey_manager.register(hotkey)?;

            app.manage(AppState::new(db_path, db_meta));
            std::mem::forget(hotkey_manager);
            startup::log_palette_infra_ready("hotkey_registered");

            if let Some(window) = app_handle.get_webview_window("main") {
                window.hide()?;
            }

            std::thread::spawn(move || {
                let receiver = GlobalHotKeyEvent::receiver();

                while let Ok(event) = receiver.recv() {
                    if event.id != hotkey_id {
                        continue;
                    }

                    if let Some(window) = app_handle.get_webview_window("main") {
                        let is_visible = window.is_visible().unwrap_or(false);

                        if is_visible {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            });

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
