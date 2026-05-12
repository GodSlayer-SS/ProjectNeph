# Setup on a new Windows PC (developers)

Use this checklist after a clean Windows install so you can clone **Project Neph** and run `npm run tauri dev` without missing dependencies.

## Required

| Software | Version | Why |
|----------|---------|-----|
| [**Git for Windows**](https://git-scm.com/download/win) | Latest | Clone, pull, push |
| [**Visual Studio Build Tools**](https://visualstudio.microsoft.com/visual-cpp-build-tools/) or full VS | 2022+ | **Desktop development with C++** workload — Rust/`link.exe`, WebView2 native bits |
| [**Rustup**](https://rustup.rs/) (`rustup-init.exe`) | Stable channel | `cargo`, `rustc` for `src-tauri` |
| [**Node.js**](https://nodejs.org/) LTS | **20+** | Vite, React, `@tauri-apps/cli`; Node sidecar for Playwright |
| [**Python**](https://www.python.org/downloads/windows/) | **3.11+** | ML sidecar (`apps/pyside`): VAD, Whisper-class STT, TTS, embeddings |
| [**Microsoft Edge WebView2**](https://developer.microsoft.com/microsoft-edge/webview2/) | Evergreen | Tauri UI runtime (often already on Windows 11; install **Fixed Version** only if you know you need it) |

Optional but recommended:

| Software | Why |
|----------|-----|
| [**Cursor**](https://cursor.com/) or VS Code | Editing + Rust analyzer |
| **Windows Terminal** | Better PowerShell experience |

## Rust toolchain

After `rustup-init`:

```powershell
rustup default stable
rustup component add rustfmt clippy
```

## Node dependencies

From repo root:

```powershell
npm install
```

Global **not** required for the desktop app; `npm run tauri` uses local `@tauri-apps/cli`.

## Python sidecar

From repo root (uses `pip` for the editable package under `apps/pyside`):

```powershell
python scripts/install_pyside.py
```

This installs `onnxruntime` (CPU) for Silero VAD and related paths. GPU extras are optional and documented in `apps/pyside/pyproject.toml`.

## Playwright / browser automation (optional)

Only if you use browser tools (`browser_read_page`, etc.):

```powershell
.\scripts\setup_nodeside.ps1
```

Then keep a terminal open:

```powershell
node apps\nodeside\server.js
```

## First run

```powershell
npm run tauri dev
```

Configure provider keys in the app (**Settings**). Data and logs default under `%LOCALAPPDATA%\Neph` (see `docs/INSTALL_WINDOWS.md`).

## Verify the tree

```powershell
npm run verify
```

Expect: Rust line-length script, `npm run typecheck`, `cargo clippy -- -D warnings`, `cargo test`.

## API keys (BYOK)

You will want at least one cloud provider for a usable voice/chat loop, per **`Blueprint.md`**:

- **Google AI (Gemini)** — primary fast model tier  
- **Groq** — fast LLM + Whisper-class STT API  
- **Anthropic** — optional Sonnet-class reasoning (direct or via router)  
- **OpenRouter** — optional aggregate access  

Keys are stored via **Windows Credential Manager** (`keyring`); never commit `.env` secrets into the repo.
