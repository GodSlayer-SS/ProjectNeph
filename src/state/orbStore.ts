/// orbStore — Zustand state store for the Orb WebGL ring.
///
/// State machine:
///   idle ──[hotkey down]──► listening
///   listening ──[stt:final]──► thinking
///   thinking ──[llm:done]──► speaking (if sidecar running) | idle
///   speaking ──[voice:state idle]──► idle
///   * ──[llm:error]──► error ──[2s]──► idle
///
/// All state transitions driven by Tauri events from the Rust side.
/// The `voice:state` event is the canonical source of truth for the orb.

import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";

export type OrbState =
  | "idle"
  | "listening"
  | "transcribing"
  | "thinking"
  | "speaking"
  | "error";

interface OrbStore {
  state: OrbState;
  amplitude: number;
  setState: (s: OrbState) => void;
  setAmplitude: (a: number) => void;
}

export const useOrbStore = create<OrbStore>((set) => ({
  state: "idle",
  amplitude: 0,
  setState: (s) => set({ state: s }),
  setAmplitude: (a) => set({ amplitude: Math.max(0, Math.min(1, a)) }),
}));

/**
 * Wire Tauri events → orb store.
 * Call this once at app startup (in App.tsx useEffect).
 */
export function initOrbListeners() {
  // The Rust VoiceActor / lib.rs emits `voice:state` for all state changes.
  // This is the canonical event — we just mirror it directly.
  void listen<OrbState>("voice:state", (e) => {
    useOrbStore.getState().setState(e.payload);
  });

  // Amplitude from WASAPI mic capture.
  void listen<{ value: number }>("voice:amplitude", (e) => {
    useOrbStore.getState().setAmplitude(e.payload.value);
  });

  // First LLM token → thinking (in case voice:state missed it).
  void listen<string>("llm:token", () => {
    const s = useOrbStore.getState().state;
    if (s === "listening" || s === "transcribing" || s === "idle") {
      useOrbStore.getState().setState("thinking");
    }
  });

  // stt:partial → listening (belt + suspenders with voice:state).
  void listen<{ text: string }>("stt:partial", () => {
    if (useOrbStore.getState().state !== "listening") {
      useOrbStore.getState().setState("listening");
    }
  });

  // Error state — auto-reset to idle after 3s.
  void listen<string>("llm:error", () => {
    useOrbStore.getState().setState("error");
    setTimeout(() => {
      if (useOrbStore.getState().state === "error") {
        useOrbStore.getState().setState("idle");
      }
    }, 3000);
  });
}
