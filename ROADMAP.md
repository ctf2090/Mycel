# Mycel Roadmap

Status: late partial progress, refreshed after the recent shared canonical-helper consolidation, top-level core-version strictness closure, path-preserving nested parser errors, replay dependency verification tightening, and sibling ID determinism batch; milestone state unchanged

This roadmap turns the current README priorities, implementation checklist, and design-note planning guidance into one repo-level build sequence.

It is intentionally narrow:

- build the first interoperable client first
- keep protocol-core changes conservative
- move mature ideas into profiles, schemas, and tests before expanding scope

## Current Position

The repository already has:

- a growing v0.1 protocol and wire-spec document set
- a Rust CLI suitable for internal validation and deterministic simulator workflows
- `mycel-core` support for object schema metadata, object-envelope parsing, replay-based revision verification, local object-store ingest/rebuild, persisted store indexes, and accepted-head inspection
- more centralized canonical hash and signed-payload helpers reused across verification, replay, and authoring paths
- early reader-plus-governance surfaces for accepted-head rendering, named fixed-profile selection, and editor-admission-aware inspect/render workflows
- broader parser / verify / CLI strictness-surface coverage for `document`, `block`, `patch`, `revision`, `view`, and `snapshot`, a materially wider `object inspect` warning surface, stronger signature-edge and replay/verification smoke coverage for merge and cross-document revision edges, and isolated validate-peer fixtures
- a more maintainable CLI test base with `assert_cmd`, `predicates`, `tempfile`, and small `rstest` use on high-duplication strictness matrices
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
2. close the remaining shared-core gaps in parsing and canonicalization
3. keep expanding fixtures, simulator coverage, and negative tests while beginning reader-plus-governance read paths

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

Late partial progress, approaching the end of the phase but not ready to declare complete.

Already in progress or partially implemented:

1. Shared object schema metadata
2. Shared object-envelope parsing
3. Object inspection and verification
4. Replay-based revision verification and `state_hash` checking
5. Local object-store ingest, rebuild, persisted manifest indexing, and query surfaces
6. Accepted-head inspection, including store-backed selector object loading
7. Internal validation and simulator harness CLI

Still missing or incomplete:

1. Final closure work around malformed field-shape depth, remaining inspect-surface parity polish, and remaining semantic-edge strictness
2. Narrow object-authoring and write path beyond verified ingest into the store
3. A cleaner reader-facing profile surface on top of the accepted-head selector
4. Shared canonicalization reuse extended into future wire-envelope work
5. Final closure work that would justify marking Phase 1 exit criteria as complete

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

Nearly complete. The shared parsing, canonical helper, top-level core-version equality checks, path-preserving nested parser field errors, broad parser / verify / CLI strictness-surface coverage, broader inspect-surface parity, stronger replay dependency verification and sibling declared-ID determinism, stronger signature-edge and replay/verification smoke coverage for revision semantics, isolated validate-peer fixtures, and canonical reproducibility coverage now exist; the remaining work is mostly the last malformed-field depth and semantic-edge closure plus a few milestone-close proof points.

Already visible in the repo:

1. shared schema metadata
2. shared object-envelope parsing
3. shared canonical JSON, derived-ID recomputation, and signed-payload helpers
4. object inspection and verification
5. protocol-level typed parsing for the supported object families, including `document`, `block`, `patch`, `revision`, `view`, and `snapshot`
6. duplicate-key rejection and unsupported-value rejection in shared JSON loading
7. canonical round-trip and reproducibility coverage for IDs, signed payloads, and signatures
8. internal validation and simulator harness coverage

Main remaining gaps:

1. final malformed-field depth and semantic-edge strictness closure after broad unknown-field and invalid-type rejection
2. deeper `mycel-core`-level coverage for the remaining semantic edge cases outside the current revision / patch, replay, and view / snapshot batches
3. shared helper reuse extended into future wire-validation work
4. clearer milestone-close criteria before widening more surfaces

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

Recommended build order:

1. finish shared protocol parsing coverage for all required object families in `crates/mycel-core/src/protocol.rs`
2. move canonical object mechanics into shared protocol-level helpers instead of leaving them only inside `crates/mycel-core/src/verify.rs`
3. extend `crates/mycel-core/src/verify.rs` to consume those shared helpers for every supported object family
4. deepen `mycel-core` tests before expanding CLI surface, so object-rule regressions are caught below the CLI layer
5. only after the shared core is stable, widen CLI and simulator-facing validation coverage where needed

First implementation batch:

Completed in the current repo state:

