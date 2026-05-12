import { useMemo } from "react";
import { useOrbStore } from "../state/orbStore";

/**
 * Orb v2 skeleton (Phase 4): keep this lightweight and behind toggle.
 * Full R3F/WebGPU implementation can replace this internal rendering later.
 */
export function OrbCanvasV2() {
  const { state, amplitude } = useOrbStore();
  const glow = useMemo(() => {
    const base = 0.22 + amplitude * 0.45;
    if (state === "error") return "rgba(255, 90, 90, 0.9)";
    if (state === "speaking") return "rgba(120, 190, 255, 0.95)";
    if (state === "thinking") return "rgba(190, 130, 255, 0.95)";
    if (state === "listening" || state === "transcribing") return "rgba(120, 255, 210, 0.95)";
    return `rgba(145, 165, 255, ${base})`;
  }, [state, amplitude]);

  return (
    <div
      aria-label="Orb v2 skeleton"
      style={{
        width: 140,
        height: 140,
        borderRadius: "999px",
        background:
          "radial-gradient(circle at 35% 35%, rgba(255,255,255,0.22) 0%, rgba(255,255,255,0.04) 35%, rgba(10,12,30,0.8) 80%)",
        boxShadow: `0 0 0 1px rgba(255,255,255,0.06), 0 0 36px ${glow}`,
        transition: "box-shadow 120ms linear",
      }}
    />
  );
}
