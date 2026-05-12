/**
 * src/history/HistoryPanel.tsx
 *
 * Blueprint §4 — history / audit log panel.
 * Previously inline in App.tsx. Extracted to keep App.tsx under 150 lines.
 */

import { useMemo, useState } from "react";
import type { HistoryItem } from "../state/paletteStore";

interface HistoryPanelProps {
  history: HistoryItem[];
  refreshHistory: () => void;
}

type HistoryFilter = "all" | "success" | "failed" | "red";

export function HistoryPanel({ history, refreshHistory }: HistoryPanelProps) {
  const [filter, setFilter] = useState<HistoryFilter>("all");
  const [expandedId, setExpandedId] = useState<number | null>(null);

  const filtered = useMemo(() => {
    return history.filter((item) => {
      if (filter === "success") return item.success === true;
      if (filter === "failed") return item.success === false;
      if (filter === "red") return item.risk_level === "red";
      return true;
    });
  }, [history, filter]);

  return (
    <div className="panel">
      <h3>History / Audit</h3>
      <div className="memory-row">
        <button className="secondary-btn" onClick={refreshHistory}>Refresh</button>
        <select value={filter} onChange={(e) => setFilter(e.target.value as HistoryFilter)}>
          <option value="all">All</option>
          <option value="success">Success</option>
          <option value="failed">Failed</option>
          <option value="red">Red Risk</option>
        </select>
      </div>
      <ul className="history-list">
        {filtered.length === 0 && <li>No history entries for this filter.</li>}
        {filtered.map((item) => (
          <li key={item.id}>
            <span>{item.intent} [{item.risk_level ?? "green"}] {item.success === false ? "FAILED" : "OK"}</span>
            <span>{item.input}</span>
            <button
              className="secondary-btn"
              onClick={() => setExpandedId(expandedId === item.id ? null : item.id)}
            >
              {expandedId === item.id ? "Hide details" : "Show details"}
            </button>
            {expandedId === item.id && (
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
  );
}
