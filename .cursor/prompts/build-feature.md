# Template: build feature

## Goal

(One sentence: what the user can do after this work.)

## Constraints

- Trust / scope: (link plan item or "none")
- Migrations: yes/no
- New IPC: yes/no

## Steps

1. Read `INVARIANTS` + affected modules.
2. Add smallest backend change + tests.
3. Wire frontend + bindings (`bindings.ts` until specta is default).
4. Update `CHANGELOG` under `Unreleased`.
5. `npm run typecheck` and `cargo test` as applicable.

## Done when

(Checkbox list: behavior, tests, docs.)
