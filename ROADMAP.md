# Mycel Roadmap

Status: draft

This roadmap turns the current README priorities, implementation checklist, and design-note planning guidance into one repo-level build sequence.

It is intentionally narrow:

- build the first interoperable client first
- keep protocol-core changes conservative
- move mature ideas into profiles, schemas, and tests before expanding scope

## Current Position

The repository already has:

- a growing v0.1 protocol and wire-spec document set
- a Rust CLI suitable for internal validation and deterministic simulator workflows
- early `mycel-core` support for object schema metadata, object-envelope parsing, object inspection, object verification, and accepted-head inspection
- simulator fixtures, topologies, tests, and reports for regression coverage

The repository does not yet have:

- a complete interoperable node implementation
- a finished object-authoring and storage-write path
- end-to-end wire sync
- a production-ready public CLI or app

## Roadmap Summary

### Now

The current lane is:

1. finish the narrow first-client core
2. harden deterministic validation and replay behavior
3. keep expanding fixtures, simulator coverage, and negative tests

### Next

After the narrow core is stable, the next lane is:

1. reader-oriented accepted-head and governance workflows
2. fixed-profile accepted reading
3. reader-first text reconstruction and inspection

### Later

The later lane is:

1. canonical wire sync
2. end-to-end peer replication
3. selective app-layer expansion on top of a stable protocol core

## Planning Levels

The roadmap follows the planning split already suggested in the design notes:

1. `minimal`
2. `reader-plus-governance`
3. `full-stack`

Each later phase assumes the earlier one is already stable.

## Milestones

The roadmap is tracked through these milestones:

1. `M1` Core Object and Validation Base
2. `M2` Replay, Storage, and Rebuild
3. `M3` Reader and Governance Surface
4. `M4` Wire Sync and Peer Interop
5. `M5` Selective App-Layer Expansion

## Phase 1: Minimal

Goal: reach a narrow first client that can parse, verify, store, replay, and inspect Mycel objects deterministically.

### Deliverables

1. Shared protocol object model for all v0.1 object families
2. Canonical serialization, derived ID recomputation, and signature verification
3. Replay-based revision verification and `state_hash` checking
4. Local object store and rebuildable indexes
5. Stable internal CLI/API for validation, object verification, object inspection, and accepted-head inspection
6. Interop fixtures and negative tests for object and simulator validation

### Exit Criteria

1. Required object types parse and validate reproducibly
2. Canonical IDs and signatures are deterministic
3. Revision replay passes on stored objects alone
4. Accepted-head selection is deterministic for fixed profiles
5. The local store can be rebuilt from canonical objects alone

### Current Status

Partially underway.

Already in progress or partially implemented:

1. Shared object schema metadata
2. Shared object-envelope parsing
3. Object inspection and verification
4. Accepted-head inspection
5. Internal validation and simulator harness CLI

Still missing or incomplete:

1. Full typed object model across all object families
2. Canonical serialization as a fully shared protocol layer
3. Revision replay engine and complete `state_hash` verification
4. Storage-write and object-authoring path
5. Formal store-rebuild workflow

### Milestones in This Phase

#### M1: Core Object and Validation Base

Focus:

1. shared object schema and parsing
2. canonical object validation rules
3. object inspection and verification tooling
4. interop fixtures and negative validation coverage

Completion gate:

1. all required v0.1 object families can be parsed into a shared protocol layer
2. derived IDs can be recomputed reproducibly
3. required signature rules are enforced consistently
4. CLI and tests expose stable validation and verification surfaces for internal workflows

Current read:

Partially complete.

Already visible in the repo:

1. shared schema metadata
2. shared object-envelope parsing
3. object inspection and verification
4. internal validation and simulator harness coverage

Main remaining gaps:

1. full typed object-family coverage
2. canonical serialization promoted into a clearly shared protocol utility
3. stronger `mycel-core`-level test depth around protocol parsing and verification

Implementation anchors:

1. Crates:
   `crates/mycel-core`
   `crates/mycel-sim`
   `apps/mycel-cli`
2. Key files:
   `crates/mycel-core/src/protocol.rs`
   `crates/mycel-core/src/verify.rs`
   `crates/mycel-core/src/lib.rs`
   `crates/mycel-sim/src/validate.rs`
   `apps/mycel-cli/src/object.rs`
   `apps/mycel-cli/tests/object_verify_smoke.rs`
   `apps/mycel-cli/tests/object_inspect_smoke.rs`
   `apps/mycel-cli/tests/validate_smoke.rs`
