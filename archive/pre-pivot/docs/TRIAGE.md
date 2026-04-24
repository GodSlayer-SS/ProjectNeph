# Issue triage loop

Solo maintainer, **best effort** (not 24/7 on-call). Target response time: **classify and acknowledge within 72 hours** of a well-formed report, not 24h.

## Daily / weekly

1. Pull latest issues and discussions.
2. Label severity (`critical`, `high`, `normal`, `low`).
3. Label type (`bug`, `feature`, `docs`, `question`).
4. Add milestone (`v0.1.x`, `v0.2`).

## Fix priority

- P0: startup crash, data loss, destructive misfire.
- P1: core command regression (`>app`, `>find`, memory commands).
- P2: UX friction, non-blocking errors.
- P3: polish.

## Triage SLA (classify, not always fix)

- **Target:** first maintainer pass (labels + short acknowledgment) within **72h** of report.
- **P0 / security:** best-effort same-week fix or documented workaround.
- This project has **no paid SLA**; GitHub is the system of record.
