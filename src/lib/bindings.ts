export type PaletteRunResponse =
  | { status: "completed"; output: string }
  | {
      status: "needConfirmation";
      planHash: string;
      preview: string;
      risk: string;
      token: string;
    }
  | { status: "rejected"; message: string };

export type TauriCommandMap = {
  hide_palette: undefined;
  run_palette_command: { input: string; confirmationToken?: string | null };
  get_history: undefined;
  save_provider_key: { provider: string; apiKey: string };
  set_active_provider: { provider: string };
  get_memory: { query?: string };
  get_admission_queue: undefined;
  keep_admission: { id: number };
  discard_admission: { id: number };
  update_memory_item: { id: number; content: string };
  toggle_memory_pin: { id: number; pinned: boolean };
  delete_memory_item: { id: number };
  test_provider: { provider: string };
  get_token_stats: undefined;
  report_issue_link: undefined;
  export_db_backup: undefined;
  export_memory_json: undefined;
  get_palette_hotkey: undefined;
  set_palette_hotkey: { spec: string };
  get_startup_diagnostics: undefined;
  export_diagnostic_bundle: undefined;
  get_dpapi_protect_exports: undefined;
  set_dpapi_protect_exports: { enabled: boolean };
};

export type TauriCommandResultMap = {
  hide_palette: void;
  run_palette_command: PaletteRunResponse;
  get_history: HistoryItem[];
  save_provider_key: void;
  set_active_provider: void;
  get_memory: MemoryItem[];
  get_admission_queue: AdmissionItem[];
  keep_admission: boolean;
  discard_admission: boolean;
  update_memory_item: boolean;
  toggle_memory_pin: boolean;
  delete_memory_item: boolean;
  test_provider: string;
  get_token_stats: [number, number];
  report_issue_link: string;
  export_db_backup: string;
  export_memory_json: string;
  get_palette_hotkey: string;
  set_palette_hotkey: string;
  get_startup_diagnostics: StartupDiagnostics;
  export_diagnostic_bundle: string;
  get_dpapi_protect_exports: boolean;
  set_dpapi_protect_exports: void;
};

export type HistoryItem = {
  id: number;
  input: string;
  intent: string;
  tool_name?: string;
  success?: boolean;
  risk_level?: string;
  state?: string;
  result_summary?: string;
  args_json?: string;
  created_at: string;
  provenance?: string;
  lineage_json?: string;
};

export type MemoryItem = {
  id: number;
  kind: string;
  content: string;
  pinned: boolean;
  created_at: string;
};

export type AdmissionItem = {
  id: number;
  content: string;
  kind: string | null;
  score: number | null;
  decision: string;
  created_at: string;
};

export type StartupDiagnostics = {
  paletteHotkey: string;
  webview2Version: string | null;
  webview2MeetsMinimum: boolean;
  webview2Minimum: string;
  webview2InstallUrl: string;
  imeHotkeyTip: string;
  vectorSearchEnabled: boolean;
  embeddingMode: string | null;
  dpapiProtectExports: boolean;
  wakeWordEnabled: boolean;
  mcpEnabled: boolean;
  orbV2Enabled: boolean;
};
