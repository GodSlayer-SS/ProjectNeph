/// ipc/tauri_cmds.rs — Re-exports and documentation for the Tauri command surface.
///
/// All `#[tauri::command]` functions live in `lib.rs` for now (Phase 1).
/// This module documents what commands exist and their expected types so the
/// IPC surface is auditable in one place.
///
/// Phase 2: Move command implementations here as `lib.rs` grows.
///
/// Commands exposed via `tauri::generate_handler!`:
///   hide_palette         → ()
///   run_palette_command  → PaletteRunResponse
///   get_history          → Vec<HistoryItem>
///   save_provider_key    → ()
///   test_provider        → String
///   set_active_provider  → ()
///   get_token_stats      → (u64, u64)
///   report_issue_link    → String
///   get_memory           → Vec<MemoryItem>
///   get_admission_queue  → Vec<AdmissionItem>
///   keep_admission       → bool
///   discard_admission    → bool
///   update_memory_item   → bool
///   toggle_memory_pin    → bool
///   delete_memory_item   → bool
///   export_db_backup     → String (path)
///   export_memory_json   → String (path)
///   get_palette_hotkey   → String
///   set_palette_hotkey   → String (confirmation msg)
///   get_startup_diagnostics → StartupDiagnostics
///   export_diagnostic_bundle → String (zip path)
///   get_dpapi_protect_exports → bool
///   set_dpapi_protect_exports → ()

// All command implementations are in lib.rs (Phase 1). See lib.rs `tauri::generate_handler!`.

