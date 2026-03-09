# Reports

This directory documents the expected output shape for simulator runs.

Tracked outputs should eventually include:

- per-peer received object IDs
- verification outcomes
- per-step event traces for load, sync, verify, replay, and finalize phases
- final heads by document
- replay results
- accepted-head comparison when enabled
- failure summaries

Generated report files should go under `sim/reports/out/`, which is ignored by git.
Generated reports under `sim/reports/out/` can also be validated with `mycel validate`.

## Schema

- `report.schema.json` is the formal contract for machine-readable simulator run reports.
- `report.example.json` is the first example bound to that schema.
- The schema now includes an `events` trace so early runs can expose step-by-step behavior without requiring a full wire implementation.
- Reports now also carry `started_at`, `finished_at`, and deterministic run metadata using `Asia/Taipei (UTC+8)` timestamps by default.
