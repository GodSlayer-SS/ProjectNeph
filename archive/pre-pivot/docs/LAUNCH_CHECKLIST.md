# Public Launch Checklist

## Repo / engineering (before widening distribution)

- [x] **Install + SmartScreen + AV** guidance: `docs/INSTALL_WINDOWS.md`.
- [x] **CHANGELOG** tracks alpha work: `[Unreleased]` plus `0.1.0-alpha.0` history in `CHANGELOG.md`.
- [x] **CI + local verify**: `.github/workflows/ci.yml` and `scripts/verify.ps1` (typecheck, clippy `-D warnings`, tests, Vite build, Playwright smoke, Tauri debug build — last two skippable via `-SkipE2E` / `-SkipTauriBuild`).
- [x] **Cold-install / WebView2 preflight**: `scripts/week4-clean-vm.ps1` and `docs/WEEK4_CLEAN_VM.md`.

## Assets

- [ ] README includes **real** screenshots/GIFs (placeholders: `docs/screenshots/README.md`; add `palette.png` / `demo.gif` when ready).

## Channels

- [ ] Hacker News (Show HN)
- [ ] Reddit (`r/Windows11`, `r/opensource`, `r/productivity`)
- [ ] Discord and X/Twitter demo post

## Post-Launch

- [ ] Monitor issues every 12h for first 48h.
- [ ] Publish hotfix release if P0/P1 issues are reported.
