/**
 * src/palette/PalettePanel.tsx
 *
 * Blueprint §4: apps/desktop/src/palette/ — power-user palette surface.
 * Extracted from monolithic App.tsx to keep each domain under its own file.
 */

import type { PendingConfirmation } from "../state/paletteStore";

interface PalettePanelProps {
  input: string;
  output: string;
  isLoading: boolean;
  isOffline: boolean;
  isRateLimited: boolean;
  errorMessage: string | null;
  pendingConfirmation: PendingConfirmation | null;
  redConfirmText: string;
  setInput: (v: string) => void;
  setRedConfirmText: (v: string) => void;
  runCommand: () => void;
  confirmPendingCommand: () => void;
  cancelPendingCommand: () => void;
}

export function PalettePanel({
  input,
  output,
  isLoading,
  isOffline,
  isRateLimited,
  errorMessage,
  pendingConfirmation,
  redConfirmText,
  setInput,
  setRedConfirmText,
  runCommand,
  confirmPendingCommand,
  cancelPendingCommand,
}: PalettePanelProps) {
  return (
    <div className="panel">
      <input
        className="palette-input"
        autoFocus
        placeholder="Try: >app, >find, >note, >notes, >findnote"
        aria-label="Command input"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && runCommand()}
      />
      <div className="output-row">{output}</div>
      <div className="memory-row">
        <button className="secondary-btn" onClick={() => { setInput(">ctx"); void runCommand(); }}>Context</button>
        <button className="secondary-btn" onClick={() => { setInput(">snip"); void runCommand(); }}>Screenshot</button>
        <button className="secondary-btn" onClick={() => { setInput(">voice"); void runCommand(); }}>Voice</button>
        <button className="secondary-btn" onClick={() => { setInput(">dailybrief"); void runCommand(); }}>Daily Brief</button>
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
            <button className="secondary-btn" onClick={confirmPendingCommand}>Confirm</button>
            <button className="secondary-btn" onClick={cancelPendingCommand}>Cancel</button>
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
            onChange={(e) => setRedConfirmText(e.target.value)}
          />
          <div className="memory-row">
            <button className="secondary-btn" onClick={confirmPendingCommand}>Confirm red action</button>
            <button className="secondary-btn" onClick={cancelPendingCommand}>Cancel</button>
          </div>
        </div>
      )}
    </div>
  );
}
