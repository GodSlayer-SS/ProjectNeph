# Template: refactor module

**Target:** (path)  
**Why now:** (reduce LOC / clarify ownership / unblock tests)

## Rules

- No behavior change unless called out in “Intentional changes”.
- Run full `cargo test` for Rust refactors; `typecheck` for TS moves.

## Plan

1. Map public API of module.
2. Move code to new files; keep re-exports stable.
3. Delete dead code in a follow-up if needed (separate commit).

## Intentional changes

(Empty or bullet list)