3. Useful commands:
   `cargo test -p mycel-core`
   `cargo test -p mycel-cli`
   `cargo run -p mycel-cli -- object inspect <path> --json`
   `cargo run -p mycel-cli -- object verify <path> --json`
   `cargo run -p mycel-cli -- validate <path> --json`

#### M2: Replay, Storage, and Rebuild

Focus:

1. replay-based revision verification
2. `state_hash` recomputation
3. local object-store indexing
4. store rebuild and recovery workflows
5. initial object-authoring and storage-write path

Completion gate:

1. revisions can be replayed deterministically from stored objects
2. `state_hash` is recomputed and verified during replay
3. indexes can be rebuilt from canonical objects alone
4. at least a narrow object creation and write path exists for the first client

Current read:

Started conceptually, but still largely incomplete.

Main remaining gaps:

1. replay engine
2. `state_hash` verification engine
3. persistent local store model
4. object builder and writer path

Implementation anchors:

1. Crates:
   `crates/mycel-core`
   `apps/mycel-cli`
2. Key files:
   `crates/mycel-core/src/verify.rs`
   `crates/mycel-core/src/protocol.rs`
   `IMPLEMENTATION-CHECKLIST.en.md`
   `fixtures/README.md`
3. Expected next code areas:
   replay and `state_hash` logic will likely land first in `crates/mycel-core`
   storage-write and rebuild entry points will likely need new files or modules, not more CLI-only glue
4. Useful commands:
   `cargo test -p mycel-core`
   `cargo run -p mycel-cli -- validate fixtures/object-sets/minimal-valid/fixture.json --json`

## Phase 2: Reader-Plus-Governance

Goal: add a usable reader-oriented client layer with deterministic accepted-head behavior and governance-aware reading state.

### Deliverables

1. Verified View ingestion as governance signal input
2. Stable accepted-head selection for fixed reader profiles
3. Reader-first text rendering from replayed revision state
4. Clear separation between reader workflows and governance publication workflows
5. CLI/API support for inspecting accepted heads, views, and governance decision detail

### Exit Criteria

1. A fixed reader profile produces stable accepted heads across repeated runs
2. Governance inputs are separated from discretionary local policy
3. A reader can reconstruct and inspect accepted text state from stored objects
4. Decision summaries and typed arrays are stable enough for tooling and tests

### Current Status

Early partial progress.

Already in progress or partially implemented:

1. Accepted-head inspection
2. Structured decision output with typed machine-readable arrays
3. Early simulator workflows around peer and topology validation

Still missing or incomplete:

1. Full reader rendering path
2. View publication workflow
3. Stable reader-facing profile selection surface
4. Complete storage and retrieval path for governance inputs

### Milestones in This Phase

#### M3: Reader and Governance Surface

Focus:

1. verified View ingestion
2. fixed-profile accepted-head selection
3. reader-first text reconstruction
4. clear separation between reader inspection and governance publication workflows

Completion gate:

1. a fixed reader profile yields deterministic accepted heads across repeated runs
2. governance data is stored and consumed separately from local discretionary policy
3. reconstructed accepted text can be rendered or inspected from stored objects
4. reader-facing CLI or API surfaces are stable enough for repeated internal use

Current read:

Early partial progress.

Already visible in the repo:

1. accepted-head inspection
2. structured decision detail in typed arrays
3. simulator and validation workflows around peer, topology, test, and report scopes

Main remaining gaps:

1. reader text rendering path
2. fixed-profile reading workflow
3. governance publication workflow
4. broader governance-state persistence

Implementation anchors:

1. Crates:
   `crates/mycel-core`
   `apps/mycel-cli`
2. Key files:
   `crates/mycel-core/src/head.rs`
   `apps/mycel-cli/src/head.rs`
   `apps/mycel-cli/tests/head_inspect_smoke.rs`
   `fixtures/head-inspect/README.md`
3. Useful commands:
   `cargo run -p mycel-cli -- head inspect <doc-id> --input <path-or-fixture> --json`
   `cargo test -p mycel-cli head_inspect`

## Phase 3: Full-Stack

Goal: extend from local verification and governed reading into interoperable replication, richer profiles, and selective app-layer support.

### Deliverables

