# Cursor workflow (Project Neph)

Solo work benefits from a strict editor habit so context windows stay honest and trust-sensitive code is not "half-remembered."

## Session reset

- **New chat** for a new task (or a clean thread after a long detour) when the work is trust/security/schema related.
- Paste `.cursor/context/INVARIANTS.md` or `.cursor/context/ARCHITECTURE.md` when the model needs grounding.

## Two-strike rule

1. If the first implementation attempt is wrong, **re-read the relevant Rust module and the IPC bindings** and try once more with a smaller diff.
2. If it is still wrong, **stop and write down** what you observed (error, file, expected behavior) before changing more code. This avoids thrash in `state/`, `risk`, and `secrets`.

## What not to do

- Do not "quick fix" file paths, secrets, or confirmation semantics without a test or an explicit follow-up in `CHANGELOG` / `SECURITY`.
- Do not add large unrelated refactors in the same PR as a bugfix unless the plan calls for it.

## Verification before merge

- `npm run typecheck`
- `cargo clippy` / `cargo test` for any Rust change touching commands or policy.
- Prefer **`npm run verify`** for a bounded, end-to-end check (per-step timeouts; avoids silent PowerShell buffering—see `CONTRIBUTING.md`).
