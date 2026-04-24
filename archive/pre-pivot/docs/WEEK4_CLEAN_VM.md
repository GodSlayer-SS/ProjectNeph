# Week 4 — clean VM / cold-install gate

Use this before widening distribution beyond your own machine. The goal is to catch missing prerequisites (WebView2, VC++ runtime), SmartScreen friction, and first-run DB migrations on a **non-developer** Windows profile.

## Automated preflight (same machine, fresh checks)

From the repo root (PowerShell):

```powershell
./scripts/week4-clean-vm.ps1
```

The script reads the WebView2 Evergreen **pv** version from the registry keys Neph uses at runtime and warns when it is missing or below the app’s documented minimum.

## Manual VM checklist (recommended)

1. Create a **local Windows user** or VM snapshot with no Node, Rust, or VS Build Tools installed (or use a stock Windows Sandbox).
2. Install the **Neph NSIS** build you intend to ship (debug or release).
3. First launch: confirm the palette opens, Settings load, and no crash dialog appears.
4. Open **Settings → About** and confirm **WebView2** and **embedding / vector** lines look sane.
5. Run `>scanfiles` on a small test folder (optional) and confirm no permission surprises with Defender **Controlled folder access**.
6. **Settings → Privacy → Export diagnostic bundle** and confirm a ZIP appears next to the database.

Record pass/fail and build ID in your release notes.
