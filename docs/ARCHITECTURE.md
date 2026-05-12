# Neph architecture (Blueprint v2 aligned)

Neph is a **Windows-first** desktop app: **Tauri 2** hosts a **React 19** UI and a **Rust** core in one process. Optional processes: a **Python** sidecar (ML/audio) over a named pipe, and a **Node** sidecar (Playwright) over a second named pipe for browser automation.

## Layers

| Layer | Location | Role |
|-------|----------|------|
| UI | `src/` | Palette, chat/history, memory inspector, settings, orb (WebGL v1 / gated v2) |
| IPC surface | `src-tauri/src/ipc/` | Tauri commands/events, `pyside`, `nodeside`, Groq STT helper |
| Core | `src-tauri/src/` | Routing, tools, LLM providers, SQLite + migrations, secrets (`keyring`) |
| Trait wall | `src-tauri/src/traits/` | `LlmProvider`, `SttProvider`, `TtsProvider`, `Embedder`, `MemoryStore`, `Tool`, `ExecutionDomain`, `Planner` |
| Actors | `src-tauri/src/actors/` | Hotkey, voice, planner, executor, automation helpers, provider router, UI bridge |
| Domains | `src-tauri/src/domains/` | Filesystem, network egress, browser profile handles, shell tiers |
| Memory | `src-tauri/src/memory/` | Hot (session), warm (SQLite), cold (LanceDB), admission, procedural |
| Providers | `src-tauri/src/providers/` | Gemini, Anthropic, Groq, OpenRouter, local llama (optional) |
| Tool manifest | `apps/desktop/tools.toml` | Names, risk, domain, egress, phase — loaded at startup |

## Typical request flow

1. User opens palette or completes **push-to-talk** voice capture.
2. Frontend invokes `run_palette_command` (or voice path emits `stt:final` → same pipeline).
3. Rust **router** classifies deterministic `>` commands vs free text.
4. **Trust kernel**: yellow/red mutations require valid **confirmation token** bound to **plan hash** and TTL.
5. Tools execute through **`state/runner`** with **domain** checks (`tools.toml` domain + network allowlist).
6. LLM streaming emits `llm:token` / `llm:done` / `llm:error` to the webview.

## Storage and secrets

- **SQLite** (WAL) under `%LOCALAPPDATA%\Neph\` — notes, memory rows, audit, settings.
- **Provider keys** — Windows Credential Manager via `keyring`, not committed to disk in plaintext.
- **LanceDB** — cold-tier vectors alongside embedding pipeline (`memory/cold.rs`).

## Sidecars

- **Python** — STT/TTS/VAD/embeddings; started by the app when voice/ML features need it (see `ipc/pyside.rs`, `scripts/install_pyside.py`).
- **Node + Playwright** — must be started manually for browser tools: `node apps/nodeside/server.js` after `setup_nodeside.ps1`.

## Further reading

- [`Blueprint.md`](../Blueprint.md) — full roadmap and security model (browser profiles, path domains, mantra).
- [`BLUEPRINT_STATUS.md`](BLUEPRINT_STATUS.md) — implementation checklist vs phases.
- [`SECURITY.md`](SECURITY.md) — trust rules and data handling.
