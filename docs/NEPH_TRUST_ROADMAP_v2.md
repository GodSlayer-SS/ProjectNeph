# Neph trust roadmap v2 (repo-grounded)

Supersedes informal “do everything in Unreleased” ordering. Use together with `docs/BLUEPRINT_VS_REPO_2026-04.md` and the Cursor plan `neph_trust-hardening_plan_6fd5acb6`.

## Principles

1. **Trust invariants live in Rust** — UI is transport + friction, not authorization.
2. **Truthful product surface** — no marketing language that implies MiniLM/transformer embeddings until shipped.
3. **Release slices** — periodically cut `[Unreleased]` into dated sections so scope is legible.

---

## Phase A — Shipment truth (this sprint)

- [ ] **A1** Run **`wk4-clean-vm-install`**: NSIS build on a clean Windows VM; capture failures; tick plan item when done.
- [ ] **A2** README / CHANGELOG: replace dead **`FULL_AUDIT_REPORT.md`** links (use `docs/SECURITY.md`, `docs/AUDIT_CHECKLIST.md`, `docs/BLUEPRINT_VS_REPO_2026-04.md`).
- [ ] **A3** Decide **provider surface**: either (a) remove Anthropic + OpenRouter from Settings + `secrets` allowlist + `llm.rs`, or (b) document “v0.1 allows four BYOK providers” explicitly in README + SECURITY (pick one; blueprint preferred **a**).
- [x] **A4** Rust test `yellow_save_memory_requires_backend_token` (`state/runner.rs`): bogus token rejected; valid server token completes save.

## Phase B — Trust hardening (next)

- [ ] **B1** **Central risk registry audit**: grep for new tools; ensure `tool_risk` + `validate_tool_schema` + router cover each; consider codegen or macro to reduce drift (optional).
- [ ] **B2** **TOCTOU tightening** (optional): thread opaque plan id from `issue` through `execute` so args cannot be swapped between preview and confirm without invalidating the token.
- [ ] **B3** **SignPath** (`wk0-signpath-apply`): external; keep pending until filed.

## Phase C — Embeddings (post–truthful-stub)

- [ ] **C1** Either ship **fastembed + bundled ONNX** + migration, or explicitly keep stub through v0.1.x and point all docs to one paragraph (no mixed signals).
- [ ] **C2** Retrieval eval harness when semantic ships (reuse intent-eval pattern).

## Phase D — Beta ops (human)

- [ ] **D1** Supervised **3-user** alpha (`BETA_PLAN.md`).
- [ ] **D2** Expand to 10 / 20 only with triage metrics.

---

## Done (reference — do not reopen unless regression)

Backend `ConfirmationToken` + `plan_hash`; LLM cannot start yellow/red; `SafePathPolicy`; sqlite-vec flag + UI; FTS file search; WAL checkpoint; DPAPI memory export + verify; diagnostic ZIP; Playwright smoke + verify script; startup SLA log+warn; intent router eval ≥300; TRIAGE 72h; updater disabled for alpha.
