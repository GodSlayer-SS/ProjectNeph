# Blueprint implementation status

This file maps [`Blueprint.md`](../Blueprint.md) (Final Blueprint v2) to the **current repository**. It is refreshed when major phases land.

## Architecture invariants (Blueprint §0–2)

| Item | Status |
|------|--------|
| Modular monolith, single Rust process (Tauri) | Yes |
| Python ML sidecar (named pipe) | Yes (`ipc/pyside`, `apps/pyside`) |
| Node Playwright sidecar (Phase 3) | Yes (`apps/nodeside`, optional at runtime) |
| Eight Rust traits (`src-tauri/src/traits/`) | Yes |
| `apps/desktop/tools.toml` manifest | Yes |
| Cloud-primary LLM routing | Yes (`providers/`, `model_router`) |

## Phase 1 — Spine

| Item | Status |
|------|--------|
| Voice loop (PTT, STT, stream LLM, TTS) | Yes (`actors/voice`, Groq STT primary + sidecar fallback) |
| Orb v1 (WebGL, states, amplitude) | Yes (`src/orb/OrbCanvas.tsx`) |
| Palette / trust kernel for writes | Yes |

## Phase 2 — Brain

| Item | Status |
|------|--------|
| Structured planner + schema validation | Yes (`actors/planner`, `tools/schema`) |
| Executor + confirmation token / plan hash / TTL | Yes (`actors/executor`, `state/runner`) |
| SQLite warm memory + LanceDB cold tier | Yes (`memory/warm`, `memory/cold`, `state/memory`) |
| Admission / distill pass | Yes (`memory/admission`, UI queue) |
| Memory inspector UI | Yes (`src/memory-inspector/`) |
| Hybrid recall: FTS5 + ANN + Python reranker | **Partial** — lexical + sqlite-vec + Lance merge; dedicated memory FTS5 + bge-reranker sidecar hookup not complete |
| Local Qwen / llama.cpp | Optional path only (`providers/local_llama.rs`), per blueprint “measure first” |

## Phase 3 — Hands

| Item | Status |
|------|--------|
| Four isolated Chromium profiles | Yes (`apps/nodeside/server.js`) |
| Browser tools wired to nodeside | Yes |
| Desktop tools (`focus_window`, `type_in_active`, `read_active`) | Yes — Windows **WScript.Shell** via PowerShell (`actors/automation` + `runner`); blueprint’s **uiautomation + enigo** called out as a future swap-in |
| File organizer template | Yes |
| Code companion / diff flow | Yes (`code_companion_diff`, confirmation-gated) |

## Phase 4 — Reach

| Item | Status |
|------|--------|
| Wake-word toggle / scaffold | Settings flag + tool; **no** full always-listening engine |
| MCP server (stdio / Claude Desktop) | **Stub** — `mcp/server.rs` config only; `rmcp` transport not wired |
| MCP client | **Stub** — `mcp/client.rs` |
| Orb v2 (WebGPU path) | Toggle + `OrbCanvasV2` scaffold |
| ElevenLabs TTS | Provider stub |
| Discord / WhatsApp | Not started (per blueprint: defer) |
| Procedural skill library | Partial (`skills.rs`, `list_skills` / `run_skill`, warm-backed procedures) |

When in doubt, **`Blueprint.md`** is the source of truth for intent; this file describes **what shipped in git today**.
