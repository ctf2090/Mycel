# Rust Workspace

This note describes the first Rust workspace cut for Mycel.

## Goals

- keep protocol-facing logic in Rust
- keep simulator logic in a reusable Rust library
- expose a thin CLI before any Flutter UI work starts

## Layout

- `crates/mycel-core/`: shared protocol-facing Rust library
- `crates/mycel-sim/`: simulator-facing Rust library
- `apps/mycel-cli/`: initial CLI binary crate

## Current Scope

The current Rust workspace now includes:

- a protocol-facing core crate
- a simulator-facing crate with scaffold data models
- a CLI crate with `info`, `validate`, and `sim run`
- repository validation for fixture, peer, topology, test-case, and report inputs
- first-pass single-process report generation for one test-case

It does not yet implement:

- wire sync
- object parsing or replay
- simulator execution
- report generation from a real run

## Recommended Next Step

Implemented now:

- `mycel info`
- `mycel validate`
- `mycel sim run`

Current validate examples:

- `cargo run -p mycel-cli -- validate`
- `cargo run -p mycel-cli -- validate fixtures/object-sets/signature-mismatch/fixture.json`
- `cargo run -p mycel-cli -- validate sim/tests/three-peer-consistency.example.json`
- `cargo run -p mycel-cli -- validate sim/tests/three-peer-consistency.example.json --json`
- `cargo run -p mycel-cli -- validate sim/tests/three-peer-consistency.example.json --strict`
- `cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json`
- `cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json --json`

Current validate output behavior:

- text output now reports a top-level validation status
- `--json` emits a stable `status` field with `ok`, `warning`, or `failed`
- `--strict` returns a non-zero exit code when warnings are present, which is useful for CI

Current `sim run` behavior:

- supports only `single-process` test-cases
- loads one `test-case -> topology -> fixture` chain
- writes a machine-readable report to `sim/reports/out/`
- emits a deterministic per-step event trace inside the generated report
- stamps generated reports with `started_at` / `finished_at` in `Asia/Taipei (UTC+8)`
- records deterministic run metadata, including source paths and validation status
- records `run_duration_ms` and a derived `deterministic_seed` for reproducible scheduling
- uses deterministic placeholder object IDs instead of real wire sync

Recommended next:

- add per-step event traces to `sim run`
- replace placeholder object flow with protocol-aware sync state
- support additional negative and recovery test-cases
