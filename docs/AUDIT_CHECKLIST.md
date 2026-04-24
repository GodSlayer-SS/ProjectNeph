# Pre-Release Audit Checklist

## Security
- [ ] API keys never persisted in SQLite.
- [ ] Red actions require typed confirmation.
- [ ] Delete routes to Recycle Bin.
- [ ] CSP restricts external origins.

## Reliability
- [ ] DB migrations run clean on fresh install.
- [ ] Undo path verified for move/rename (legacy overwrite payloads may still exist in old DB rows).
- [ ] Crash logs written for command errors.

## Performance
- [ ] Startup hotkey-ready latency measured.
- [ ] Command latency sampled from `command_history`.
- [ ] Memory retrieval median latency validated.
