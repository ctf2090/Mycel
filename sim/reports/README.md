# Reports

This directory documents the expected output shape for simulator runs.

Tracked outputs should eventually include:

- per-peer received object IDs
- verification outcomes
- final heads by document
- replay results
- accepted-head comparison when enabled
- failure summaries

Generated report files should go under `sim/reports/out/`, which is ignored by git.

## Schema

- `report.schema.json` is the formal contract for machine-readable simulator run reports.
- `report.example.json` is the first example bound to that schema.
- The schema keeps report output narrow enough for early harness work while leaving room for future summary and failure fields.
