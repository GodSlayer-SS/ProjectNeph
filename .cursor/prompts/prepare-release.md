# Template: prepare release

**Version:** (semver + tag)  
**Channel:** (alpha / beta / public)

## Checklist

- [ ] `CHANGELOG` section for this version; `Unreleased` cleared appropriately.
- [ ] `README` status matches (alpha/beta/GA).
- [ ] `SECURITY` / `INSTALL` match signing and updater reality.
- [ ] CI green: typecheck, clippy, test, build.
- [ ] Release notes: known limitations + trust guarantees (honest).

## Artifacts

(Installer, `.sig` / `latest.json` if updater enabled; attach checksums in GitHub release body.)
