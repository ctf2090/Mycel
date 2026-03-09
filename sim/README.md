# Simulator Scaffold

This directory is the language-neutral scaffold for the Mycel peer simulator.

It exists to separate simulator structure from implementation choice.

## Layout

- `peers/`: peer role definitions and peer-local configuration shape
- `topologies/`: named peer graph and bootstrap examples
- `tests/`: simulator test cases and expected assertions
- `reports/`: machine-readable output shape and report conventions
- `negative-validation/`: matrix and index for intentionally invalid validation examples
- `SCHEMA-CROSS-CHECK.md`: consistency rules between fixture, peer, topology, test case, and report schemas
- `SCHEMA-CROSS-CHECK.zh-TW.md`: Traditional Chinese version of the schema cross-check note
- `runtime/`: ignored local runtime state for manual experiments

## Build Direction

Recommended sequence:

1. implement one single-process multi-peer harness
2. bind it to fixtures in `../fixtures/`
3. add deterministic report output
4. later reuse the same peer logic in a localhost multi-process harness

This scaffold does not commit us to a language yet.

## Current Rust CLI

The Rust workspace currently exposes:

- `cargo run -p mycel-cli -- info`
- `cargo run -p mycel-cli -- help`
- `cargo run -p mycel-cli -- head inspect <doc_id> --input <path|fixture>`
- `cargo run -p mycel-cli -- head inspect <doc_id> --input <path|fixture> --json`
- `cargo run -p mycel-cli -- object inspect <path>`
- `cargo run -p mycel-cli -- object inspect <path> --json`
- `cargo run -p mycel-cli -- object verify <path>`
- `cargo run -p mycel-cli -- object verify <path> --json`
- `cargo run -p mycel-cli -- report diff <left-report> <right-report>`
- `cargo run -p mycel-cli -- report diff <left-report> <right-report> --json`
- `cargo run -p mycel-cli -- report diff <left-report> <right-report> --fail-on-diff`
- `cargo run -p mycel-cli -- report diff <left-report> <right-report> --field run-id`
- `cargo run -p mycel-cli -- report diff <left-report> <right-report> --ignore-field run-id`
- `cargo run -p mycel-cli -- report diff <left-report> <right-report> --events`
- `cargo run -p mycel-cli -- report diff <left-report> <right-report> --events --json`
- `cargo run -p mycel-cli -- report inspect <path>`
- `cargo run -p mycel-cli -- report inspect <path> --json`
- `cargo run -p mycel-cli -- report inspect <path> --full --json`
- `cargo run -p mycel-cli -- report list`
- `cargo run -p mycel-cli -- report list --json`
- `cargo run -p mycel-cli -- report list --result pass --json`
- `cargo run -p mycel-cli -- report list --validation-status warning --json`
- `cargo run -p mycel-cli -- report list --path-only`
- `cargo run -p mycel-cli -- report list <path> --json`
- `cargo run -p mycel-cli -- report latest`
- `cargo run -p mycel-cli -- report latest --json`
- `cargo run -p mycel-cli -- report latest --full --json`
- `cargo run -p mycel-cli -- report latest --path-only`
- `cargo run -p mycel-cli -- report latest --result pass --json`
- `cargo run -p mycel-cli -- report latest --validation-status warning --json`
- `cargo run -p mycel-cli -- report latest <path> --json`
- `cargo run -p mycel-cli -- report stats`
- `cargo run -p mycel-cli -- report stats --json`
- `cargo run -p mycel-cli -- report stats --counts-only --json`
- `cargo run -p mycel-cli -- report stats --full-latest --json`
- `cargo run -p mycel-cli -- report stats --path-only-latest`
- `cargo run -p mycel-cli -- report stats --result pass --json`
- `cargo run -p mycel-cli -- report stats --validation-status warning --json`
- `cargo run -p mycel-cli -- report stats <path> --json`
- `cargo run -p mycel-cli -- report inspect <path> --events`
- `cargo run -p mycel-cli -- report inspect <path> --failures`
- `cargo run -p mycel-cli -- report inspect <path> --phase <name>`
- `cargo run -p mycel-cli -- report inspect <path> --action <name>`
- `cargo run -p mycel-cli -- report inspect <path> --outcome <name>`
- `cargo run -p mycel-cli -- report inspect <path> --step <n>`
- `cargo run -p mycel-cli -- report inspect <path> --step-range <a>:<b>`
- `cargo run -p mycel-cli -- report inspect <path> --first <n>`
- `cargo run -p mycel-cli -- report inspect <path> --last <n>`
- `cargo run -p mycel-cli -- report inspect <path> --node <id>`
- `cargo run -p mycel-cli -- validate`
- `cargo run -p mycel-cli -- validate <path>`
- `cargo run -p mycel-cli -- validate <path> --json`
- `cargo run -p mycel-cli -- validate <path> --strict`
- `cargo run -p mycel-cli -- sim run <test-case>`
- `cargo run -p mycel-cli -- sim run <test-case> --json`
- `cargo run -p mycel-cli -- sim run <test-case> --seed custom-seed`
- `cargo run -p mycel-cli -- sim run <test-case> --seed random`
- `cargo run -p mycel-cli -- sim run <test-case> --seed auto`

