# Changelog

All notable changes to this project are recorded in this file.

## [Unreleased]

### Added

- **`Blueprint.md`** at repo root: Nephis Council Round 2 definitive architecture and phased roadmap.
- **`docs/SETUP.md`**: Windows developer install checklist (Git, MSVC Build Tools, Rust, Node 20+, Python 3.11+, WebView2).
- **`docs/BLUEPRINT_STATUS.md`**: implementation status vs blueprint phases.
- **Nephis architecture spine**: `src-tauri/src/traits/`, `actors/`, `domains/`, `memory/` (hot/warm/cold LanceDB, admission), `providers/`, `ipc/` (pyside, nodeside, Groq STT), `mcp/` stubs, `bus.rs`.
- **Python ML sidecar** (`apps/pyside/`) with `scripts/install_pyside.py`; **Node Playwright sidecar** (`apps/nodeside/`) with `scripts/setup_nodeside.ps1`.
- **Voice pipeline** (push-to-talk, barge-in, streaming STT/LLM/TTS) and **orb** UI (WebGL v1, gated v2).
- **`apps/desktop/tools.toml`**: versioned tool manifest with domain + egress metadata.

### Changed

- **README** and **`docs/ARCHITECTURE.md`** rewritten around Blueprint v2 and sidecars.
- **`docs/INSTALL_WINDOWS.md`**: links to SETUP + BLUEPRINT_STATUS; browser profile table retained.
- **`docs/SUSTAINABILITY.md`** / **`docs/FUTURE.md`**: scope pointers updated; local LLM deferred per blueprint measurement rule.
- **`actors/automation.rs`**: desktop tool helpers shared with `state/runner` (Windows WScript.Shell path).

### Removed

- **`archive/pre-pivot/`** tree, obsolete planning docs **`docs/V0_2_PLAN.md`**, **`docs/NEPH_TRUST_ROADMAP_v2.md`**.
- Root **Playwright** test harness: **`playwright.config.ts`**, **`e2e/smoke.spec.ts`**, and root **`playwright`** / **`@playwright/test`** npm dependencies (browser automation remains under **`apps/nodeside`** only).
- **Repo-local editor / CI scaffolding:** **`.cursor/`**, **`.cursorignore`**, **`.cursorrules`**, **`.github/`**, **`.vscode/`**, and **`docs/CURSOR_WORKFLOW.md`**. These are no longer tracked so you can recreate them locally as needed; quality gates are **`npm run verify`** and manual **`cargo`** runs.

### Fixed

- **`scripts/verify.ps1`**: four steps (line-length guard, typecheck, clippy `-D warnings`, cargo test) after e2e removal.
- Stale module comments for LanceDB / automation stubs aligned with current code.

## [0.1.0-alpha.0] - 2026-01-15

> Alpha snapshot: functionality exists for local/supervised testing. Trust/confirmation semantics, embeddings, and updater signing are not release-complete (see `docs/SECURITY.md` and `docs/AUDIT_CHECKLIST.md`).

### Added

- Initialized Day 1 Neph command palette foundation in the Tauri app.
- Registered a global `Ctrl+Space` hotkey in Rust to toggle the main overlay window.
- Added a Tauri `hide_palette` command and wired `Esc` in the frontend to hide the overlay.
- Added `global-hotkey` dependency for global shortcut support.
- Added a root CI workflow at `.github/workflows/ci.yml` for push/PR validation on Windows.
- Added a frontend `typecheck` script in `package.json` for CI and local verification.
- Installed local development prerequisites for Day 1 verification: Node.js LTS, npm, Rust (`cargo`/`rustc`), and project npm dependencies.
- Added typed frontend command bindings at `src/lib/bindings.ts` and a typed invoke wrapper at `src/lib/ipc.ts`.
- Added Zustand state management via `src/stores/paletteStore.ts` and integrated it into the palette UI.
- Added app indexing and launching baseline with `>scanapps` and `>app <name>` in `src-tauri/src/apps.rs`.
- Added fallback file indexing and search with `>scanfiles` and `>find <name>` in `src-tauri/src/files.rs`.
- Expanded baseline database schema in `V1__init.sql` with settings/users, notes FTS + triggers, memory metadata, logs, file/app index, workflows, and supporting indexes.
- Added note command prefixes for CRUD/read paths: `>notes`, `>findnote`, `>updatenote`, and `>deletenote`.
- Added Memory Editor APIs (`get_memory`, `update_memory_item`, `toggle_memory_pin`, `delete_memory_item`) and wired them through Tauri invoke handlers.
- Added Memory Editor UI capabilities for search, inline edit, pin/unpin, and soft-delete actions.
- Added OpenAI/Groq BYOK provider implementations using a shared `LlmProvider` abstraction in `src-tauri/src/llm.rs`.
- Added strict JSON intent classification with one retry/fallback path for unknown natural-language inputs.
- Added LLM-powered `>summarize` and `>rewrite` commands with usage estimates in command output.
- Added richer history payload fields (risk/state/result/args) to support actionable audit drill-down.
- Added provider allowlist validation for credential read/write operations in `src-tauri/src/secrets.rs`.
- Added file mutation commands and parsing for `>movefile`, `>renamefile`, `>deletefile`, `>overwritefile`, and `>undo`.
- Added backend undo payload persistence and `undo_action` execution path.
- Added sqlite vector-stage migration `V2__embeddings.sql` plus best-effort sqlite-vec extension loading and virtual table creation.
- Added `trash` crate integration so delete actions route to Recycle Bin.
- Added embedding pipeline scaffolding with content-hash based re-embed checks and memory vector upserts.
- Added backup/export commands for DB and memory JSON (`export_db_backup`, `export_memory_json`).
- Added provider test command (`test_provider`) surfaced to settings.
- Added onboarding stepper controls (next/skip) and settings section model in app state.
- Added performance notes document at `docs/PERFORMANCE.md`.
- Added updater plugin wiring and provider/token stats commands (`set_active_provider`, `get_token_stats`, `report_issue_link`).
- Added installation and launch operations documents: `docs/INSTALL_WINDOWS.md`, `docs/BETA_PLAN.md`, `docs/TRIAGE.md`, `docs/AUDIT_CHECKLIST.md`, `docs/LAUNCH_CHECKLIST.md`, `docs/V0_2_PLAN.md`.
- Added GitHub Pages landing page source at `docs/site/index.html` and deploy workflow at `.github/workflows/pages.yml`.
- Added screenshots asset placeholder docs at `docs/screenshots/README.md`.
- **MIT** `LICENSE` at repository root; link from README (trust-hardening / week-0 hygiene).

