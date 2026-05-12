# packages/orb/

**Purpose**: Orb v2 shaders, state machine, and R3F components (Blueprint §9, Phase 4).

## Phase Status

**Phase 4** — Orb v2 is behind a feature toggle. Orb v1 (`src/orb/OrbCanvas.tsx`) is the active renderer.

## What Goes Here

- `shaders/` — WebGPU WGSL shaders (ring, particle system, bloom)
- `state-machine.ts` — typed orb state machine (idle → listening → thinking → speaking → error)
- `OrbV2.tsx` — R3F component tree (Phase 4)
- `audio-reactive.ts` — audio amplitude processing utilities

## Orb v1 vs v2

| Feature | Orb v1 (active) | Orb v2 (Phase 4) |
|---|---|---|
| Renderer | WebGL2 | WebGPU + R3F |
| LOC | ~200 | ~800 |
| Dependencies | none | `@react-three/fiber`, `@react-three/drei` |
| Particles | no | yes |
| Bloom | no | yes |
| Enable via | always | `toggle_orb_v2` tool |

## Toggle

The Orb v2 can be enabled via:
```
>toggle_orb_v2 enabled=true
```
This writes `orb_v2_enabled=1` to the settings table and the `App.tsx` reads it to switch renderers.

## References

- Blueprint §9 Phase 4: "Orb v2 with WebGPU + R3F shaders"
- `src/orb/OrbCanvas.tsx` — active v1 implementation
- `src/orb/OrbCanvasV2.tsx` — Phase 4 skeleton
