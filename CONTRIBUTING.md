# Contributing to Project Neph

Solo or small-team changes should keep the **trust kernel** honest: no new yellow/red paths without backend confirmation tests.

## Before you open a PR

1. `npm run typecheck`
2. `npm run verify` (or at minimum: `cd src-tauri; cargo clippy --all-targets -- -D warnings; cargo test`)
3. If you add a Tauri command or event: update **`src/lib/bindings.ts`** (and read [`docs/IPC_BINDINGS.md`](docs/IPC_BINDINGS.md)).
4. If you add a tool: declare it in **`apps/desktop/tools.toml`** first, then wire Rust (`state/runner`, `tools/schema`, etc.).

## Product direction

- **[`Blueprint.md`](Blueprint.md)** — architecture and phased roadmap.
- **[`docs/BLUEPRINT_STATUS.md`](docs/BLUEPRINT_STATUS.md)** — what is done vs planned.

## Scope

Large or risky features (new providers, autonomous file agents, MCP transport) should have a short **ADR** under `docs/ADRs/` or a tracked issue before landing on `main`.

## Changelog

User-visible behavior changes belong in **[`CHANGELOG.md`](CHANGELOG.md)** under `[Unreleased]`.
