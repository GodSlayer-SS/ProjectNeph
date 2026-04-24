# Install on Windows (Unsigned v0.1)

## Installer
- Download the latest NSIS installer from GitHub Releases.
- Run the installer and complete the setup wizard.
- The bundle uses **WebView2 Evergreen** via an embedded bootstrapper (small size increase, needs network during install unless the runtime is already present). If the UI is blank, install or repair the runtime from [Microsoft’s WebView2 page](https://developer.microsoft.com/microsoft-edge/webview2/).

## SmartScreen Warning
- Because v0.1 is unsigned, Windows SmartScreen may show "Windows protected your PC".
- Click **More info** then **Run anyway** if you trust the release origin.
- Verify download source is the official GitHub release page before proceeding.

## Antivirus and Controlled folder access
- Some AV products may quarantine a new binary; use your vendor’s “restore / allow” flow and optionally exclude the install folder and `%LOCALAPPDATA%\Neph`.
- If **Controlled folder access** (ransomware protection) is on, allow **Neph** under *Windows Security → Virus & threat protection → Ransomware protection → Allow an app through Controlled folder access* if file moves/deletes fail with access denied.

## ONNX / ML-heavy builds (optional)
- If you ship or side-load ONNX or GPU inference DLLs, some AV heuristics flag them. Document expected DLL names and consider code signing for anything beyond local dev.

## Troubleshooting
- If install fails, run installer as Administrator.
- If antivirus blocks the binary, restore and add an exception for the install directory.
- Diagnostic and crash logs: `%LOCALAPPDATA%\Neph\logs\` (rolling daily `neph.*` files). File an issue from Settings > About > Report Issue and attach the latest log when possible.

## Hotkey (IME)
- Default palette hotkey is **Ctrl+Space**. On some East Asian IMEs that combination toggles input mode instead of reaching Neph. Use **Settings → General** to pick **Ctrl+Shift+Space** or **Alt+Space**, then **restart Neph**.
