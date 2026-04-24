# Neph v0.1.0-alpha — release notes

## What this build is

A **Windows-first** command palette for local, supervised testing. Trust controls (confirmation tokens, path policy, LLM injection gate) are implemented in the backend; some product polish and signing paths are still **not** production-grade.

## Trust guarantees (this snapshot)

- Yellow / red tools require a **backend-issued confirmation token** bound to `plan_hash` (one-shot, TTL).
- **LLM-routed** intents cannot execute mutating yellow/red tools; explicit user prefixes are required.
- API keys live in the **OS credential store**, not SQLite or command history.
- File deletes go to the **Recycle Bin** where supported.
- **Auto-updater is disabled** in repo config until signing and a real endpoint are wired.

## Known limitations

- **Unsigned** Windows builds: expect SmartScreen friction (see `docs/INSTALL_WINDOWS.md`).
- **Embeddings** are stub-hash based, not a real transformer model; see `docs/EMBEDDING.md`.
- **sqlite-vec** may be unavailable on some machines; vector recall degrades gracefully.
- **Clean-room VM install** validation is a manual gate before widening distribution.

## Diagnostics

- Settings → Privacy → **Export diagnostic bundle** writes a ZIP next to the database with `report.json` (counts, settings keys, schema version) and recent `neph*` log files from `%LOCALAPPDATA%\Neph\logs`.
- Optional **DPAPI-protected** memory exports (Windows only) when enabled in the same screen.