### Changed

- Replaced the default Tauri starter UI with a minimal command palette shell and input.
- Updated Tauri window config for palette behavior: fixed-size, borderless, always-on-top, hidden by default, and skipped from the taskbar.
- Updated app styling to a dark overlay palette layout focused on low-latency command entry.
- Began installing Visual Studio Build Tools C++ workload to satisfy the Rust MSVC linker requirement (`link.exe`) for `tauri dev` on Windows.
- Refactored `src/App.tsx` to use centralized state/actions instead of direct invoke calls in component logic.
- Extended router/risk/state command handling to support app/file indexing flows and execution responses.
- Improved app launch ranking to prioritize exact matches, prefix matches, frequent usage, and recent launches.
- Extended note operations to include FTS-based search and soft-delete-aware update/list behaviors.
- Expanded shared command bindings/store state to include typed memory item operations and data models.
- Expanded history panel UI with filters (all/success/failed/red) and expandable action details.
- Updated command execution persistence to record tool args and latency in `command_history`.
- Hardened Tauri CSP to explicit self/API origins instead of permissive null policy.
- Expanded risk classification coverage to include undo-safe command routing and destructive file operations.
- Added yellow inline confirmation card and red modal confirmation UX with typed DELETE guard for red actions.
- Added keyboard undo trigger (`Ctrl+Z`) in palette flow, wired to backend `>undo`.
- Upgraded memory recall to hybrid retrieval (lexical + embedding cosine) over bounded candidates.
- Added command intent cache (last 20 commands) to reduce repeated routing overhead.
- Expanded settings UI into sections: general, AI, privacy, memory, folders, about.
- Added loading/offline/rate-limit/error UI states and trust footer telemetry line in the palette.
- Added animation/micro-interaction polish for palette and list rows.
- Implemented Anthropic, OpenRouter, and Ollama providers under the shared `LlmProvider` interface.
- Added provider fallback chain selection and session token accounting surfaced in Settings.
- Configured NSIS bundle target and updater artifact generation in `tauri.conf.json`.
- Updated release workflow to upload updater-related artifacts.
- Expanded README links and release asset guidance for launch readiness; README now states alpha status and known limits.

### Removed

- None in this tag (scope cuts tracked for future releases; see `docs/FUTURE.md` and `Blueprint.md`).

### Fixed

- Fixed Rust build errors in Tauri setup by replacing non-Send manager state registration and normalizing setup error conversion.
- Re-verified current implementation with passing `npm run typecheck` and `cargo check`.
- Re-verified post-LLM/security batch with passing `npm run typecheck` and `cargo check`.
- Re-verified risk/confirmation/file-action/sqlite-vec batch with passing `npm run typecheck` and `cargo check`.
- Re-verified embeddings/hybrid/backup/ux/onboarding/settings batch with passing `npm run typecheck` and `cargo check`.
- Completed full verification sweep with passing `npm run typecheck`, `cargo check`, `cargo test`, and `npm run build`.

[Unreleased]: https://github.com/GodSlayer-SS/ProjectNeph/compare/v0.1.0-alpha.0...HEAD
[0.1.0-alpha.0]: https://github.com/GodSlayer-SS/ProjectNeph/releases/tag/v0.1.0-alpha.0
