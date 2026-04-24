import { create } from "zustand";
import type { HistoryItem, MemoryItem, PaletteRunResponse, StartupDiagnostics } from "../lib/bindings";
import { invokeTyped } from "../lib/ipc";

type TabId = "palette" | "memory" | "history" | "settings" | "onboarding";
type RiskLevel = "green" | "yellow" | "red";

type PendingConfirmation = {
  input: string;
  risk: RiskLevel;
  token: string;
  planHash: string;
  preview: string;
};

type SettingsSection = "general" | "ai" | "privacy" | "memory" | "folders" | "about";

function displayPaletteResponse(res: PaletteRunResponse): string {
  if (res.status === "completed") {
    return res.output;
  }
  if (res.status === "rejected") {
    return res.message;
  }
  return res.preview;
}

type PaletteState = {
  activeTab: TabId;
  input: string;
  output: string;
  history: HistoryItem[];
  memoryItems: MemoryItem[];
  memoryQuery: string;
  provider: string;
  apiKey: string;
  pendingConfirmation: PendingConfirmation | null;
  redConfirmText: string;
  isLoading: boolean;
  isOffline: boolean;
  isRateLimited: boolean;
  errorMessage: string | null;
  lastLatencyMs: number;
  onboardingStep: number;
  onboardingSkipped: boolean;
  settingsSection: SettingsSection;
  providerTestResult: string;
  tokenStats: { input: number; output: number };
  hotkeySpec: string;
  hotkeySaveMessage: string | null;
  startupDiagnostics: StartupDiagnostics | null;
  setActiveTab: (tab: TabId) => void;
  setInput: (value: string) => void;
  setOutput: (value: string) => void;
  appendOutput: (chunk: string) => void;
  setProvider: (value: string) => void;
  setApiKey: (value: string) => void;
  setMemoryQuery: (value: string) => void;
  setRedConfirmText: (value: string) => void;
  setSettingsSection: (value: SettingsSection) => void;
  nextOnboardingStep: () => void;
  skipOnboarding: () => void;
  hidePalette: () => Promise<void>;
  runCommand: () => Promise<void>;
  confirmPendingCommand: () => Promise<void>;
  cancelPendingCommand: () => void;
  runUndo: () => Promise<void>;
  refreshHistory: () => Promise<void>;
  saveProviderKey: () => Promise<void>;
  refreshMemory: () => Promise<void>;
  updateMemory: (id: number, content: string) => Promise<void>;
  toggleMemoryPin: (id: number, pinned: boolean) => Promise<void>;
  deleteMemory: (id: number) => Promise<void>;
  testProvider: () => Promise<void>;
  refreshTokenStats: () => Promise<void>;
  exportDbBackup: () => Promise<void>;
  exportMemoryJson: () => Promise<void>;
  openReportIssue: () => Promise<void>;
  setHotkeySpec: (value: string) => void;
  loadHotkeyFromRust: () => Promise<void>;
  saveHotkeyToRust: () => Promise<void>;
  loadStartupDiagnostics: () => Promise<void>;
  exportDiagnosticBundle: () => Promise<void>;
  setDpapiExportSetting: (enabled: boolean) => Promise<void>;
};

