import { useEffect, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import { usePaletteStore } from "./state/paletteStore";
import { OrbCanvas } from "./orb/OrbCanvas";
import { OrbCanvasV2 } from "./orb/OrbCanvasV2";
import { TranscriptStrip } from "./orb/TranscriptStrip";
import { LatencyBadge } from "./orb/LatencyBadge";
import { initOrbListeners } from "./state/orbStore";
import { PalettePanel } from "./palette/PalettePanel";
import { MemoryInspector } from "./memory-inspector/MemoryInspector";
import { SettingsPanel } from "./settings/SettingsPanel";

// ── History panel (small enough to stay inline for now) ───────────────────────
import { HistoryPanel } from "./history/HistoryPanel";

function App() {
  const store = usePaletteStore();

  // Wire orb event listeners once at startup
  useEffect(() => { initOrbListeners(); }, []);

  // Load hotkey when navigating to settings > general
  useEffect(() => {
    if (store.activeTab === "settings" && store.settingsSection === "general") {
      void store.loadHotkeyFromRust();
    }
  }, [store.activeTab, store.settingsSection, store.loadHotkeyFromRust]);

  // Load diagnostics when needed
  useEffect(() => {
    if (
      store.activeTab === "memory" ||
      (store.activeTab === "settings" && (store.settingsSection === "about" || store.settingsSection === "privacy"))
    ) {
      void store.loadStartupDiagnostics();
    }
  }, [store.activeTab, store.settingsSection, store.loadStartupDiagnostics]);

  // Load admission queue when memory tab opens
  useEffect(() => {
    if (store.activeTab === "memory") void store.refreshAdmissionQueue();
  }, [store.activeTab, store.refreshAdmissionQueue]);

  // LLM streaming events
  useEffect(() => {
    let unlistenToken: (() => void) | null = null;
    let unlistenDone: (() => void) | null = null;
    let unlistenErr: (() => void) | null = null;
    void listen<string>("llm:token", (e) => store.appendOutput(e.payload)).then((u) => { unlistenToken = u; });
    void listen<string>("llm:done", (e) => store.setOutput(e.payload)).then((u) => { unlistenDone = u; });
    void listen<string>("llm:error", (e) => store.setOutput(`Error: ${e.payload}`)).then((u) => { unlistenErr = u; });
    return () => { unlistenToken?.(); unlistenDone?.(); unlistenErr?.(); };
  }, [store.appendOutput, store.setOutput]);

  const tabs = useMemo(() => [
    { id: "palette" as const, label: "Palette" },
    { id: "memory" as const, label: "Memory" },
    { id: "history" as const, label: "History" },
    { id: "settings" as const, label: "Settings" },
    { id: "onboarding" as const, label: "Onboarding" },
  ], []);

  const orbV2Enabled = useMemo(() => {
    if (store.startupDiagnostics) return store.startupDiagnostics.orbV2Enabled;
    if (typeof window === "undefined") return false;
    return window.localStorage.getItem("neph.orb_v2_enabled") === "1";
  }, [store.startupDiagnostics]);

  return (
    <main
      className="palette-shell"
      onKeyDown={(e) => {
        if (e.key === "Escape") store.hidePalette();
        if (e.ctrlKey && e.key.toLowerCase() === "z") { e.preventDefault(); store.runUndo(); }
      }}
    >
      {/* Orb v1 — amplitude-reactive, 5-state colour machine (Blueprint §3) */}
      <div className="orb-container">
        {orbV2Enabled ? <OrbCanvasV2 /> : <OrbCanvas />}
        <TranscriptStrip />
        <LatencyBadge />
      </div>

      <div className="palette-card">
        {/* Tab navigation */}
        <div className="tab-row">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              className={`tab-pill ${store.activeTab === tab.id ? "active" : ""}`}
              onClick={() => store.setActiveTab(tab.id)}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {/* ── Tab panels (Blueprint §4 — each domain has its own directory) ── */}

        {store.activeTab === "palette" && (
          <PalettePanel
            input={store.input}
            output={store.output}
            isLoading={store.isLoading}
            isOffline={store.isOffline}
            isRateLimited={store.isRateLimited}
            errorMessage={store.errorMessage}
            pendingConfirmation={store.pendingConfirmation}
            redConfirmText={store.redConfirmText}
            setInput={store.setInput}
            setRedConfirmText={store.setRedConfirmText}
            runCommand={store.runCommand}
            confirmPendingCommand={store.confirmPendingCommand}
            cancelPendingCommand={store.cancelPendingCommand}
          />
        )}

        {store.activeTab === "memory" && (
          <MemoryInspector
            memoryItems={store.memoryItems}
            admissionItems={store.admissionItems}
            memoryQuery={store.memoryQuery}
            startupDiagnostics={store.startupDiagnostics}
            setMemoryQuery={store.setMemoryQuery}
            refreshMemory={store.refreshMemory}
            keepAdmission={store.keepAdmission}
            discardAdmission={store.discardAdmission}
            updateMemory={store.updateMemory}
            toggleMemoryPin={store.toggleMemoryPin}
            deleteMemory={store.deleteMemory}
          />
        )}

        {store.activeTab === "history" && (
          <HistoryPanel
            history={store.history}
            refreshHistory={store.refreshHistory}
          />
        )}

        {store.activeTab === "settings" && (
          <SettingsPanel
            settingsSection={store.settingsSection}
            provider={store.provider}
            apiKey={store.apiKey}
            hotkeySpec={store.hotkeySpec}
            hotkeySaveMessage={store.hotkeySaveMessage}
            startupDiagnostics={store.startupDiagnostics}
            providerTestResult={store.providerTestResult}
            tokenStats={store.tokenStats}
            setSettingsSection={store.setSettingsSection}
            setProvider={store.setProvider}
            setApiKey={store.setApiKey}
            setHotkeySpec={store.setHotkeySpec}
            saveProviderKey={store.saveProviderKey}
            testProvider={store.testProvider}
            refreshTokenStats={store.refreshTokenStats}
            exportDbBackup={store.exportDbBackup}
            exportMemoryJson={store.exportMemoryJson}
            openReportIssue={store.openReportIssue}
            saveHotkeyToRust={store.saveHotkeyToRust}
            exportDiagnosticBundle={store.exportDiagnosticBundle}
            setDpapiExportSetting={store.setDpapiExportSetting}
          />
        )}

        {store.activeTab === "onboarding" && (
          <div className="panel">
            <h3>Onboarding</h3>
            {!store.onboardingSkipped ? (
              <>
                <div className="output-row">Step {store.onboardingStep} of 3</div>
                {store.onboardingStep === 1 && (
                  <div className="output-row">
                    Open the palette with your configured shortcut (default Ctrl+Space). If you use an East Asian IME
                    and nothing happens, set Ctrl+Shift+Space or Alt+Space under Settings → General and restart Neph.
                    Ensure WebView2 Evergreen is installed if the window stays blank (see Settings → About).
                  </div>
                )}
                {store.onboardingStep === 2 && <div className="output-row">Add your provider key in Settings &gt; AI</div>}
                {store.onboardingStep === 3 && <div className="output-row">Index folders and run your first command.</div>}
                <div className="memory-row">
                  <button className="secondary-btn" onClick={store.nextOnboardingStep}>Next</button>
                  <button className="secondary-btn" onClick={store.skipOnboarding}>Skip</button>
                </div>
              </>
            ) : (
              <div className="output-row">Onboarding skipped. Re-open anytime from this tab.</div>
            )}
          </div>
        )}

        <div className="trust-footer">
          Local | Provider: {store.provider} | Latency: {store.lastLatencyMs}ms | Tokens in/out: {store.tokenStats.input}/{store.tokenStats.output} | Keys: Credential Manager
        </div>
      </div>
    </main>
  );
}

export default App;

