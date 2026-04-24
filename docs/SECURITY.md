# Security Model (v0.1 Baseline)

## Core Principles
- Local-first by default.
- Explicit risk-based confirmation for side effects.
- No plaintext API keys in project files or SQLite.

## Secret Handling
- Provider keys are written to Windows Credential Manager using `keyring`.
- Frontend only sends key material for explicit save operations.
- Provider names are allowlisted before key read/write (`groq`, `gemini`, `openrouter`).
- API keys are never persisted in SQLite and never logged in command history.
- Saving a key **refuses** empty values and surfaces credential-store errors (no plaintext fallback).

## Trust Surfaces
- `command_history` records what was requested.
- `actions` records what executed and with what risk level.
- File-destructive operations are designated red risk and must be user-confirmed **in the backend**: yellow/red plans require a one-shot `confirmationToken` bound to a `planHash` (60s TTL). The UI alone cannot authorize execution.
- **LLM routing** cannot start yellow/red (state-changing / destructive) tools; users must use explicit `>` prefixes for those.
- **Path policy** restricts mutating file operations to resolved paths under an allowlist (profile, local app data, temp, current working directory) and blocks protected system locations.
- Persisted `tool_args` and lineage JSON are **redacted** (secret patterns and long content previews).
- History UI exposes action args/result details, provenance (`user_prefix` vs `llm`), and lineage for auditability.

## CSP and IPC Hardening
- Tauri CSP is locked to self-hosted app assets and explicit API domains.
- Frontend can only call typed command wrappers; no generic shell/file execution is exposed.

## Deferred
- Plugin loading, autonomous file agents, and cross-platform support are out of scope in v0.1.