Runnable examples:

- `cargo run -p mycel-cli -- info`
- `cargo run -p mycel-cli -- help`
- `cargo run -p mycel-cli -- report list`
- `cargo run -p mycel-cli -- report list --json`
- `cargo run -p mycel-cli -- report list --result fail --json`
- `cargo run -p mycel-cli -- report list --validation-status failed --path-only`
- `cargo run -p mycel-cli -- report list --path-only`
- `cargo run -p mycel-cli -- report list sim/reports/report.example.json --json`
- `cargo run -p mycel-cli -- report latest`
- `cargo run -p mycel-cli -- report latest --json`
- `cargo run -p mycel-cli -- report latest --full --json`
- `cargo run -p mycel-cli -- report latest --path-only`
- `cargo run -p mycel-cli -- report latest --result fail --json`
- `cargo run -p mycel-cli -- report latest --validation-status failed --path-only`
- `cargo run -p mycel-cli -- report latest sim/reports/out --json`
- `cargo run -p mycel-cli -- report stats`
- `cargo run -p mycel-cli -- report stats --json`
- `cargo run -p mycel-cli -- report stats --counts-only --json`
- `cargo run -p mycel-cli -- report stats --full-latest --json`
- `cargo run -p mycel-cli -- report stats --path-only-latest`
- `cargo run -p mycel-cli -- report stats --result pass --json`
- `cargo run -p mycel-cli -- report stats --validation-status warning --json`
- `cargo run -p mycel-cli -- report stats sim/reports/out --json`
- `cargo run -p mycel-cli -- report diff sim/reports/report.example.json sim/reports/invalid/missing-seed-source.example.json`
- `cargo run -p mycel-cli -- report diff sim/reports/report.example.json sim/reports/invalid/missing-seed-source.example.json --json`
- `cargo run -p mycel-cli -- report diff sim/reports/report.example.json sim/reports/invalid/missing-seed-source.example.json --fail-on-diff`
- `cargo run -p mycel-cli -- report diff sim/reports/report.example.json sim/reports/invalid/missing-seed-source.example.json --field run-id --field peer-count`
- `cargo run -p mycel-cli -- report diff sim/reports/report.example.json sim/reports/invalid/missing-seed-source.example.json --ignore-field run-id --ignore-field seed-source`
- `cargo run -p mycel-cli -- report diff sim/reports/report.example.json sim/reports/invalid/missing-seed-source.example.json --events`
- `cargo run -p mycel-cli -- report diff sim/reports/report.example.json sim/reports/invalid/missing-seed-source.example.json --events --json`
- `cargo run -p mycel-cli -- report inspect sim/reports/report.example.json`
- `cargo run -p mycel-cli -- report inspect sim/reports/report.example.json --full --json`
- `cargo run -p mycel-cli -- report inspect sim/reports/report.example.json --events`
- `cargo run -p mycel-cli -- report inspect sim/reports/report.example.json --phase sync`
- `cargo run -p mycel-cli -- report inspect sim/reports/report.example.json --action seed-advertise`
- `cargo run -p mycel-cli -- report inspect sim/reports/report.example.json --outcome ok`
- `cargo run -p mycel-cli -- report inspect sim/reports/report.example.json --step 2`
- `cargo run -p mycel-cli -- report inspect sim/reports/report.example.json --step-range 2:3`
- `cargo run -p mycel-cli -- report inspect sim/reports/report.example.json --first 2`
- `cargo run -p mycel-cli -- report inspect sim/reports/report.example.json --last 2`
- `cargo run -p mycel-cli -- report inspect sim/reports/report.example.json --node node:peer-seed`
- `cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json`
- `cargo run -p mycel-cli -- sim run sim/tests/hash-mismatch.example.json`
- `cargo run -p mycel-cli -- sim run sim/tests/signature-mismatch.example.json`
- `cargo run -p mycel-cli -- sim run sim/tests/partial-want-recovery.example.json`

