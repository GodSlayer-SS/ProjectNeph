# Changelog

All notable changes to this project are recorded in this file.

## [Unreleased]

### Added

- **Docs:** `docs/BLUEPRINT_VS_REPO_2026-04.md` (external audit claims vs current code) and `docs/NEPH_TRUST_ROADMAP_v2.md` (prioritized next steps). Plan file `neph_trust-hardening_plan_6fd5acb6` updated with `ops-blueprint-reconcile-2026-04`.
- **Test:** `state::runner::token_gate_tests::yellow_save_memory_requires_backend_token` — proves yellow `save_memory` requires a valid backend confirmation token (refutes outdated “frontend-only gate” narrative).

- **Startup cold-start budgets**: log `target_ms` alongside measured `ms` for hotkey infra and first LLM completion; **`tracing::warn!`** when over **500ms** / **1500ms** (advisory; see trust plan week-4 metrics).

- `docs/LAUNCH_CHECKLIST.md`: engineering pre-launch items marked done (install docs, changelog, CI/verify, Week 4 WebView2 script); marketing/post-launch rows unchanged.

- **Week-3 / Week-4 retrieval and ops hardening**: FTS5 `file_index_fts` (migration `V6__file_index_fts.sql`) with Rust backfill; `search_files` uses `MATCH` when FTS exists; `initialize_database` returns `DbInitMeta` (`sqlite_vec_loaded`, `embedding_mode`); recovery for interrupted `executing` actions → `failed`; memory recall **half-life decay** and **stub semantic** only when sqlite-vec loads (model id `neph-stub-hash-v0.1`).
- **LLM token accounting** from provider JSON (`usage` / Anthropic `input_tokens`/`output_tokens`); classify path returns token counts for telemetry.
- **Periodic WAL checkpoint** thread (`PRAGMA wal_checkpoint(TRUNCATE)` every 30s); startup timing hooks (`record_setup_start`, `log_palette_infra_ready`, `log_first_llm_completion_if_needed`).
- **Diagnostics**: `export_diagnostic_bundle` (ZIP with `report.json` + recent `neph*` logs); `get_startup_diagnostics` extended with `vectorSearchEnabled`, `embeddingMode`, `dpapiProtectExports`.
- **Optional DPAPI-protected memory exports** (Windows): `get_dpapi_protect_exports` / `set_dpapi_protect_exports`; Settings → Privacy UI + Memory tab banner when vector is off.
- **Router eval tests** (`intent_router_eval`): large deterministic prefix suite with pass-rate gate.
- Docs: `docs/EMBEDDING.md`, `docs/RELEASE_NOTES_v0.1.0-alpha.md`, **`docs/WEEK4_CLEAN_VM.md`** and **`scripts/week4-clean-vm.ps1`** (WebView2 registry preflight for cold-install checks).
- **Playwright** smoke test (`e2e/smoke.spec.ts`) against **`vite preview`** after production build; `@playwright/test` dev dependency; `npm run test:e2e`.
- **`scripts/verify.ps1`** is now **[1/8]–[8/8]**: clippy with **`-Dwarnings`**, Playwright install+test, **`npm run tauri build -- --debug`**. Optional **`-SkipE2E`** / **`-SkipTauriBuild`** for faster local loops.

- **Week-2 Windows trust / install hardening**: WebView2 Evergreen **registry check** at startup (warn below minimum), **NSIS `embedBootstrapper`**, language selector on; **rolling daily tracing logs** under `%LOCALAPPDATA%\Neph\logs`; **palette hotkey presets** persisted in SQLite (`settings.palette_hotkey`) with Settings UI + `get_palette_hotkey` / `set_palette_hotkey` / `get_startup_diagnostics` commands.
- **OneDrive / cloud placeholder** guard before move/rename/delete; clearer **access denied** hint for Defender Controlled folder access.
- DB migrations **`V4__drop_workflows.sql`**, **`V5__drop_logs_table.sql`** (logs move to files; workflows deferred — see `docs/FUTURE.md`).
- Unicode path policy test (tempdir).
- `docs/FUTURE.md` for deferred scope (Ollama, overwrite command, workflows, plugins).
- Week-1 trust execution path: immutable `ExecutionPlan`, SHA-256 `planHash`, backend **confirmation tokens** (one-shot, TTL 60s) for all yellow/red tools; `run_palette_command` returns structured `PaletteRunResponse` (`completed` / `needConfirmation` / `rejected`).
- `SafePathPolicy` for file mutations (allowlisted roots, block system locations, traversal hygiene).
- **LLM prompt-injection gate**: model-routed intents cannot execute yellow/red tools.
- **Tool args schema validation** (`schemars` + `serde_json` deserialize) and **per-tool redaction** for persisted args / lineage; crash logs redacted via `telemetry`.
- **IPC rate limiting**: sliding window (120/min) on palette command execution.
- `command_history` **provenance** + **lineage_json** (`V3__lineage.sql` migration).
- `tracing` + `tracing-subscriber` with `neph_cmd` target; command input logged redacted.
- Unit tests: confirmation token binding, rate limit cap, path policy matrix, plan hash stability, secrets empty/disallowed provider, tool arg redaction.
- `scripts/verify.ps1` and **`npm run verify`**: per-step timeouts (`.NET WaitForExit`), fresh `CARGO_TARGET_DIR` per run, `cmd.exe /c` for npm on Windows (`npm.ps1` is not a valid `Start-Process` target), and docs explaining why `cargo | Select-Object -Last` looks “stuck” with no output.
- `.cursorrules` and `.cursor/context/*` (architecture, invariants, glossary) for agent-safe editing.
- `.cursor/prompts/` templates: build, debug, security audit, refactor, performance, tests, release.
- `.cursorignore` to keep heavy/build artifacts out of default Cursor context.
- `docs/CURSOR_WORKFLOW.md` (session reset, two-strike rule) and `docs/CONTRIBUTING.md` cross-links.
- `docs/SUSTAINABILITY.md` (weekend maintenance cadence), `docs/COMPAT.md` (Windows matrix stub), `docs/IPC_BINDINGS.md` (specta policy until codegen), `docs/SIGNING.md` (SignPath / signing honesty).
- `scripts/check-rust-line-length.ps1` and a CI step to fail if any `src-tauri/src/**/*.rs` file exceeds 500 lines.

