# Simulator Scaffold

This directory is the language-neutral scaffold for the Mycel peer simulator.

It exists to separate simulator structure from implementation choice.

## Layout

- `peers/`: peer role definitions and peer-local configuration shape
- `topologies/`: named peer graph and bootstrap examples
- `tests/`: simulator test cases and expected assertions
- `reports/`: machine-readable output shape and report conventions
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
- `cargo run -p mycel-cli -- validate`
- `cargo run -p mycel-cli -- validate <path>`
- `cargo run -p mycel-cli -- validate <path> --json`
- `cargo run -p mycel-cli -- validate <path> --strict`
- `cargo run -p mycel-cli -- sim run <test-case>`
- `cargo run -p mycel-cli -- sim run <test-case> --json`
- `cargo run -p mycel-cli -- sim run <test-case> --seed custom-seed`

Runnable examples:

- `cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json`
- `cargo run -p mycel-cli -- sim run sim/tests/hash-mismatch.example.json`
- `cargo run -p mycel-cli -- sim run sim/tests/signature-mismatch.example.json`
- `cargo run -p mycel-cli -- sim run sim/tests/partial-want-recovery.example.json`

Validation output notes:

- `--json` includes a stable top-level `status`
- `--strict` treats warnings as failures for CI-oriented validation

Simulator run notes:

- `sim run` currently supports only `single-process` test-cases
- generated reports are written under `sim/reports/out/`
- generated reports now include a step-by-step `events` trace
- generated reports now include `started_at`, `finished_at`, and deterministic run metadata
- deterministic run metadata now includes `run_duration_ms` and `deterministic_seed`
- deterministic run metadata now records whether the seed was `derived` or `override`
- runtime observation metadata now includes `events_per_second` and `ms_per_event`
- deterministic scheduling now records `scheduled_peer_order`
- deterministic fault ordering now records `fault_plan`
