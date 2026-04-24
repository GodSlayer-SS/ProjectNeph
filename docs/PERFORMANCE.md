# Performance Notes (v0.1)

## Current Profiling Baseline
- Command execution latency is captured in `command_history.latency_ms`.
- App intent routing uses a 20-entry in-memory command cache for repeated inputs.
- Memory retrieval uses a hybrid score (lexical + embedding cosine) over bounded candidates.

## Startup Optimizations Applied
- Hotkey registration happens once at setup.
- App and file scans are bounded and lazy-invoked (`>scanapps`, `>scanfiles`) for heavy refresh.
- SQLite runs in WAL mode with indexed lookup paths.

## Next Profiling Pass
- Instrument cold-start timestamp and first-command latency.
- Add benchmark script for 1k memory retrieval queries.
- Evaluate optional ONNX-backed embedding runtime when binary size budget allows.
