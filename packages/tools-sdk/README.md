# packages/tools-sdk/

**Purpose**: Tool trait contract + `tools.toml` JSON Schema reference for IDE validation and external tool development.

## Phase Status

**Phase 2+** — The SDK is deferred until tool definitions stabilise.

## What Goes Here

- `tools.toml` JSON Schema (for IDE autocompletion in tools.toml)
- TypeScript types for tool manifests (`ToolEntry`, `ArgSpec`, etc.)
- Documentation for third-party tool authors

## tools.toml Schema Preview

```toml
[[tool]]
name = "my_tool"          # snake_case, unique
risk = "green"            # green | yellow | red
description = "..."       # what the LLM planner reads
domain = "workspace"      # see domains/ for valid handles
egress = []               # allowed hostnames ([] = no network)
phase = 1                 # minimum phase required

[tool.args]
my_arg = { type = "string", required = true, description = "..." }
```

## References

- Blueprint §4: `packages/tools-sdk/ — Tool trait + manifest schema`
- `apps/desktop/tools.toml` — the master manifest
- `src-tauri/src/tools/manifest.rs` — runtime loader