### Changed

- Root **`index.html` `<title>`** set to **Neph** (was generic Tauri template text).

- **CI** (`.github/workflows/ci.yml`): `npm run build` before Rust; **Playwright** Chromium install + smoke test; **clippy `-- -D warnings`**; order aligned with local verify.
- Removed unused **`models::Note`** struct (notes use DB row shapes elsewhere).

- **`V1__init.sql`**: removed `PRAGMA journal_mode` / `foreign_keys` / `synchronous` from the migration body (they ran inside refinery’s migration transaction and could fail with “cannot change into wal mode from within a transaction” on fresh DBs). The same pragmas are applied in Rust immediately after migrations complete. Refinery runs with `set_abort_divergent(false)` so existing databases that already recorded the old V1 checksum keep working without a manual repair.

- **Alpha updater**: `plugins.updater.active` and `bundle.createUpdaterArtifacts` set to **false** until a real signing pubkey and endpoint are configured; removed unused `tauri-plugin-updater` crate dependency.
- Removed **`>overwritefile` / `overwrite_file`** from the command surface, tools, and router (undo still applies legacy `overwrite` payloads from older DB rows).
- Removed **Ollama** from provider allowlist, LLM bridge, and Settings UI (deferred per `docs/FUTURE.md`).
- `docs/INSTALL_WINDOWS.md`: SmartScreen, AV, Controlled folder access, ONNX note, WebView2, IME hotkey troubleshooting.
- Frontend palette flow uses backend-driven confirmation (dry-run **preview** text from Rust); removed prefix-only gating before invoke.
- Consolidated tool **risk** authority in `tools.rs` (removed standalone `risk.rs`).
- Undo remains **JSON payload–based** in `actions.undo_payload` (move/overwrite); a future pass may introduce a fully typed `Tool::undo_plan -> ExecutionPlan` layer without changing user-visible behavior.
- README: explicit [MIT](LICENSE) link, **alpha** status, honest known limits, document index expanded.
- `CHANGELOG`: split `0.1.0-alpha.0` (historical batch) from `[Unreleased]`; alpha snapshot called out in the release block.
- `TRIAGE.md`: **72h** classification target and solo maintainer / no-paid-SLA language.
- `BETA_PLAN.md`: **3 → 10 → 20** staged rollout and gate language.
- `CONTRIBUTING.md`: Cursor context, prompt templates, and IPC binding rules.
- Split backend `state` into focused modules (notes, memory, file ops, command runner, exports, LLM bridge, index) to keep ownership boundaries clear and files under the line-count guard.

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

- None in this tag (scope cuts tracked for future releases; see `docs/V0_2_PLAN.md`).

### Fixed

- Fixed Rust build errors in Tauri setup by replacing non-Send manager state registration and normalizing setup error conversion.
- Re-verified current implementation with passing `npm run typecheck` and `cargo check`.
- Re-verified post-LLM/security batch with passing `npm run typecheck` and `cargo check`.
- Re-verified risk/confirmation/file-action/sqlite-vec batch with passing `npm run typecheck` and `cargo check`.
- Re-verified embeddings/hybrid/backup/ux/onboarding/settings batch with passing `npm run typecheck` and `cargo check`.
- Completed full verification sweep with passing `npm run typecheck`, `cargo check`, `cargo test`, and `npm run build`.

[Unreleased]: https://github.com/mohit/Project-Neph/compare/v0.1.0-alpha.0...HEAD
[0.1.0-alpha.0]: https://github.com/mohit/Project-Neph/releases/tag/v0.1.0-alpha.0
