# IPC bindings policy

## Goal

- **Type-safe, drift-resistant** Tauri command bindings between Rust and TypeScript.
- **Authoritative** API surface: Rust commands and shared DTOs.

## Current state (alpha)

- TypeScript still uses a **manually maintained** `src/lib/bindings.ts` plus `invokeTyped` helpers.
- This is **acceptable only during alpha**; it is a known source of drift (renamed arg, new command) if not paired with review.

## Direction (tauri-specta)

- Adopt **[tauri-specta](https://crates.io/crates/tauri-specta)** (and `#[specta::specta]` on commands) so TypeScript (or a checked stub) is **generated** from the same definitions as the Rust `invoke` handler.
- **Gate:** new commands and public result types should either (a) be added to the generated export in the same change, or (b) come with a **tracked exception** in this file until generation is wired.

## Rules until generation is the default

1. **Any** new `#[tauri::command]` must update `src/lib/bindings.ts` in the **same** PR.
2. Prefer named argument types in Rust and mirror them in TS.
3. Do not duplicate “magic strings” for command names in the UI; centralize in one map (`bindings` / store).

## Generated output (when enabled)

- Commit generated output (or enforce regeneration in CI) so PRs show binding diffs.
- Do not hand-edit the generated file except through re-running the export.

This document satisfies the “specta policy” week-0 gate until full codegen lands.