Info/help output notes:

- `info` prints the workspace banner, simulator scaffold banner, and the current fixture / peer / topology / test / report paths
- `help` and a no-argument invocation both print the same top-level usage sections
- unknown top-level commands print the same usage text and exit with an error

Report-inspection output notes:

- `report list` discovers report JSON files under `sim/reports/` by default, recursively skipping `report.schema.json`
- `report list --json` emits a stable listing summary with `root`, `status`, `report_count`, `valid_report_count`, `invalid_report_count`, `reports[]`, and `errors`
- `report list --result <pass|fail>` narrows listed valid reports to one result while still keeping invalid parse entries visible as warnings
- `report list --validation-status <ok|warning|failed>` narrows listed valid reports to one validation status while still keeping invalid parse entries visible as warnings
- `report list --path-only` prints only matching valid report paths and is intended for shell-pipeline handoff
- `report list <path>` accepts either one directory or one explicit report file
- list entries carry stable fields such as `path`, `status`, `run_id`, `topology_id`, `fixture_id`, `test_id`, `started_at`, `finished_at`, `validation_status`, `result`, `peer_count`, `event_count`, `failure_count`, and `parse_error`
- parse failures inside a listing downgrade the overall list status to `warning` but do not fail the command; target resolution failures still return `status: failed`
- `report latest` selects the newest valid report under the target and prints a human-readable summary
- `report latest --json` emits a stable summary with `root`, `status`, counts, `selected`, and `errors`
- `report latest --full --json` emits the selected raw report JSON without summary reshaping
- `report latest --path-only` prints only the selected report path and is intended for shell-script handoff
- `report latest --result <pass|fail>` narrows latest selection to one report result before any summary, raw, or path-only output is produced
- `report latest --validation-status <ok|warning|failed>` narrows latest selection to one report validation status before any summary, raw, or path-only output is produced
- latest selection prefers `finished_at`, then `started_at`, then path as a deterministic tie-break
- invalid reports do not block `report latest` if at least one valid report exists; they downgrade the top-level status to `warning`
- `report stats` summarizes one report directory or file and aggregates counts across valid reports
- `report stats --json` emits a stable summary with `root`, `status`, counts, `result_counts`, `validation_status_counts`, `latest_finished_at`, `latest_valid_report`, and `errors`
- `report stats --counts-only --json` emits only aggregate counts plus top-level status/filter fields, without latest-report detail fields
- `report stats --full-latest --json` emits the latest matching raw report JSON and falls back to a failed stats summary JSON when no valid report matches
- `report stats --path-only-latest` prints only the latest matching valid report path and is intended for shell-script handoff
- `report stats --result <pass|fail>` narrows the aggregated valid-report set to one result while still retaining invalid parse entries in the top-level counts and status
- `report stats --validation-status <ok|warning|failed>` narrows the aggregated valid-report set to one validation status while still retaining invalid parse entries in the top-level counts and status
- `report stats` counts valid reports by `result` and `validation_status`, while still surfacing invalid parse entries through the shared top-level status and counts
- `report diff <left> <right>` compares two reports at the summary level and prints a human-readable difference list
- `report diff <left> <right> --json` emits a stable diff summary with `status`, `comparison`, `difference_count`, `left`, `right`, `differences`, and `errors`
- `report diff <left> <right> --fail-on-diff` keeps the same payload but exits with failure when `comparison` is `different`
- `report diff <left> <right> --field <field>` turns diffing into an allowlist mode; repeat the flag to compare multiple specific fields
- `report diff <left> <right> --ignore-field <field>` removes specific fields from comparison; repeat the flag to ignore multiple fields
- `report diff <left> <right> --events` compares event traces step-by-step and prints event-level differences instead of summary-field differences
- `report diff <left> <right> --events --json` emits `event_difference_count` plus `event_differences[]`, where each entry is `changed`, `left_only`, or `right_only`
- `--field` and `--ignore-field` are mutually exclusive
- `status: ok` means both inputs parsed as report targets successfully; use `comparison: match|different` to tell whether the summaries differ
- the default diff mode compares stable summary fields such as IDs, execution metadata, counts, expected outcomes, scheduled peer order, and fault-plan count
- event diff currently aligns trace entries by `step`
- event diff can ignore event subfields such as `event-detail`, `event-node-id`, and `event-object-ids`
- `report inspect <path>` prints a human-readable summary for one simulator report
- `report inspect <path> --json` emits a stable inspection summary including run identity, result, counts, selected metadata, and errors
- `report inspect <path> --full --json` emits the raw report JSON without summary reshaping
- `report inspect <path> --events` narrows the view to event trace entries
- `report inspect <path> --failures` narrows the view to failure entries
- `report inspect <path> --phase <name>` narrows event inspection to one phase and implicitly uses event view
- `report inspect <path> --action <name>` narrows event inspection to one action and implicitly uses event view
- `report inspect <path> --outcome <name>` narrows event inspection to one outcome and implicitly uses event view
- `report inspect <path> --step <n>` narrows event inspection to one step number and implicitly uses event view
- `report inspect <path> --step-range <a>:<b>` narrows event inspection to one inclusive step range and implicitly uses event view
- `report inspect <path> --first <n>` keeps the first `n` matching events after other event filters are applied
- `report inspect <path> --last <n>` keeps the last `n` matching events after other event filters are applied
- `report inspect <path> --node <id>` narrows event inspection to one node, or failure inspection when combined with `--failures`
- `--events`, `--failures`, and `--full` are mutually exclusive
- `--phase` cannot be combined with `--failures` or `--full`
- `--action` cannot be combined with `--failures` or `--full`
- `--outcome` cannot be combined with `--failures` or `--full`
- `--step` cannot be combined with `--failures` or `--full`
- `--step-range` cannot be combined with `--failures` or `--full`
- `--first` cannot be combined with `--failures` or `--full`
- `--last` cannot be combined with `--failures` or `--full`
- `--step` and `--step-range` are mutually exclusive
- `--node` cannot be combined with `--full`
- `--full` requires `--json`
- schema files are not valid inspect targets; use an actual report file such as `sim/reports/report.example.json` or one generated under `sim/reports/out/`

