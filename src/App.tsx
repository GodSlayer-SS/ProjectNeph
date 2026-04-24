import { useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import { usePaletteStore } from "./stores/paletteStore";

function App() {
  const {
    activeTab,
    input,
    output,
    history,
    memoryItems,
    memoryQuery,
    provider,
    apiKey,
    pendingConfirmation,
    redConfirmText,
    isLoading,
    isOffline,
    isRateLimited,
    errorMessage,
    lastLatencyMs,
    onboardingStep,
    onboardingSkipped,
    settingsSection,
    providerTestResult,
    tokenStats,
    hotkeySpec,
    hotkeySaveMessage,
    startupDiagnostics,
    setActiveTab,
    setInput,
    setOutput,
    appendOutput,
    setProvider,
    setApiKey,
    setMemoryQuery,
    setRedConfirmText,
    setSettingsSection,
    nextOnboardingStep,
    skipOnboarding,
    hidePalette,
    runCommand,
    confirmPendingCommand,
    cancelPendingCommand,
    runUndo,
    refreshHistory,
    saveProviderKey,
    refreshMemory,
    updateMemory,
    toggleMemoryPin,
    deleteMemory,
    testProvider,
    refreshTokenStats,
    exportDbBackup,
    exportMemoryJson,
    openReportIssue,
    setHotkeySpec,
    loadHotkeyFromRust,
    saveHotkeyToRust,
    loadStartupDiagnostics,
    exportDiagnosticBundle,
    setDpapiExportSetting,
  } = usePaletteStore();
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editingValue, setEditingValue] = useState("");
  const [historyFilter, setHistoryFilter] = useState<"all" | "success" | "failed" | "red">("all");
  const [expandedHistoryId, setExpandedHistoryId] = useState<number | null>(null);

  useEffect(() => {
    if (activeTab === "settings" && settingsSection === "general") {
      void loadHotkeyFromRust();
    }
  }, [activeTab, settingsSection, loadHotkeyFromRust]);

  useEffect(() => {
    if (
      activeTab === "memory" ||
      (activeTab === "settings" && (settingsSection === "about" || settingsSection === "privacy"))
    ) {
      void loadStartupDiagnostics();
    }
  }, [activeTab, settingsSection, loadStartupDiagnostics]);

  useEffect(() => {
    let unlistenToken: (() => void) | null = null;
    let unlistenDone: (() => void) | null = null;
    let unlistenErr: (() => void) | null = null;
    void listen<string>("llm:token", (event) => {
      appendOutput(event.payload);
    }).then((u) => {
      unlistenToken = u;
    });
    void listen<string>("llm:done", (event) => {
      setOutput(event.payload);
    }).then((u) => {
      unlistenDone = u;
    });
    void listen<string>("llm:error", (event) => {
      setOutput(`Error: ${event.payload}`);
    }).then((u) => {
      unlistenErr = u;
    });
    return () => {
      unlistenToken?.();
      unlistenDone?.();
      unlistenErr?.();
    };
  }, [appendOutput, setOutput]);

  const tabs = useMemo(
    () => [
      { id: "palette" as const, label: "Palette" },
      { id: "memory" as const, label: "Memory" },
      { id: "history" as const, label: "History" },
      { id: "settings" as const, label: "Settings" },
      { id: "onboarding" as const, label: "Onboarding" },
    ],
    [],
  );
  const filteredHistory = useMemo(() => {
    return history.filter((item) => {
      if (historyFilter === "success") {
        return item.success === true;
      }
      if (historyFilter === "failed") {
        return item.success === false;
      }
      if (historyFilter === "red") {
        return item.risk_level === "red";
      }
      return true;
    });
  }, [history, historyFilter]);

  return (
    <main
      className="palette-shell"
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          hidePalette();
        }
        if (event.ctrlKey && event.key.toLowerCase() === "z") {
          event.preventDefault();
          runUndo();
        }
      }}
    >
      <div className="palette-card">
        <div className="tab-row">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              className={`tab-pill ${activeTab === tab.id ? "active" : ""}`}
              onClick={() => setActiveTab(tab.id)}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {activeTab === "palette" && (
          <div className="panel">
            <input
              className="palette-input"
              autoFocus
              placeholder="Try: >app, >find, >note, >notes, >findnote"
              aria-label="Command input"
              value={input}
              onChange={(event) => setInput(event.target.value)}
              onKeyDown={(event) => event.key === "Enter" && runCommand()}
            />
            <div className="output-row">{output}</div>
            <div className="memory-row">
              <button
                className="secondary-btn"
                onClick={() => {
                  setInput(">ctx");
                  void runCommand();
                }}
              >
                Context
              </button>
              <button
                className="secondary-btn"
                onClick={() => {
                  setInput(">snip");
                  void runCommand();
                }}
              >
                Screenshot
              </button>
              <button
                className="secondary-btn"
                onClick={() => {
                  setInput(">voice");
                  void runCommand();
                }}
              >
                Voice
              </button>
              <button
                className="secondary-btn"
                onClick={() => {
                  setInput(">dailybrief");
                  void runCommand();
                }}
              >
                Daily Brief
              </button>
            </div>
            {isLoading && <div className="output-row">Loading...</div>}
            {isOffline && <div className="output-row">Offline mode - AI features unavailable.</div>}
            {isRateLimited && <div className="output-row">Provider rate limited. Please retry shortly.</div>}
            {errorMessage && <div className="output-row">Error: {errorMessage}</div>}
            {pendingConfirmation?.risk === "yellow" && (
              <div className="confirm-card">
                <div className="confirm-preview">{pendingConfirmation.preview}</div>
                <div className="confirm-meta">Command: `{pendingConfirmation.input}`</div>
                <div className="memory-row">
                  <button className="secondary-btn" onClick={confirmPendingCommand}>
                    Confirm
                  </button>
                  <button className="secondary-btn" onClick={cancelPendingCommand}>
                    Cancel
                  </button>
                </div>
              </div>
            )}
            {pendingConfirmation?.risk === "red" && (
              <div className="confirm-modal">
                <h4>High risk action</h4>
                <p className="confirm-preview">{pendingConfirmation.preview}</p>
                <p className="confirm-meta">{pendingConfirmation.input}</p>
                <p>Type DELETE to confirm.</p>
                <input
                  className="palette-input"
                  value={redConfirmText}
                  onChange={(event) => setRedConfirmText(event.target.value)}
                />
                <div className="memory-row">
                  <button className="secondary-btn" onClick={confirmPendingCommand}>
                    Confirm red action
                  </button>
                  <button className="secondary-btn" onClick={cancelPendingCommand}>
                    Cancel
                  </button>
                </div>
              </div>
            )}
          </div>
        )}

        {activeTab === "memory" && (
          <div className="panel">
            <h3>Memory Editor</h3>
            {startupDiagnostics && !startupDiagnostics.vectorSearchEnabled && (
              <div className="output-row">
                Vector search: disabled (sqlite-vec did not load). Recall uses keyword match plus optional stub
                similarity only when the extension is available. Mode: {startupDiagnostics.embeddingMode ?? "unknown"}.
              </div>
            )}
            <div className="memory-controls">
              <input
                className="palette-input"
                placeholder="Search memory..."
                value={memoryQuery}
                onChange={(event) => setMemoryQuery(event.target.value)}
                onKeyDown={(event) => event.key === "Enter" && refreshMemory()}
              />
              <button className="secondary-btn" onClick={refreshMemory}>
                Search
              </button>
            </div>
            <ul className="history-list">
              {memoryItems.length === 0 && <li>No memory items yet. Use &gt;remember to add one.</li>}
              {memoryItems.map((item) => (
                <li key={item.id}>
                  <span>
                    #{item.id} [{item.kind}] {item.pinned ? "PINNED" : ""}
                  </span>
                  {editingId === item.id ? (
                    <div className="memory-row">
                      <input
                        className="palette-input"
                        value={editingValue}
                        onChange={(event) => setEditingValue(event.target.value)}
                      />
                      <button
                        className="secondary-btn"
                        onClick={async () => {
                          await updateMemory(item.id, editingValue);
                          setEditingId(null);
                          setEditingValue("");
                        }}
                      >
                        Save
                      </button>
                      <button
                        className="secondary-btn"
                        onClick={() => {
                          setEditingId(null);
                          setEditingValue("");
                        }}
                      >
                        Cancel
                      </button>
                    </div>
                  ) : (
                    <>
                      <span>{item.content}</span>
                      <div className="memory-row">
                        <button
                          className="secondary-btn"
                          onClick={() => {
                            setEditingId(item.id);
                            setEditingValue(item.content);
                          }}
                        >
                          Edit
                        </button>
                        <button
                          className="secondary-btn"
                          onClick={() => toggleMemoryPin(item.id, !item.pinned)}
                        >
                          {item.pinned ? "Unpin" : "Pin"}
                        </button>
                        <button className="secondary-btn" onClick={() => deleteMemory(item.id)}>
                          Delete
                        </button>
                      </div>
                    </>
                  )}
                </li>
              ))}
            </ul>
          </div>
        )}

        {activeTab === "history" && (
          <div className="panel">
            <h3>History / Audit</h3>
            <div className="memory-row">
              <button className="secondary-btn" onClick={refreshHistory}>
                Refresh
              </button>
              <select value={historyFilter} onChange={(event) => setHistoryFilter(event.target.value as typeof historyFilter)}>
                <option value="all">All</option>
                <option value="success">Success</option>
                <option value="failed">Failed</option>
                <option value="red">Red Risk</option>
              </select>
            </div>
            <ul className="history-list">
              {filteredHistory.length === 0 && <li>No history entries for this filter.</li>}
              {filteredHistory.map((item) => (
                <li key={item.id}>
                  <span>
                    {item.intent} [{item.risk_level ?? "green"}] {item.success === false ? "FAILED" : "OK"}
                  </span>
                  <span>{item.input}</span>
                  <button
                    className="secondary-btn"
                    onClick={() => setExpandedHistoryId(expandedHistoryId === item.id ? null : item.id)}
                  >
                    {expandedHistoryId === item.id ? "Hide details" : "Show details"}
                  </button>
                  {expandedHistoryId === item.id && (
                    <div className="output-row">
                      <div>Tool: {item.tool_name ?? "n/a"}</div>
                      <div>Provenance: {item.provenance ?? "n/a"}</div>
                      <div>State: {item.state ?? "n/a"}</div>
                      <div>Args: {item.args_json ?? "{}"}</div>
                      <div>Lineage: {item.lineage_json ?? "-"}</div>
                      <div>Result: {item.result_summary ?? "-"}</div>
                    </div>
                  )}
                </li>
              ))}
            </ul>
          </div>
        )}

        {activeTab === "settings" && (
          <div className="panel">
            <h3>Settings</h3>
            <div className="memory-row">
              {(["general", "ai", "privacy", "memory", "folders", "about"] as const).map((section) => (
                <button key={section} className="tab-pill" onClick={() => setSettingsSection(section)}>
                  {section}
                </button>
              ))}
            </div>
            {settingsSection === "general" && (
              <div className="settings-grid">
                <div className="output-row">
                  Palette global shortcut (restart Neph after change). If Ctrl+Space conflicts with your IME, use
                  Ctrl+Shift+Space or Alt+Space.
                </div>
                <label className="output-row" htmlFor="hotkey-spec">
                  Hotkey
                </label>
                <select
                  id="hotkey-spec"
                  value={hotkeySpec}
                  onChange={(event) => setHotkeySpec(event.target.value)}
                >
                  <option value="ctrl+space">Ctrl+Space</option>
                  <option value="ctrl+shift+space">Ctrl+Shift+Space</option>
                  <option value="alt+space">Alt+Space</option>
                </select>
                <div className="memory-row">
                  <button className="secondary-btn" type="button" onClick={() => void saveHotkeyToRust()}>
                    Save hotkey
                  </button>
                </div>
                {hotkeySaveMessage && <div className="output-row">{hotkeySaveMessage}</div>}
              </div>
            )}
            {settingsSection === "ai" && (
              <div className="settings-grid">
                <select value={provider} onChange={(event) => setProvider(event.target.value)}>
                  <option value="groq">Groq</option>
                  <option value="gemini">Gemini</option>
                  <option value="openrouter">OpenRouter</option>
                </select>
                <input
                  className="palette-input"
                  value={apiKey}
                  onChange={(event) => setApiKey(event.target.value)}
                  placeholder="Paste API key"
                />
                <div className="memory-row">
                  <button className="secondary-btn" onClick={saveProviderKey}>
                    Save Key
                  </button>
                  <button className="secondary-btn" onClick={testProvider}>
                    Test Provider
                  </button>
                  <button className="secondary-btn" onClick={refreshTokenStats}>
                    Refresh Token Stats
                  </button>
                </div>
                {providerTestResult && <div className="output-row">{providerTestResult}</div>}
                <div className="output-row">
                  Session tokens: in {tokenStats.input} / out {tokenStats.output}
                </div>
              </div>
            )}
            {settingsSection === "privacy" && (
              <div className="settings-grid">
                <div className="output-row">
                  Export memory JSON with Windows DPAPI (current user). Files use a <code>.json.dpapi</code> extension and
                  can only be read on the same machine/profile.
                </div>
                <label className="output-row">
                  <input
                    type="checkbox"
                    checked={startupDiagnostics?.dpapiProtectExports ?? false}
                    onChange={(event) => void setDpapiExportSetting(event.target.checked)}
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
                <button className="secondary-btn" onClick={exportMemoryJson}>
                  Export Memory JSON
                </button>
                <button className="secondary-btn" onClick={exportDbBackup}>
                  Export DB Backup
                </button>
              </div>
            )}
            {settingsSection === "folders" && <div className="output-row">Indexed folders: Documents, Desktop, Downloads (editable in upcoming pass).</div>}
            {settingsSection === "about" && <div className="output-row">Neph v0.1.0-dev</div>}
            {settingsSection === "about" && startupDiagnostics && (
              <div className="output-row">
                <div>Configured hotkey: {startupDiagnostics.paletteHotkey}</div>
                <div>
                  WebView2:{" "}
                  {startupDiagnostics.webview2Version ??
                    "not detected in registry (install Evergreen if the window is blank)"}
                  {startupDiagnostics.webview2Version && (
                    <>
                      {" "}
                      — meets minimum {startupDiagnostics.webview2Minimum}:{" "}
                      {startupDiagnostics.webview2MeetsMinimum ? "yes" : "no"}
                    </>
                  )}
                </div>
                {!startupDiagnostics.webview2MeetsMinimum && (
                  <div>
                    Install or update:{" "}
                    <a href={startupDiagnostics.webview2InstallUrl} target="_blank" rel="noreferrer">
                      WebView2 runtime
                    </a>
                  </div>
                )}
                <div>{startupDiagnostics.imeHotkeyTip}</div>
                <div>
                  Embedding: {startupDiagnostics.embeddingMode ?? "n/a"} — vector extension active:{" "}
                  {startupDiagnostics.vectorSearchEnabled ? "yes" : "no"}
                </div>
              </div>
            )}
            {settingsSection === "about" && (
              <button className="secondary-btn" onClick={openReportIssue}>
                Report Issue
              </button>
            )}
          </div>
        )}

        {activeTab === "onboarding" && (
          <div className="panel">
            <h3>Onboarding</h3>
            {!onboardingSkipped ? (
              <>
                <div className="output-row">Step {onboardingStep} of 3</div>
                {onboardingStep === 1 && (
                  <div className="output-row">
                    Open the palette with your configured shortcut (default Ctrl+Space). If you use an East Asian IME
                    and nothing happens, set Ctrl+Shift+Space or Alt+Space under Settings → General and restart Neph.
                    Ensure WebView2 Evergreen is installed if the window stays blank (see Settings → About).
                  </div>
                )}
                {onboardingStep === 2 && <div className="output-row">Add your provider key in Settings &gt; AI</div>}
                {onboardingStep === 3 && <div className="output-row">Index folders and run your first command.</div>}
                <div className="memory-row">
                  <button className="secondary-btn" onClick={nextOnboardingStep}>
                    Next
                  </button>
                  <button className="secondary-btn" onClick={skipOnboarding}>
                    Skip
                  </button>
                </div>
              </>
            ) : (
              <div className="output-row">Onboarding skipped. Re-open anytime from this tab.</div>
            )}
          </div>
        )}
        <div className="trust-footer">
          Local | Provider: {provider} | Latency: {lastLatencyMs}ms | Tokens in/out: {tokenStats.input}/{tokenStats.output} | Keys: Credential Manager
        </div>
      </div>
    </main>
  );
}

export default App;
