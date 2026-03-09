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
- `cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json --seed random`
- `cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json --seed auto`

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
- allows `sim run --seed <value>` to override the derived seed and records `seed_source`
- treats `sim run --seed random` and `sim run --seed auto` as runtime-generated seeds and persists the generated value for replay
- records `events_per_second` and `ms_per_event` as runtime observation metrics
- derives `scheduled_peer_order` from the deterministic seed and uses it for peer/event processing
- derives `fault_plan` from the deterministic seed for negative fixture ordering
- uses deterministic placeholder object IDs instead of real wire sync

## Internal Production Boundary

The current CLI can be treated as internal-production-ready only for narrow repository-local workflows.

Allowed internal-production use:

- repository validation in CI or operator-run local checks via `mycel validate`
- deterministic simulator-harness runs via `mycel sim run` against version-controlled fixture/test-case inputs
- machine-readable report generation and replay of deterministic test metadata inside the repository scaffold

Required conditions for that internal-production use:

- the CLI is run inside a repository root that contains `Cargo.toml`, `fixtures/`, and `sim/`
- CI remains green for formatting, workspace tests, and negative validation smoke coverage
- `validate` JSON/text output and `sim run` JSON/text output are treated as stable operational contracts for internal tooling
- generated reports continue to validate successfully with `mycel validate`
- usage remains bounded to the checked-in scaffold, examples, and deterministic test inputs

Not allowed to be described as current CLI capability:

- a production Mycel client
- a production Mycel node
- real wire-sync execution
- real object parsing/replay from live network traffic
- multi-process or distributed deployment orchestration
- external-operator production automation based on assumptions beyond the checked-in simulator scaffold

Practical interpretation:

- `mycel validate` is close to internal-production tooling for repository quality gates
- `mycel sim run` is suitable as an internal deterministic harness, not as evidence of live network behavior
- the CLI should still be described as implementation scaffold, not as a finished product surface

Recommended next:

- add per-step event traces to `sim run`
- replace placeholder object flow with protocol-aware sync state
- support additional negative and recovery test-cases