Minimal `report diff --json` shape example:

```json
{
  "status": "ok",
  "comparison": "different",
  "difference_count": 3,
  "left": {
    "path": "sim/reports/report.example.json",
    "run_id": "run:example-001",
    "result": "pass",
    "peer_count": 2,
    "event_count": 3,
    "failure_count": 0,
    "errors": []
  },
  "right": {
    "path": "sim/reports/invalid/missing-seed-source.example.json",
    "run_id": "run:missing-seed-source-001",
    "result": "pass",
    "peer_count": 3,
    "event_count": 3,
    "failure_count": 0,
    "errors": []
  },
  "differences": [
    {
      "field": "run_id",
      "left": "run:example-001",
      "right": "run:missing-seed-source-001"
    },
    {
      "field": "peer_count",
      "left": 2,
      "right": 3
    },
    {
      "field": "seed_source",
      "left": "derived",
      "right": null
    }
  ],
  "errors": []
}
```

Minimal `report diff --events --json` shape example:

```json
{
  "status": "ok",
  "comparison": "different",
  "event_difference_count": 1,
  "left": {
    "path": "sim/reports/report.example.json",
    "event_count": 3,
    "errors": []
  },
  "right": {
    "path": "sim/reports/invalid/missing-seed-source.example.json",
    "event_count": 3,
    "errors": []
  },
  "event_differences": [
    {
      "step": 1,
      "change": "changed",
      "left": {
        "step": 1,
        "phase": "load",
        "action": "load-fixture",
        "outcome": "ok",
        "detail": "Loaded minimal-valid fixture for baseline sync testing."
      },
      "right": {
        "step": 1,
        "phase": "load",
        "action": "load-fixture",
        "outcome": "ok",
        "detail": "Loaded minimal-valid fixture for missing-seed-source warning demonstration."
      }
    }
  ],
  "errors": []
}
```