1. Canonical wire envelope implementation
2. `HELLO`, `MANIFEST`, `HEADS`, `WANT`, `OBJECT`, `BYE`, and `ERROR`
3. Optional `SNAPSHOT_OFFER` and `VIEW_ANNOUNCE` for supported profiles
4. End-to-end sync workflow between peers
5. Merge-generation profile support for local authoring tools
6. Selective app-layer profiles on top of a stable protocol core

### Exit Criteria

1. Minimal sync succeeds end-to-end between peers
2. Received objects are verified before indexing and exposure
3. Merge generation can emit replayable patch operations
4. Profile-specific extensions remain outside the protocol core unless clearly justified

### Current Status

Mostly not started.

Already in progress or partially implemented:

1. Simulator topology and report scaffolding
2. CLI workflows for report inspection, listing, stats, and diffing

Still missing or incomplete:

1. Real wire implementation
2. Object fetch and sync state machine
3. Snapshot-assisted catch-up
4. Production replication behavior
5. App-layer runtime support

### Milestones in This Phase

#### M4: Wire Sync and Peer Interop

Focus:

1. canonical wire envelope
2. minimal message set
3. end-to-end sync between peers
4. verified object ingestion before indexing

Completion gate:

1. `HELLO`, `MANIFEST`, `HEADS`, `WANT`, `OBJECT`, `BYE`, and `ERROR` work end-to-end
2. peers can complete a minimal first-time and incremental sync flow
3. fetched objects are verified before storage and exposure
4. interop fixtures and simulator coverage include sync success and negative sync cases

Current read:

Not started in implementation, but scaffolded in docs and simulator structure.

Implementation anchors:

1. Crates:
   `crates/mycel-sim`
   `apps/mycel-cli`
2. Key files:
   `crates/mycel-sim/src/run.rs`
   `crates/mycel-sim/src/model.rs`
   `crates/mycel-sim/src/manifest.rs`
   `sim/README.md`
   `WIRE-PROTOCOL.en.md`
   `PROTOCOL.en.md`
3. Useful commands:
   `cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json --json`
   `cargo run -p mycel-cli -- report inspect sim/reports/out/three-peer-consistency.report.json --events --json`
   `cargo run -p mycel-cli -- report diff <left> <right> --events --json`

#### M5: Selective App-Layer Expansion

Focus:

1. conservative profile growth above the protocol core
2. selective app-layer support only after the first client is stable
3. authoring and merge-generation workflows where the protocol already supports them

Completion gate:

1. app-layer additions depend on stable core protocol behavior
2. merge generation emits replayable patch operations
3. profile-specific logic stays outside the protocol core unless clearly justified

Current read:

Mostly deferred by design.

Implementation anchors:

1. Design and spec files:
   `docs/design-notes/`
   `PROFILE.fund-auto-disbursement-v0.1.en.md`
   `PROFILE.mycel-over-tor-v0.1.en.md`
   `PROJECT-INTENT.md`
2. Key rule for this milestone:
   mature features should become profiles or schemas before they become protocol-core work

## Cross-Cutting Priorities

These priorities apply across all phases:

1. Keep the first client deliberately narrow
2. Prefer profiles and schemas over frequent protocol-core expansion
3. Keep machine-readable CLI output stable where tests rely on it
4. Add regression coverage whenever a new protocol rule or CLI contract is introduced
5. Preserve the separation between protocol state, governance state, and local discretionary policy

## Immediate Priorities

The highest-value near-term work is:

1. complete `M1` by finishing shared object-family coverage and shared canonical object mechanics
2. begin `M2` with replay, `state_hash`, and store-rebuild foundations
3. keep strengthening interop fixtures and negative tests as each protocol rule lands
4. turn mature governance behavior into fixed reader-profile workflows only after the minimal core is stable

## What Moves a Milestone Forward

A milestone should normally move only when all of these are true:

1. the core behavior exists in `mycel-core` or another shared implementation layer, not only in CLI glue
2. CLI or simulator surfaces expose the behavior in a stable enough form for internal use
3. fixtures or negative tests cover the new rule or behavior
4. the change narrows the first-client path instead of widening the protocol scope prematurely

## Not Yet the Target

The roadmap does not currently treat these as near-term targets:

1. rich editor UX
2. production network deployment
3. generalized app runtime
4. broad plugin systems
5. rapid protocol-core expansion driven by speculative design notes
