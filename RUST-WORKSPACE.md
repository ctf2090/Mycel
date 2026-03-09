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
- a CLI crate with `info` and `validate`
- repository validation for fixture, peer, topology, test-case, and report inputs

It does not yet implement:

- wire sync
- object parsing or replay
- simulator execution
- report generation from a real run

## Recommended Next Step

Implemented now:

- `mycel info`
- `mycel validate`

Recommended next:

- `mycel sim run`
- richer schema-level validation
- fixture/topology/test/report loading into executable simulator state
