# Code signing (Windows)

## SignPath.io OSS (long lead time)

- For public releases that should avoid the worst of SmartScreen friction, plan on applying to an **OSS code-signing** program (e.g. **SignPath** for open-source projects).
- **File the application early**; approval and credential setup can take **weeks**.

## What Neph does today

- NSIS and bundle settings exist in Tauri; **signed binaries are not implied** by the repo state alone. Read the latest release notes and `INSTALL_WINDOWS.md` for the actual user experience (SmartScreen, unsigned install).

## Checklist (when a tag is public)

- [ ] Confirm whether the `*.exe` / installer is **signed** and with which cert.
- [ ] If unsigned, document the exact user steps (More info / Run anyway) in install docs.
- [ ] Add signing timestamp / renewal dates to your internal calendar if you have a cert.
