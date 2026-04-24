# Windows compatibility matrix (informal)

Neph is **Windows-first**; behavior can vary with OS build, language, and cloud sync. This is a **living** matrix, not a guarantee.

| Area | Tested / expected |
| --- | --- |
| Windows 10 22H2 (64-bit) | Baseline for desktop automation and Start Menu link indexing. |
| Windows 11 (64-bit) | Primary dev target. |
| Japanese / non-English display locale | Path and `.lnk` resolution may differ; file actions need UTF-8 paths. |
| OneDrive “Files On-Demand” / placeholders | See audit plan: moves/deletes on placeholders are **high risk** — behavior must be explicit in a future release. |
| WebView2 Evergreen | Required; if missing, the app should lead users to the runtime (future hardening). |

Add rows here as real installs are validated on VMs or user cohorts.