Minimal `report stats --counts-only --json` shape example:

```json
{
  "root": "sim/reports",
  "status": "warning",
  "result_filter": null,
  "validation_status_filter": null,
  "report_count": 3,
  "valid_report_count": 2,
  "invalid_report_count": 1,
  "result_counts": {
    "fail": 1,
    "pass": 1
  },
  "validation_status_counts": {
    "ok": 1,
    "warning": 1
  },
  "errors": []
}
```

Validation output notes:

- `--json` includes a stable top-level `status`
- `--strict` treats warnings as failures for CI-oriented validation
- tools and tests should rely on JSON fields such as `status`, `root`, `target`, `fixture_count`, `peer_count`, `topology_count`, `test_case_count`, `report_count`, `warnings`, and `errors`
- warning-only validation still emits `status: warning`; `--strict` changes the exit behavior, not the warning payload itself

Minimal JSON shape example:

```json
{
  "status": "warning",
  "root": "/workspaces/Mycel",
  "target": "/workspaces/Mycel/sim/reports/invalid/missing-seed-source.example.json",
  "fixture_count": 1,
  "peer_count": 0,
  "topology_count": 1,
  "test_case_count": 1,
  "report_count": 1,
  "warnings": [
    {
      "message": "report metadata does not include seed_source"
    }
  ],
  "errors": []
}
```

Object-inspection output notes:

- text output is intended for quick structural inspection
- `--json` is the stable machine-consumable surface
- `object inspect` summarizes structure and declared metadata without recomputing IDs or verifying signatures
- tools and tests should rely on fields such as `status`, `object_type`, `version`, `signature_rule`, `signer_field`, `declared_id_field`, `declared_id`, `has_signature`, `top_level_keys`, `notes`, and `errors`
- `status: warning` means the file parsed as one JSON object but the shape is incomplete, unsupported, or suspicious

Minimal JSON shape example:

```json
{
  "status": "warning",
  "object_type": "patch",
  "version": "mycel/0.1",
  "signature_rule": "required",
  "signer_field": "author",
  "declared_id_field": "patch_id",
  "declared_id": null,
  "has_signature": false,
  "top_level_keys": [
    "base_revision",
    "doc_id",
    "ops",
    "timestamp",
    "type",
    "version"
  ],
  "notes": [
    "patch object is missing string signer field 'author'",
    "patch object is missing string field 'patch_id'",
    "patch object is missing top-level 'signature'"
  ],
  "errors": []
}
```

Object-verification output notes:

- text output is intended for human inspection
- `--json` is the stable machine-consumable surface
- tools and tests should rely on fields such as `status`, `object_type`, `signature_rule`, `signature_verification`, `declared_id`, `recomputed_id`, `notes`, and `errors`
- `declared_id` and `recomputed_id` are the primary fields for derived-ID mismatch checks

Minimal JSON shape example:

```json
{
  "status": "ok",
  "object_type": "patch",
  "signature_rule": "required",
  "signer_field": "author",
  "signer": "pk:ed25519:...",
  "signature_verification": "verified",
  "declared_id": "patch:...",
  "recomputed_id": "patch:...",
  "notes": [],
  "errors": []
}
```

Head-inspection output notes:

- `decision_trace` is intentionally kept as a high-level explanatory summary for humans
- typed arrays such as `effective_weights[]`, `maintainer_support[]`, and `critical_violations[]` carry the stable machine-consumable selection detail
- tools and tests should rely on typed arrays for detailed assertions instead of parsing `decision_trace`

