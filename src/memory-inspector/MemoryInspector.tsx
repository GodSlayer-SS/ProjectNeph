/**
 * src/memory-inspector/MemoryInspector.tsx
 *
 * Blueprint §4: apps/desktop/src/memory-inspector/
 * Extracted from monolithic App.tsx.
 */

import { useState } from "react";
import type { MemoryItem, AdmissionItem, StartupDiagnostics } from "../state/paletteStore";


interface MemoryInspectorProps {
  memoryItems: MemoryItem[];
  admissionItems: AdmissionItem[];
  memoryQuery: string;
  startupDiagnostics: StartupDiagnostics | null;

  setMemoryQuery: (v: string) => void;
  refreshMemory: () => void;
  keepAdmission: (id: number) => void;
  discardAdmission: (id: number) => void;
  updateMemory: (id: number, content: string) => Promise<void>;
  toggleMemoryPin: (id: number, pinned: boolean) => void;
  deleteMemory: (id: number) => void;
}

export function MemoryInspector({
  memoryItems,
  admissionItems,
  memoryQuery,
  startupDiagnostics,
  setMemoryQuery,
  refreshMemory,
  keepAdmission,
  discardAdmission,
  updateMemory,
  toggleMemoryPin,
  deleteMemory,
}: MemoryInspectorProps) {
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editingValue, setEditingValue] = useState("");

  return (
    <div className="panel">
      <h3>Memory Editor</h3>

      {admissionItems.length > 0 && (
        <div className="output-row">
          <strong>Admission queue</strong>
          <ul className="history-list">
            {admissionItems.map((item) => (
              <li key={`admission-${item.id}`}>
                <div>#{item.id} [{item.kind ?? "unknown"}] {item.score != null ? `score ${item.score}` : ""}</div>
                <div>{item.content}</div>
                <div className="memory-row">
                  <button className="secondary-btn" onClick={() => keepAdmission(item.id)}>Keep → Memory</button>
                  <button className="secondary-btn" onClick={() => discardAdmission(item.id)}>Discard</button>
                </div>
              </li>
            ))}
          </ul>
        </div>
      )}

      {startupDiagnostics && !startupDiagnostics.vectorSearchEnabled && (
        <div className="output-row">
          Vector search: disabled (sqlite-vec did not load). Mode: {startupDiagnostics.embeddingMode ?? "unknown"}.
        </div>
      )}

      <div className="memory-controls">
        <input
          className="palette-input"
          placeholder="Search memory..."
          value={memoryQuery}
          onChange={(e) => setMemoryQuery(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && refreshMemory()}
        />
        <button className="secondary-btn" onClick={refreshMemory}>Search</button>
      </div>

      <ul className="history-list">
        {memoryItems.length === 0 && <li>No memory items yet. Use &gt;remember to add one.</li>}
        {memoryItems.map((item) => (
          <li key={item.id}>
            <span>#{item.id} [{item.kind}] {item.pinned ? "PINNED" : ""}</span>
            {editingId === item.id ? (
              <div className="memory-row">
                <input
                  className="palette-input"
                  value={editingValue}
                  onChange={(e) => setEditingValue(e.target.value)}
                />
                <button
                  className="secondary-btn"
                  onClick={async () => {
                    await updateMemory(item.id, editingValue);
                    setEditingId(null);
                    setEditingValue("");
                  }}
                >Save</button>
                <button className="secondary-btn" onClick={() => { setEditingId(null); setEditingValue(""); }}>Cancel</button>
              </div>
            ) : (
              <>
                <span>{item.content}</span>
                <div className="memory-row">
                  <button className="secondary-btn" onClick={() => { setEditingId(item.id); setEditingValue(item.content); }}>Edit</button>
                  <button className="secondary-btn" onClick={() => toggleMemoryPin(item.id, !item.pinned)}>{item.pinned ? "Unpin" : "Pin"}</button>
                  <button className="secondary-btn" onClick={() => deleteMemory(item.id)}>Delete</button>
                </div>
              </>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}
