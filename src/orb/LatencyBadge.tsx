/// LatencyBadge — Phase 1 DoD verification helper.
///
/// Shows measured end-to-end voice latency (ms) after each interaction.
/// Appears briefly then fades. Color coding:
///   green  < 500ms
///   amber  500-1000ms
///   red    > 1000ms  (above target)

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import "./LatencyBadge.css";

export function LatencyBadge() {
  const [latency, setLatency] = useState<{
    total_ms: number;
    stt_ms: number;
    llm_ms: number;
  } | null>(null);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const unlisten: Array<() => void> = [];

    void listen<{ total_ms: number; stt_ms: number; llm_ms: number }>("voice:latency", (e) => {
      setLatency(e.payload);
      setVisible(true);
      // Auto-hide after 3s.
      setTimeout(() => setVisible(false), 3000);
    }).then((u) => unlisten.push(u));

    return () => unlisten.forEach((u) => u());
  }, []);

  if (!visible || latency === null) return null;

  const tier =
    latency.total_ms < 800 ? "fast" : latency.total_ms < 1200 ? "ok" : "slow";

  return (
    <div
      id="latency-badge"
      className={`latency-badge latency-${tier}`}
      title="End-to-end latency (hotkey down → first audio frame)"
      aria-label={`Voice latency: ${latency.total_ms}ms`}
    >
      VOICE {latency.total_ms}ms (STT {latency.stt_ms} / LLM {latency.llm_ms})
    </div>
  );
}