Minimal JSON shape example:

```json
{
  "status": "ok",
  "selected_head": "rev:...",
  "effective_weights": [
    {
      "maintainer": "pk:ed25519:...",
      "admitted": true,
      "effective_weight": 2
    }
  ],
  "maintainer_support": [
    {
      "maintainer": "pk:ed25519:...",
      "revision_id": "rev:...",
      "effective_weight": 2
    }
  ],
  "critical_violations": [],
  "decision_trace": [
    {
      "step": "effective_weight",
      "detail": "maintainers=3 admitted=2 penalized=0 zero_weight=1 max_effective_weight=2"
    },
    {
      "step": "selected_head",
      "detail": "selected=rev:... tie_break_reason=higher_selector_score"
    }
  ]
}
```

Simulator-run output notes:

- text output is intended for human-readable run summaries
- `--json` is the stable machine-consumable run summary surface
- tools and tests should rely on fields such as `result`, `validation_status`, `report_path`, `deterministic_seed`, `seed_source`, `event_count`, `peer_count`, `verified_object_count`, `rejected_object_count`, `matched_expected_outcomes`, `scheduled_peer_order`, and `fault_plan`
- `report_path` points to the generated full report; use the report for step-by-step `events`, `failures`, and persisted run metadata
- `seed_source` records whether the seed was `derived`, `override`, `random`, or `auto`

Minimal JSON shape example:

```json
{
  "result": "pass",
  "validation_status": "ok",
  "report_path": "/workspaces/Mycel/sim/reports/out/three-peer-consistency.example.report.json",
  "deterministic_seed": "derived:...",
  "seed_source": "derived",
  "peer_count": 3,
  "event_count": 12,
  "verified_object_count": 3,
  "rejected_object_count": 0,
  "matched_expected_outcomes": [
    "sync-success"
  ],
  "scheduled_peer_order": [
    "reader-A",
    "seed-B",
    "observer-C"
  ],
  "fault_plan": []
}
```

Simulator run notes:

- `sim run` currently supports only `single-process` test-cases
- generated reports are written under `sim/reports/out/`
- generated reports now include a step-by-step `events` trace
- generated reports now include `started_at`, `finished_at`, and deterministic run metadata
- deterministic run metadata now includes `run_duration_ms` and `deterministic_seed`
- deterministic run metadata now records whether the seed was `derived`, `override`, `random`, or `auto`
- `--seed random` and `--seed auto` both generate a fresh runtime seed and persist it in the report for replay
- runtime observation metadata now includes `events_per_second` and `ms_per_event`
- deterministic scheduling now records `scheduled_peer_order`
- deterministic fault ordering now records `fault_plan`

Negative validation notes:

- `sim/negative-validation/README.md` indexes intentionally invalid artifacts without mixing them into normal simulator examples
- `sim/negative-validation/test-matrix.md` tracks current and planned validator failure cases
- `sim/negative-validation/smoke.sh` runs one positive and one intentional negative validation path together
- `apps/mycel-cli/tests/common/mod.rs` provides shared CLI test helpers for command execution, JSON parsing, report loading, exit-code checks, JSON status checks, stdout/stderr assertions, and shared section assertions for usage/help and info output
- `apps/mycel-cli/tests/info_smoke.rs` fixes the `info` command contract for workspace banner and scaffold path output
- `apps/mycel-cli/tests/cli_usage_smoke.rs` fixes the top-level help and usage contract for `help`, no-arg, and unknown-command flows
- `apps/mycel-cli/tests/validate_smoke.rs` covers core validator smoke cases plus path-targeted and argument-parsing edge cases for directory, schema-file, missing-path, and unexpected-argument targets
- `apps/mycel-cli/tests/sim_run_smoke.rs` covers baseline `sim run` behavior for positive, negative, and recovery paths, including generated-report round-trip validation, `random` / `auto` seed modes, and CLI edge cases for subcommands, targets, and invalid arguments
- GitHub Actions now runs `./sim/negative-validation/smoke.sh --summary-only` in CI alongside Rust formatting, compile, and workspace test checks
