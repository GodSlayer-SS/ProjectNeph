# Neph Architecture

Neph is a Windows-first Tauri desktop app with a React UI and Rust core.

## Layers
- Frontend (`src`): palette, memory, history, settings, onboarding.
- Backend (`src-tauri/src`): command routing, risk classification, tools, storage.
- Storage: local SQLite (`WAL`) initialized at startup through migrations.
- Secrets: provider keys in Windows Credential Manager via `keyring`.

## Command Flow
1. User enters command in palette.
2. Frontend calls `run_palette_command` via Tauri invoke.
3. Rust routes deterministic intent (`>note`, `>remember`, `>recall`).
4. Action and command history are persisted.
5. Response is returned to UI.

## Safety Baseline
- Risk map is hard-coded per tool.
- Command and action tables provide auditability.
- BYOK keys are never written to local DB.
