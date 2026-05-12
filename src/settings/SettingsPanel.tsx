/**
 * src/settings/SettingsPanel.tsx
 *
 * Blueprint §4: apps/desktop/src/settings/
 * Extracted from monolithic App.tsx.
 */

import type { StartupDiagnostics } from "../state/paletteStore";

type SettingsSection = "general" | "ai" | "privacy" | "memory" | "folders" | "about";

interface SettingsPanelProps {
  settingsSection: SettingsSection;
  provider: string;
  apiKey: string;
  hotkeySpec: string;
  hotkeySaveMessage: string | null;
  startupDiagnostics: StartupDiagnostics | null;
  providerTestResult: string | null;
  tokenStats: { input: number; output: number };
  setSettingsSection: (s: SettingsSection) => void;
  setProvider: (p: string) => void;
  setApiKey: (k: string) => void;
  setHotkeySpec: (s: string) => void;
  saveProviderKey: () => void;
  testProvider: () => void;
  refreshTokenStats: () => void;
  exportDbBackup: () => void;
  exportMemoryJson: () => void;
  openReportIssue: () => void;
  saveHotkeyToRust: () => Promise<void>;
  exportDiagnosticBundle: () => Promise<void>;
  setDpapiExportSetting: (enabled: boolean) => Promise<void>;
}

export function SettingsPanel({
  settingsSection,
  provider,
  apiKey,
  hotkeySpec,
  hotkeySaveMessage,
  startupDiagnostics,
  providerTestResult,
  tokenStats,
  setSettingsSection,
  setProvider,
  setApiKey,
  setHotkeySpec,
  saveProviderKey,
  testProvider,
  refreshTokenStats,
  exportDbBackup,
  exportMemoryJson,
  openReportIssue,
  saveHotkeyToRust,
  exportDiagnosticBundle,
  setDpapiExportSetting,
}: SettingsPanelProps) {
  const sections: SettingsSection[] = ["general", "ai", "privacy", "memory", "folders", "about"];

  return (
    <div className="panel">
      <h3>Settings</h3>
      <div className="memory-row">
        {sections.map((s) => (
          <button key={s} className="tab-pill" onClick={() => setSettingsSection(s)}>{s}</button>
        ))}
      </div>

      {settingsSection === "general" && (
        <div className="settings-grid">
          <div className="output-row">
            Palette global shortcut (restart Neph after change). If Ctrl+Space conflicts with your IME, use Ctrl+Shift+Space or Alt+Space.
          </div>
          <label className="output-row" htmlFor="hotkey-spec">Hotkey</label>
          <select id="hotkey-spec" value={hotkeySpec} onChange={(e) => setHotkeySpec(e.target.value)}>
            <option value="ctrl+space">Ctrl+Space</option>
            <option value="ctrl+shift+space">Ctrl+Shift+Space</option>
            <option value="alt+space">Alt+Space</option>
          </select>
          <div className="memory-row">
            <button className="secondary-btn" type="button" onClick={() => void saveHotkeyToRust()}>Save hotkey</button>
          </div>
          {hotkeySaveMessage && <div className="output-row">{hotkeySaveMessage}</div>}
        </div>
      )}

      {settingsSection === "ai" && (
        <div className="settings-grid">
          <select value={provider} onChange={(e) => setProvider(e.target.value)}>
            <option value="groq">Groq</option>
            <option value="gemini">Gemini</option>
            <option value="openrouter">OpenRouter</option>
            <option value="anthropic">Anthropic (Claude Sonnet 4.5)</option>
          </select>
          <input className="palette-input" value={apiKey} onChange={(e) => setApiKey(e.target.value)} placeholder="Paste API key" />
          <div className="memory-row">
            <button className="secondary-btn" onClick={saveProviderKey}>Save Key</button>
            <button className="secondary-btn" onClick={testProvider}>Test Provider</button>
            <button className="secondary-btn" onClick={refreshTokenStats}>Refresh Token Stats</button>
          </div>
          {providerTestResult && <div className="output-row">{providerTestResult}</div>}
          <div className="output-row">Session tokens: in {tokenStats.input} / out {tokenStats.output}</div>
        </div>
      )}

      {settingsSection === "privacy" && (
        <div className="settings-grid">
          <div className="output-row">
            Export memory JSON with Windows DPAPI (current user). Files use a <code>.json.dpapi</code> extension and can only be read on the same machine/profile.
          </div>
          <label className="output-row">
            <input
              type="checkbox"
              checked={startupDiagnostics?.dpapiProtectExports ?? false}
              onChange={(e) => void setDpapiExportSetting(e.target.checked)}
            />{" "}
            Protect memory exports with DPAPI
          </label>
          <div className="memory-row">
            <button className="secondary-btn" type="button" onClick={() => void exportDiagnosticBundle()}>
              Export diagnostic bundle (ZIP)
            </button>
          </div>
          <div className="output-row">Other privacy toggles (clipboard, window context) remain planned.</div>
        </div>
      )}

      {settingsSection === "memory" && (
        <div className="memory-row">
          <button className="secondary-btn" onClick={exportMemoryJson}>Export Memory JSON</button>
          <button className="secondary-btn" onClick={exportDbBackup}>Export DB Backup</button>
        </div>
      )}

      {settingsSection === "folders" && (
        <div className="output-row">Indexed folders: Documents, Desktop, Downloads (editable in upcoming pass).</div>
      )}

      {settingsSection === "about" && (
        <>
          <div className="output-row">Neph v0.1.0-dev</div>
          {startupDiagnostics && (
            <div className="output-row">
              <div>Configured hotkey: {startupDiagnostics.paletteHotkey}</div>
              <div>
                WebView2:{" "}
                {startupDiagnostics.webview2Version ?? "not detected in registry (install Evergreen if the window is blank)"}
                {startupDiagnostics.webview2Version && (
                  <> — meets minimum {startupDiagnostics.webview2Minimum}: {startupDiagnostics.webview2MeetsMinimum ? "yes" : "no"}</>
                )}
              </div>
              {!startupDiagnostics.webview2MeetsMinimum && (
                <div>Install or update: <a href={startupDiagnostics.webview2InstallUrl} target="_blank" rel="noreferrer">WebView2 runtime</a></div>
              )}
              <div>{startupDiagnostics.imeHotkeyTip}</div>
              <div>Embedding: {startupDiagnostics.embeddingMode ?? "n/a"} — vector extension active: {startupDiagnostics.vectorSearchEnabled ? "yes" : "no"}</div>
              <div>Phase 4 toggles — Wake-word: {startupDiagnostics.wakeWordEnabled ? "on" : "off"}, MCP: {startupDiagnostics.mcpEnabled ? "on" : "off"}, Orb v2: {startupDiagnostics.orbV2Enabled ? "on" : "off"}</div>
            </div>
          )}
          <button className="secondary-btn" onClick={openReportIssue}>Report Issue</button>
        </>
      )}
    </div>
  );
}
