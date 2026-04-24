# Contributing to Neph

## Development
- Install Node.js LTS and Rust stable.
- Run `npm install`.
- Run `npm run tauri dev`.

## Quality Gates
- Frontend typecheck: `npm run typecheck`
- Rust lint (warnings fail CI): `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml`
- UI smoke (built bundle via Vite preview, not the Tauri shell): `npm run build` then `npx playwright test`
- **One-shot full gate (timeouts + fresh `CARGO_TARGET_DIR`):** `npm run verify` (runs `scripts/verify.ps1`: LOC, typecheck, clippy `-Dwarnings`, `cargo check`, `cargo test`, `npm run build`, Playwright install+test, `tauri build --debug`). Faster loops: `powershell -File scripts/verify.ps1 -SkipE2E` and/or `-SkipTauriBuild`.

### PowerShell / Cursor: “terminal runs forever with no output”
- **Do not** pipe live `cargo`/`rustc` output through `Select-Object -Last N` (or similar). PowerShell buffers the whole stream until the child exits, so the UI stays blank while work is still running—or while blocked on a **file lock** (`Blocking waiting for file lock on build directory`).
- Prefer: run `cargo` **without** that pipe, or use `npm run verify`, or stream with `ForEach-Object { Write-Host $_ }`.
- If cargo is wedged: stop other terminals/IDE builds using the same `target` dir, or set a unique `CARGO_TARGET_DIR` (the verify script does this automatically).

## Scope Rules
- Follow milestone order from the implementation plan.
- Do not bypass risk confirmations for red actions.
- Keep BYOK and local-first defaults intact.

## Cursor / agents
- Read `docs/CURSOR_WORKFLOW.md` and `.cursorrules` before large edits.
- Before trust/security refactors, read **`docs/BLUEPRINT_VS_REPO_2026-04.md`** (corrects common stale audit claims) and **`docs/NEPH_TRUST_ROADMAP_v2.md`** (current prioritized checklist).
- Context files: `.cursor/context/INVARIANTS.md`, `ARCHITECTURE.md`, `GLOSSARY.md`.
- Prompt templates: `.cursor/prompts/` (build, debug, audit, refactor, performance, tests, release).

## IPC bindings
- See `docs/IPC_BINDINGS.md`: new Tauri commands must update `src/lib/bindings.ts` in the same change until generated bindings land.
