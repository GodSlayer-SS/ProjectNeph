/// TranscriptStrip — shows live STT partial transcripts and the final transcript
/// while the user is speaking. Fades out 2s after the LLM response arrives.
///
/// Positioned above the OrbCanvas inside the orb-container.

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import "./TranscriptStrip.css";

interface Props {
  className?: string;
}

export function TranscriptStrip({ className = "" }: Props) {
  const [text, setText] = useState("");
  const [visible, setVisible] = useState(false);
  const [fadeTimer, setFadeTimer] = useState<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    const unlisten: Array<() => void> = [];

    void listen<{ text: string }>("stt:partial", (e) => {
      setText(e.payload.text);
      setVisible(true);
      if (fadeTimer) clearTimeout(fadeTimer);
    }).then((u) => unlisten.push(u));

    void listen<{ text: string }>("stt:final", (e) => {
      setText(e.payload.text);
      setVisible(true);
      if (fadeTimer) clearTimeout(fadeTimer);
    }).then((u) => unlisten.push(u));

    void listen<string>("llm:done", () => {
      // Fade out 2s after response is ready.
      const t = setTimeout(() => {
        setVisible(false);
        setText("");
      }, 2000);
      setFadeTimer(t);
    }).then((u) => unlisten.push(u));

    void listen<string>("llm:error", () => {
      setVisible(false);
      setText("");
    }).then((u) => unlisten.push(u));

    return () => {
      unlisten.forEach((u) => u());
      if (fadeTimer) clearTimeout(fadeTimer);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (!visible || !text) return null;

  return (
    <div
      id="transcript-strip"
      className={`transcript-strip ${className}`}
      aria-live="polite"
      aria-label="Live transcript"
    >
      {text}
    </div>
  );
}