1. typed parsing coverage for `document` and `block` logical-ID handling in `crates/mycel-core/src/protocol.rs`
2. typed parsing coverage for `patch`, `revision`, `view`, and `snapshot` derived-ID fields in `crates/mycel-core/src/protocol.rs`
3. shared protocol-level canonical JSON, derived-ID recomputation, and signed-payload helpers extracted from verification-only ownership
4. `crates/mycel-core/src/verify.rs` consuming the shared typed parsing and canonical helpers for every supported object family
5. `mycel-core` tests for malformed object type, missing signer fields, wrong derived-ID fields, duplicate keys, unsupported values, and malformed field-shape cases before widening more CLI behavior

Concrete completion check for this batch:

Completed:

1. `protocol.rs` understands every currently supported object family through one shared parsing layer.
2. `verify.rs` no longer owns the only copy of canonical object mechanics.
3. `cargo test -p mycel-core` provides direct coverage for shared protocol helpers and object-family edge cases.
4. Existing `object inspect` and `object verify` CLI contracts still pass without needing CLI-only fallback logic.

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

Substantially underway. Replay-based verification, store rebuild, persisted indexes, a narrow store write path, and an initial conservative merge-authoring workflow now exist, but the milestone is still not closeable.

Main remaining gaps:

1. broader reuse of persisted store indexes across reader workflows
2. broader replay and store reconstruction coverage tied to more realistic fixture sets beyond the current direct store-backed replay proof point
3. conservative merge authoring now covers basic move/reorder, insert/delete composition, reparenting into newly introduced parents, simple composed parent-chain reparenting, and a broader initial nested structural matrix, but richer nested/reparenting conflict cases still require manual curation
4. broader core reuse so authoring and replay helpers do not remain disproportionately CLI-driven

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

Recommended build order:

1. land replay primitives in `crates/mycel-core` before building any new storage-writing CLI flow
2. implement deterministic `state_hash` recomputation on top of replay, not as a separate isolated utility
3. define the minimal local store and rebuild model once replay output is stable
4. add a narrow object builder and storage-write path only after replay and rebuild semantics are fixed
5. expose CLI or API entry points last, so they sit on top of shared replay and storage logic instead of inventing parallel behavior

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

Early partial progress, now with accepted-head rendering, named fixed-profile selection, and editor-admission-aware inspect/render behavior on top of the deterministic selector path.

Already in progress or partially implemented:

1. Accepted-head inspection
2. Structured decision output with typed machine-readable arrays
3. Store-backed accepted-head inspection using persisted store indexes
4. Accepted-head render output from persisted store state or explicit bundle objects
5. Named fixed-profile selection for accepted-head inspection and render workflows
6. Editor-admission-aware accepted-head inspect/render behavior for named-profile and store-backed paths
7. Dedicated `view inspect` / `view list` / `view publish` governance workflows alongside reader-facing `head` commands
8. Persisted governance reverse indexes for maintainer, profile, and document view lookups
9. Early simulator workflows around peer and topology validation

Still missing or incomplete:

1. Broader governance-state persistence beyond selector, reverse view indexes, and replay inputs
2. Reader-facing profile ergonomics beyond the minimal named fixed-profile surface
3. Richer governance retrieval and publication surfaces beyond the initial filtered/sorted/projected `view` inspection/listing/publication surface
4. Stronger dedicated governance-state tooling once wire and sync work begin to land

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

Early partial progress, now with accepted-head render support from persisted stores and explicit replay bundles, plus editor-admission-aware named-profile and store-backed flows.

Already visible in the repo:

1. accepted-head inspection
2. structured decision detail in typed arrays
3. store-backed selector object loading for accepted-head inspection
4. accepted-head rendering from persisted store state or explicit bundle objects
5. named fixed-profile selection for accepted-head inspection and render workflows
6. editor-admission-aware inspect/render behavior for named-profile and store-backed reader flows
7. dedicated `view inspect` / `view list` / `view publish` governance workflows alongside reader-facing `head` commands, with filtered listing, sorting, time windows, grouped summaries, and projection modes
8. persisted governance reverse indexes for maintainer, profile, and document-oriented view lookups
9. simulator and validation workflows around peer, topology, test, and report scopes

Main remaining gaps:

1. broader governance-state persistence beyond the current reverse governance indexes, plus dedicated inspection surfaces
2. stronger dedicated governance inspection and publication surfaces beyond the initial `view` workflow
3. reader-facing profile ergonomics beyond the minimal named fixed-profile surface
4. governance-state tooling that can later align with wire/sync transport

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
   `cargo run -p mycel-cli -- head render <doc-id> --input <path-or-fixture> --json`
   `cargo run -p mycel-cli -- view inspect <view-id> --store-root <store> --json`
   `cargo run -p mycel-cli -- view list --store-root <store> --latest-per-profile --limit 10 --summary-only --group-by profile-id --json`
   `cargo run -p mycel-cli -- view publish <path> --into <store> --json`
   `cargo run -p mycel-cli -- store index <store> --governance-only --maintainer <maintainer> --json`
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
3. A conservative local merge-authoring workflow that emits replayable patch operations for narrow resolved-state merges

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
