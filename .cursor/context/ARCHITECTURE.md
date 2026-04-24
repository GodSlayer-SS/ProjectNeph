# Architecture snapshot (Neph)

- **Shell:** Tauri 2, single hidden window toggled by global `Ctrl+Space` (`lib.rs` + `global-hotkey`).
- **Frontend:** React, Zustand store (`src/stores/paletteStore.ts`), typed invoke via `src/lib/ipc.ts` and `src/lib/bindings.ts`.
- **Backend:** Rust `AppState` in `src-tauri/src/state/` (split modules: `command`, `memory`, `notes`, `file_ops`, `exports`, `llm_bridge`, `index`).
- **Persistence:** SQLite in the user data dir, migrations in `src-tauri/src/db/migrations/`.
- **LLM:** `llm` module with pluggable providers; `secrets` uses Windows keyring; intent routing via `router` + optional LLM classification in `llm_bridge`.
- **Risk / undo:** `risk` tool → string level; `actions` + `command_history` for audit; file undo JSON payloads in `file_ops` / `command`.

This file is a **map**, not a spec. Deeper design lives in `docs/ARCHITECTURE.md`.
