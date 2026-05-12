# Neph (Project Neph)

Private, trust-first **Windows** desktop assistant: **Tauri 2** (Rust core + WebView2 UI), **React 19**, optional **Python** ML sidecar and **Node** Playwright sidecar. Alpha quality: strong confirmation gates for risky tools; supervise automations.

**Canonical product spec:** [`Blueprint.md`](Blueprint.md) (Nephis Council Round 2 — architecture, phases, tool manifest).

## Fresh machine — start here

1. Install everything in **[`docs/SETUP.md`](docs/SETUP.md)** (Rust, Node, Python, MSVC/WebView2 build prerequisites).
2. Clone and install:

```powershell
git clone https://github.com/GodSlayer-SS/ProjectNeph.git
cd ProjectNeph
npm install
```

3. Python ML sidecar (voice, VAD, local STT/TTS fallbacks, embeddings):

```powershell
python scripts/install_pyside.py
```

4. Optional — browser tools (Phase 3): run `.\scripts\setup_nodeside.ps1`, then in a second terminal `node apps\nodeside\server.js`.

5. Run the app:

```powershell
npm run tauri dev
```

6. In the app: **Settings** → add API keys (Gemini, Groq, OpenRouter, Anthropic as needed). Keys live in **Windows Credential Manager**, not plaintext in the repo.

## Quality commands

```powershell
npm run typecheck
npm run verify   # line-length guard, typecheck, clippy -D warnings, cargo test
```

## Documentation map

| Doc | Purpose |
|-----|---------|
| [`Blueprint.md`](Blueprint.md) | Definitive architecture and phased roadmap |
| [`docs/SETUP.md`](docs/SETUP.md) | Software to install on a new Windows PC |
| [`docs/INSTALL_WINDOWS.md`](docs/INSTALL_WINDOWS.md) | End-user install, SmartScreen, WebView2, troubleshooting |
| [`docs/BLUEPRINT_STATUS.md`](docs/BLUEPRINT_STATUS.md) | What is implemented vs each blueprint phase |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Layer diagram and request flow |
| [`docs/SECURITY.md`](docs/SECURITY.md) | Trust model and data handling |
| [`docs/IPC_BINDINGS.md`](docs/IPC_BINDINGS.md) | Tauri command / event conventions |
| [`docs/ADRs/`](docs/ADRs/) | Architecture decision records |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | How to change the repo safely |

## Tool manifest

Runtime tools and risk tiers are declared in **`apps/desktop/tools.toml`**. Add or change a tool there first; Rust loads this at startup (`NEPH_TOOLS_TOML` can override the path).
