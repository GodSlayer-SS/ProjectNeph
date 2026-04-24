# External “Master Audit” blueprint vs this repository (April 2026)

This document **reconciles** the long-form audit/blueprint the founder pasted into Cursor with **what Neph actually ships today**, so future agents do not repeat outdated findings.

## Verdict: which audit claims still hold?

| Audit claim | Verdict today | Evidence |
|-------------|----------------|----------|
| **H1 — “Risk gate is frontend-only; red runs without consent”** | **Outdated / incorrect** for current code. | `src-tauri/src/state/runner.rs`: yellow/red paths require `ConfirmationStore::issue` / `consume(token, plan_hash)` on the **Rust** path before `execute_plan`. `paletteStore.ts` only **forwards** the server-issued token; it cannot mint a valid one. |
| **Red “type DELETE”** | UX only, not the security boundary. | Frontend blocks the second IPC call until the user types `DELETE`; backend still requires the **token** issued with the dry-run `plan_hash`. |
| **C2 — “Fake MiniLM / silent embedding lie”** | **Partially addressed.** | Embedding mode is **`stub_hash_v01`** with explicit `embeddingMode` + `vectorSearchEnabled` in startup diagnostics and `docs/EMBEDDING.md`. Hybrid/stub vectors apply **only** when sqlite-vec loads; UI can show vector disabled. Remaining gap: semantic quality is not transformer-grade until fastembed (deferred). |
| **C1 — Updater placeholder pubkey** | **Mitigated for alpha.** | `tauri.conf.json`: `plugins.updater.active: false`, `createUpdaterArtifacts: false`. |
| **C3 — Path traversal / naive paths** | **Mostly implemented; keep auditing.** | `path_policy.rs` + `SafePathPolicy` used from file mutation paths; `trash` for delete. Continue to review every new file tool. |
| **Prompt injection → yellow/red** | **Mitigated.** | `runner.rs`: `LlmClassify` + `privileged_mutation_risk` → **rejected** unless user uses explicit `>` prefixes. |
| **A1 — `risk.rs` map vs Tool trait** | **Half true.** | Standalone `risk.rs` was removed; risk is still a **central** `tool_risk()` match in `tools.rs`, not per-tool trait methods. Drift risk remains if new tools forget to update the match. |
| **A2 — TOCTOU plan vs execute** | **Residual design tension.** | `ExecutionPlan` is built once; args validated; `plan_hash` binds the token. Stronger guarantee would be “execute only accepts an opaque plan handle” (future hardening). |
| **WAL never checkpointed** | **Outdated.** | Periodic `wal_checkpoint(TRUNCATE)` thread in `lib.rs`. |
| **No sqlite-vec failure surface** | **Outdated.** | `sqlite_vec_loaded` setting + diagnostics + Memory UI. |
| **No .cursorrules / no context files** | **Outdated for this repo.** | Present under `.cursor/` and `docs/` per week-0 plan. |
| **README vs CHANGELOG contradiction** | **Improved but watch drift.** | README is alpha-realistic; CHANGELOG `[Unreleased]` is long — roadmap v2 asks for periodic **release slicing** (see `NEPH_TRUST_ROADMAP_v2.md`). |
| **Provider cut to Groq + OpenAI only** | **Not implemented** (product choice). | UI + `secrets.rs` still allow Anthropic and OpenRouter; Ollama removed per `docs/FUTURE.md`. Align or document intentionally. |
| **24h triage SLA** | **Outdated.** | `TRIAGE.md` uses **72h** solo-maintainer language. |

## Health score (revised, repo-grounded)

Rough **7.5 / 10** for **alpha trust plumbing** (backend token, path policy, injection gate, WAL, diagnostics, verify pipeline), **not** for “public stranger-ready v1.0.” Remaining major gaps: optional real embeddings, SignPath/updater for signed releases, clean-VM proof, provider surface discipline, and sustained doc hygiene.

## How to use this file

- When an external audit says “frontend-only gate,” **open `runner.rs` first** and cite this reconcile doc.
- When planning work, use **`NEPH_TRUST_ROADMAP_v2.md`** next-step checkboxes instead of re-deriving from the long blueprint alone.