export const usePaletteStore = create<PaletteState>((set, get) => ({
  activeTab: "palette",
  input: "",
  output: "Ready",
  history: [],
  memoryItems: [],
  memoryQuery: "",
  provider: "groq",
  apiKey: "",
  pendingConfirmation: null,
  redConfirmText: "",
  isLoading: false,
  isOffline: false,
  isRateLimited: false,
  errorMessage: null,
  lastLatencyMs: 0,
  onboardingStep: 1,
  onboardingSkipped: false,
  settingsSection: "general",
  providerTestResult: "",
  tokenStats: { input: 0, output: 0 },
  hotkeySpec: "ctrl+space",
  hotkeySaveMessage: null,
  startupDiagnostics: null,
  setActiveTab: (activeTab) => set({ activeTab }),
  setInput: (input) => set({ input }),
  setOutput: (output) => set({ output }),
  appendOutput: (chunk) => set((state) => ({ output: `${state.output}${chunk}` })),
  setProvider: (provider) => set({ provider }),
  setApiKey: (apiKey) => set({ apiKey }),
  setMemoryQuery: (memoryQuery) => set({ memoryQuery }),
  setRedConfirmText: (redConfirmText) => set({ redConfirmText }),
  setSettingsSection: (settingsSection) => set({ settingsSection }),
  nextOnboardingStep: () => set((state) => ({ onboardingStep: Math.min(3, state.onboardingStep + 1) })),
  skipOnboarding: () => set({ onboardingSkipped: true }),
  hidePalette: async () => {
    await invokeTyped("hide_palette");
  },
  runCommand: async () => {
    const { input } = get();
    if (!input.trim()) {
      return;
    }
    const trimmed = input.trim();
    const start = performance.now();
    set({ isLoading: true, errorMessage: null, isOffline: false, isRateLimited: false });
    try {
      const res = await invokeTyped("run_palette_command", { input: trimmed });
      if (res.status === "needConfirmation") {
        set({
          pendingConfirmation: {
            input: trimmed,
            risk: res.risk as RiskLevel,
            token: res.token,
            planHash: res.planHash,
            preview: res.preview,
          },
          redConfirmText: "",
          output: res.preview,
          lastLatencyMs: Math.round(performance.now() - start),
        });
        return;
      }
      const history = await invokeTyped("get_history");
      set({
        output: displayPaletteResponse(res),
        history,
        input: "",
        pendingConfirmation: null,
        redConfirmText: "",
        lastLatencyMs: Math.round(performance.now() - start),
      });
    } catch (error) {
      const text = String(error);
      set({
        errorMessage: text,
        isOffline: /network|offline|failed to fetch/i.test(text),
        isRateLimited: /429|rate/i.test(text),
      });
    } finally {
      set({ isLoading: false });
    }
  },
  confirmPendingCommand: async () => {
    const { pendingConfirmation, redConfirmText } = get();
    if (!pendingConfirmation) {
      return;
    }
    if (pendingConfirmation.risk === "red" && redConfirmText !== "DELETE") {
      set({ output: "Type DELETE to confirm red action." });
      return;
    }
    const start = performance.now();
    set({ isLoading: true, errorMessage: null });
    try {
      const res = await invokeTyped("run_palette_command", {
        input: pendingConfirmation.input,
        confirmationToken: pendingConfirmation.token,
      });
      if (res.status === "needConfirmation") {
        set({
          pendingConfirmation: {
            input: pendingConfirmation.input,
            risk: res.risk as RiskLevel,
            token: res.token,
            planHash: res.planHash,
            preview: res.preview,
          },
          output: res.preview,
        });
        return;
      }
      const history = await invokeTyped("get_history");
      set({
        output: displayPaletteResponse(res),
        history,
        input: "",
        pendingConfirmation: null,
        redConfirmText: "",
        lastLatencyMs: Math.round(performance.now() - start),
      });
    } catch (error) {
      set({ errorMessage: String(error) });
    } finally {
      set({ isLoading: false });
    }
  },
  cancelPendingCommand: () => {
    set({ pendingConfirmation: null, redConfirmText: "" });
  },
  runUndo: async () => {
    const res = await invokeTyped("run_palette_command", { input: ">undo" });
    const history = await invokeTyped("get_history");
    set({ output: displayPaletteResponse(res), history });
  },
  refreshHistory: async () => {
    const history = await invokeTyped("get_history");
    set({ history });
  },
  saveProviderKey: async () => {
    const { provider, apiKey } = get();
    if (!apiKey.trim()) {
      return;
    }
    await invokeTyped("save_provider_key", { provider, apiKey });
    await invokeTyped("set_active_provider", { provider });
    set({
      apiKey: "",
      output: `Saved ${provider} key in Credential Manager`,
    });
  },
  refreshMemory: async () => {
    const { memoryQuery } = get();
    const memoryItems = await invokeTyped("get_memory", {
      query: memoryQuery.trim() || undefined,
    });
    set({ memoryItems });
  },
  updateMemory: async (id, content) => {
    await invokeTyped("update_memory_item", { id, content });
    await get().refreshMemory();
  },
  toggleMemoryPin: async (id, pinned) => {
    await invokeTyped("toggle_memory_pin", { id, pinned });
    await get().refreshMemory();
  },
  deleteMemory: async (id) => {
    await invokeTyped("delete_memory_item", { id });
    await get().refreshMemory();
  },
  testProvider: async () => {
    const { provider } = get();
    await invokeTyped("set_active_provider", { provider });
    const providerTestResult = await invokeTyped("test_provider", { provider });
    set({ providerTestResult });
  },
  refreshTokenStats: async () => {
    const [input, output] = await invokeTyped("get_token_stats");
    set({ tokenStats: { input, output } });
  },
  exportDbBackup: async () => {
    const path = await invokeTyped("export_db_backup");
    set({ output: `Database backup exported: ${path}` });
  },
  exportMemoryJson: async () => {
    const path = await invokeTyped("export_memory_json");
    set({ output: `Memory JSON exported: ${path}` });
  },
  openReportIssue: async () => {
    const url = await invokeTyped("report_issue_link");
    set({ output: `Report issue: ${url}` });
  },
  setHotkeySpec: (hotkeySpec) => set({ hotkeySpec }),
  loadHotkeyFromRust: async () => {
    const hotkeySpec = await invokeTyped("get_palette_hotkey");
    set({ hotkeySpec, hotkeySaveMessage: null });
  },
  saveHotkeyToRust: async () => {
    const { hotkeySpec } = get();
    const hotkeySaveMessage = await invokeTyped("set_palette_hotkey", { spec: hotkeySpec });
    set({ hotkeySaveMessage });
  },
  loadStartupDiagnostics: async () => {
    const startupDiagnostics = await invokeTyped("get_startup_diagnostics");
    set({ startupDiagnostics });
  },
  exportDiagnosticBundle: async () => {
    const path = await invokeTyped("export_diagnostic_bundle");
    set({ output: `Diagnostic bundle written: ${path}` });
  },
  setDpapiExportSetting: async (enabled) => {
    await invokeTyped("set_dpapi_protect_exports", { enabled });
    await get().loadStartupDiagnostics();
  },
}));
