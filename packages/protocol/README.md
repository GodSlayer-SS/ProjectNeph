# packages/protocol/

**Purpose**: Shared TypeScript types generated from Rust structs via `ts-rs` or `specta`.

## Phase Status

**Phase 3+** — Type codegen is deferred until the API surface stabilises.

## What Goes Here

When activated, this package will contain:
- Auto-generated TypeScript types for all Tauri command payloads
- Shared enums (RiskLevel, MemoryTier, VoiceState, etc.)
- Zod schemas for runtime validation

## Generation Command (Future)

```bash
cd src-tauri
cargo test --test ts_export   # generates ../packages/protocol/src/generated.ts
```

## References

- Blueprint §4: `packages/protocol/ — ts-rs / specta generated types`
- [ts-rs crate](https://github.com/Aleph-Alpha/ts-rs)
- [specta crate](https://github.com/oscartbeaumont/specta)
