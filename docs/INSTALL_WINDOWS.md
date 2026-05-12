# Windows: install, runtime, and troubleshooting

For **developer prerequisites** (Git, Rust, Node, Python, MSVC, WebView2 SDK expectations), read **[`SETUP.md`](SETUP.md)** first.

For **what is implemented vs the product blueprint**, see **[`BLUEPRINT_STATUS.md`](BLUEPRINT_STATUS.md)**.

---

## End-user installer (releases)

- Download the latest **NSIS** installer from **GitHub Releases** (when published).
- The bundle targets **WebView2 Evergreen** (bootstrapper may download runtime during install). If the window is blank, install or repair from [Microsoft’s WebView2 page](https://developer.microsoft.com/microsoft-edge/webview2/).

## SmartScreen (unsigned builds)

- Alpha builds may be unsigned: SmartScreen can show “Windows protected your PC”.
- Use **More info** → **Run anyway** only if you trust the release source (official GitHub release).

## Antivirus and Controlled folder access

- Allow/quarantine restore for the app if AV blocks a fresh binary.
- If **Controlled folder access** is on, allow Neph under *Windows Security → Virus & threat protection → Ransomware protection* if file moves fail.

## ONNX / ML-heavy paths (optional)

- Optional ONNX / GPU DLLs can trigger heuristics on some AV products. Prefer signed releases for distribution; for local dev, document any restored binaries.

## Troubleshooting

- Re-run installer as Administrator if setup fails.
- Logs: `%LOCALAPPDATA%\Neph\logs\` (rolling `neph.*`). Use **Settings → About** to report issues with the latest log attached.

## Hotkey (IME)

- Default palette hotkey is **Ctrl+Space**. Some IMEs capture it — use **Settings → General** for **Ctrl+Shift+Space** or **Alt+Space**, then restart the app.

---

## Phase 3: Browser automation (optional)

Browser tools need the **Playwright sidecar** and **Node 20+**.

```powershell
.\scripts\setup_nodeside.ps1
node apps\nodeside\server.js
```

Pipe: `\\.\pipe\NephNodeSide`. The desktop app connects when a browser tool runs.

### Profile isolation

| Profile | Tier | Purpose |
|---------|------|---------|
| `nephis-research` | Green | Anonymous research |
| `nephis-tools` | Yellow | Automation / forms |
| `nephis-personal` | Red | Logged-in sessions — explicit args + confirmation |
| `nephis-throwaway` | Green | Disposable context |

Personal-profile tools require `explicit_personal=true` in the manifest and user confirmation.
