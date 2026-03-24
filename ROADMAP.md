# Mycel Roadmap

Status: major progress, refreshed after the implementation checklist was split into a closed `M1` minimal-client gate plus a live post-`M1` follow-up checklist; `M2` replay/storage/rebuild closure is now landed at the current narrow scope, so the active lane now centers on `M3` / `M4` while broader governance persistence, richer governance tooling, reader-facing profile ergonomics, final independent dual-role closure, and the remaining peer interop session/capability/error-path proof stay open after the current production replication sub-items were completed, the first permanent messages-after-BYE session proof landed, `HEADS`-before-`MANIFEST` sync-root setup plus stale root/dependency and stale snapshot `WANT` rejection after `HEADS replace=true` landed, unknown-sender and HELLO sender-identity mismatch rejection plus explicit `ERROR`-only and unreachable `WANT` fault proofs landed, and per-document current-governance summaries were added to the current M3 baseline

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
- `mycel-core` support for early wire-envelope parsing, payload validation, generic wire-signature verification, sender mapping, inbound session sequencing/head-tracking, reachability gating, and store-backed session bootstrap for the minimal message set
- a transcript-backed sync-pull core, peer-store sync driver, and CLI entry points with first-time and incremental verify/store coverage, including capability-gated `SNAPSHOT_OFFER` and `VIEW_ANNOUNCE` flows
- more centralized canonical hash and signed-payload helpers reused across verification, replay `state_hash`, head/render pre-verification, authoring, and wire-object identity checks
- early reader-plus-governance surfaces for accepted-head rendering, named fixed-profile selection, and editor-admission-aware inspect/render workflows
- broader parser / verify / CLI strictness-surface coverage for `document`, `block`, `patch`, `revision`, `view`, and `snapshot`, a materially wider `object inspect` warning surface, stronger signature-edge and replay/verification smoke coverage for merge and cross-document revision edges, clearer multi-hop ancestry context in replay-derived failures, and isolated validate-peer fixtures
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

1. keep `M2` closed at the current narrow replay/storage/rebuild scope now that the richer mixed content/metadata competing-branch rebuild-and-reporting proof is landed
2. expand `M3` reader-plus-governance workflows without reopening the closed minimal-client gate while keeping broader governance persistence, richer governance tooling, reader-facing profile ergonomics, and final independent dual-role closure explicit
3. advance `M4` from peer-store proof toward the remaining peer-interop session/capability/error-path coverage now that the currently tracked production replication sub-items are proved and the current negative-proof baseline includes permanent messages-after-BYE rejection, `HEADS`-before-`MANIFEST` sync-root setup, stale root/dependency and stale snapshot `WANT` rejection after `HEADS replace=true`, sender-validation faults, explicit `ERROR`-only failure, and unreachable `WANT` rejection

### Next

After the narrow core is stable, the next lane is:

1. broader `M3` governance persistence, richer governance tooling, reader-facing profile ergonomics, and final independent dual-role closure on top of the current `view inspect` / `view list` / `view publish`, persisted-relationship summaries, and per-document current-governance summary baseline
2. the remaining `M4` session, capability, and error-path interop proof beyond the current positive-path and optional-message set
3. reader-facing text reconstruction and presentation refinements only after the current governance and interop baselines are more stable

### Later

The later lane is:

1. canonical wire sync beyond the current peer-store-driven proof surface
2. end-to-end peer replication on top of a stabilized interop core
3. selective app-layer expansion on top of a stable protocol core and sync baseline

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

Phase 1 exit criteria are now fully satisfied. The Ready-to-Build Gate in `IMPLEMENTATION-CHECKLIST.en.md` remains all green (7/7 items), and the checklist now retains that gate as a closed historical section while tracking active post-`M1` follow-up work separately.

Already complete:

1. Shared object schema metadata
2. Shared object-envelope parsing
3. Object inspection and verification
4. Replay-based revision verification and `state_hash` checking
5. Local object-store ingest, rebuild, persisted manifest indexing, and query surfaces
6. Accepted-head inspection, including store-backed selector object loading
7. Internal validation and simulator harness CLI
8. Malformed field-shape depth and semantic-edge strictness closure (dual-role key, depth validation)
9. Canonical JSON reuse confirmed across all wire-validation paths

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

Complete. All implementation checklist items are marked `[x]`. The shared parsing, converged canonical helper module, top-level core-version equality checks, path-preserving nested parser field errors, broad parser / verify / CLI strictness-surface coverage, broader inspect-surface parity, stronger replay dependency verification and sibling declared-ID determinism, direct CLI smoke coverage for invalid sibling/parent dependency IDs and signatures, clearer multi-hop ancestry failure context, isolated validate-peer fixtures, canonical reproducibility coverage, field-shape depth and semantic-edge closure for all object families, dual-role key closure with independent role-assignment validation, and canonical JSON reuse confirmed across all wire-validation paths now all exist.

Already complete in the repo:

1. shared schema metadata
2. shared object-envelope parsing
3. shared canonical JSON, derived-ID recomputation, and signed-payload helpers — reused by all wire-validation paths
4. object inspection and verification
5. protocol-level typed parsing for the supported object families, including `document`, `block`, `patch`, `revision`, `view`, and `snapshot`
6. duplicate-key rejection and unsupported-value rejection in shared JSON loading
7. canonical round-trip and reproducibility coverage for IDs, signed payloads, and signatures
8. internal validation and simulator harness coverage
9. field-shape depth validation, semantic-edge strictness, and dual-role key closure

Main remaining gaps:

None that block M1 exit. Phase 2 work (M2/M3) is substantially underway.

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

Closed for the current narrow scope. Replay-based verification, store rebuild, persisted indexes, explicit CLI smoke proof that multi-document indexes can be rebuilt after index loss from stored canonical objects, a narrow store write path, an initial conservative merge-authoring workflow, ancestry-context-preserving render/store verification, scoped document-level index reuse in author and merge workflows, a persisted `doc_heads` index for sync, richer mixed content/metadata competing-branch classification with matching CLI smoke coverage, and rebuild-after-index-loss proof for the richer metadata multi-variant merge case now all exist.

Main remaining gaps:

1. None that block the current narrow `M2` milestone. Future merge-authoring expansion can stay scoped as later follow-up instead of active `M2` closure debt.

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

Early partial progress, now with accepted-head rendering, named fixed-profile selection, clearer available-profile discovery and profile-error feedback, editor-admission-aware inspect/render behavior, distinct human/debug text output modes for `head inspect` / `head render`, bounded viewer score surfaces in head inspection, persisted governance relationship summaries exposed through both `view inspect` and `view list`, and per-document current-governance summaries in `view current` on top of the deterministic selector path; `M3` still remains open for broader governance persistence, richer governance tooling beyond the current inspect/list/publish base, reader-facing profile ergonomics beyond this initial polish, and final independent dual-role role-assignment closure.

Already in progress or partially implemented:

1. Accepted-head inspection
2. Structured decision output with typed machine-readable arrays
3. Store-backed accepted-head inspection using persisted store indexes
4. Accepted-head render output from persisted store state or explicit bundle objects
5. Named fixed-profile selection for accepted-head inspection and render workflows, including clearer available-profile summaries and symmetric profile-error feedback
6. Editor-admission-aware accepted-head inspect/render behavior for named-profile and store-backed paths
7. Dedicated `view inspect` / `view list` / `view publish` governance workflows alongside reader-facing `head` commands
8. Persisted governance reverse indexes for maintainer, profile, and document view lookups
9. Persisted governance relationship summaries surfaced through both `view inspect` and `view list`
10. Per-document current-governance summaries surfaced through `view current`
11. Early simulator workflows around peer and topology validation
12. Bounded viewer score channels in head inspection, including typed signal summaries, anti-Sybil gating, challenge review/freeze pressure, and fixture-backed coverage

Still missing or incomplete:

1. Broader governance-state persistence beyond selector, reverse view indexes, and replay inputs
2. Reader-facing profile ergonomics beyond the minimal named fixed-profile surface
3. Richer governance retrieval and publication surfaces beyond the initial filtered/sorted/projected `view` inspection/listing/publication surface
4. Stronger dedicated governance-state tooling once wire and sync work begin to land
5. Final independent editor-maintainer / view-maintainer role-assignment closure for mixed-role and shared-key cases, plus any later decision about promoting viewer inputs beyond the current head-inspect-local bundle surface

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

Early partial progress, now with accepted-head render support from persisted stores and explicit replay bundles, clearer available-profile discovery and profile-error feedback, editor-admission-aware named-profile and store-backed flows, bounded viewer score surfaces in head inspection, persisted governance relationship summaries exposed through `view inspect` and `view list`, and per-document current-governance summaries exposed through `view current`; broader governance persistence, richer governance tooling beyond the current inspect/list/publish base, reader-facing profile ergonomics beyond this initial polish, and final independent dual-role role-assignment closure remain.

Already visible in the repo:

1. accepted-head inspection
2. structured decision detail in typed arrays
3. store-backed selector object loading for accepted-head inspection
4. accepted-head rendering from persisted store state or explicit bundle objects
5. named fixed-profile selection for accepted-head inspection and render workflows, including clearer available-profile summaries and symmetric profile-error feedback
6. editor-admission-aware inspect/render behavior for named-profile and store-backed reader flows
7. distinct `human` and `debug` text output modes for `head inspect` / `head render`, keeping high-level decision summaries separate from debug trace detail
8. dedicated `view inspect` / `view list` / `view publish` governance workflows alongside reader-facing `head` commands, with filtered listing, sorting, time windows, grouped summaries, and projection modes
9. persisted governance reverse indexes for maintainer, profile, and document-oriented view lookups
10. persisted governance relationship summaries surfaced through `view inspect` and `view list`
11. per-document current-governance summaries surfaced through `view current`
12. simulator and validation workflows around peer, topology, test, and report scopes
13. bounded viewer score channels in head inspection, including typed signal summaries, anti-Sybil gating, challenge review/freeze pressure, and fixture-backed coverage

Main remaining gaps:

1. broader governance-state persistence beyond the current reverse governance indexes, plus dedicated inspection surfaces
2. stronger dedicated governance inspection and publication surfaces beyond the initial `view` workflow
3. reader-facing profile ergonomics beyond the minimal named fixed-profile surface
4. governance-state tooling that can later align with wire/sync transport
5. final independent editor-maintainer / view-maintainer role-assignment closure for mixed-role and shared-key cases, plus any broader governance persistence or governance-tooling follow-up we would need before moving beyond the current head-inspect-local viewer signal surface

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
3. Capability-gated optional `SNAPSHOT_OFFER` and `VIEW_ANNOUNCE` support for supported profiles
4. End-to-end sync workflow between peers
5. Merge-generation profile support for local authoring tools
6. Selective app-layer profiles on top of a stable protocol core

### Exit Criteria

1. Minimal sync succeeds end-to-end between peers
2. Received objects are verified before indexing and exposure
3. Merge generation can emit replayable patch operations
4. Profile-specific extensions remain outside the protocol core unless clearly justified

### Current Status

Early partial.

Already in progress or partially implemented:

1. Simulator topology and report scaffolding
2. CLI workflows for report inspection, listing, stats, and diffing
3. A conservative local merge-authoring workflow that emits replayable patch operations for narrow resolved-state merges
4. `mycel-core` wire-envelope parsing, payload validation, RFC 3339 timestamp checks, signature verification, sender identity checks, and inbound session sequencing for the minimal message set

Still missing or incomplete:

1. Wiring `OBJECT` body-derived hash and object-ID recomputation into the main incoming verification path
2. Object fetch and sync state machine
3. Snapshot-assisted catch-up and capability-gated optional message handling
4. Broader session, capability, and error-path interop proof beyond the current positive-path and optional-message set
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

Substantially underway. All M4 completion-gate items are now satisfied at the simulator level. Canonical envelope parsing, payload-shape validation, RFC 3339 timestamp enforcement, generic wire-signature verification, sender checks, inbound sequencing/head-tracking, reachability gating, store-backed session bootstrap, and `OBJECT` body-derived hash / `object_id` verification exist in `mycel-core`. A peer-store-driven sync path, CLI entry points, and 9 simulator scenarios now prove the full required coverage:

1. First-time sync with multi-peer convergence (`three-peer-consistency`)
2. Incremental HEADS-based sync for follow-up revisions (`incremental-sync`)
3. WANT-based recovery from partial store (`partial-want-recovery`, `mixed-reader-recovery`)
4. Negative: hash-mismatch rejection, signature-mismatch rejection
5. Capability-gated `SNAPSHOT_OFFER` delivery (`snapshot-catchup`)
6. Capability-gated `VIEW_ANNOUNCE` delivery for governance views (`view-sync`)
7. Per-peer accepted-head comparison surfaced in report (`matching-accepted-heads` outcome)
8. Localhost multi-process transport proof via `mycel sync stream | mycel sync pull --transcript -` (`localhost-multi-process`)

What is still missing is broader session/capability/error-path interop closure. Re-sync idempotency is now proved: running sync twice when already current produces zero new writes. Depth-N incremental catchup is now proved: a reader at revision depth 2 catches up to a depth-3 seed in a single HEADS/WANT pass, fetching only the delta. Partial-doc selective sync is now also proved: a reader can request only a subset of the seed's documents, maintain a stable partial store, and compute accepted heads only for the requested subset, matching PROTOCOL §8 partial replication support. The landed negative proof set is also materially broader now: it covers missing-capability rejection for `SNAPSHOT_OFFER` / `VIEW_ANNOUNCE`, pre-`HELLO` rejection for `MANIFEST`, `HEADS`, `WANT`, `BYE`, `SNAPSHOT_OFFER`, and `VIEW_ANNOUNCE`, duplicate-`HELLO`, unknown-sender rejection, HELLO sender-identity mismatch rejection, explicit `ERROR`-only transcript failure, pre-root `WANT` rejection for generic, snapshot, and announced-view fetches, stale root/dependency and stale snapshot `WANT` rejection after `HEADS replace=true`, unreachable `WANT` revision/object rejection outside accepted sync roots, immediate `OBJECT` rejection before accepted sync roots exist, and permanent messages-after-`BYE` rejection. The remaining M4 gap is therefore the next broader set of session/capability/error-path interop faults such as advertised-root/root-set violations and other post-`HELLO` protocol-state errors rather than the already-landed sequencing, sender-validation, reachability, and optional-message cases. The localhost proof also confirms the current wire flow works across real process boundaries instead of only inside transcript fixtures or in-process simulator hooks.

Implementation anchors:

1. Crates:
   `crates/mycel-core`
   `crates/mycel-sim`
   `apps/mycel-cli`
2. Key files:
   `crates/mycel-core/src/wire.rs`
   `crates/mycel-core/src/signature.rs`
   `crates/mycel-sim/src/run.rs`
   `crates/mycel-sim/src/model.rs`
   `crates/mycel-sim/src/manifest.rs`
   `sim/README.md`
   `WIRE-PROTOCOL.en.md`
   `PROTOCOL.en.md`
3. Useful commands:
   `cargo test -p mycel-core wire::`
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

1. keep expanding `M3` with narrow governance-persistence, governance-tooling, profile-ergonomics, and dual-role follow-up slices without reopening the closed minimal-client gate
2. keep strengthening `M4` with additional deterministic session, capability, and error-path interop proofs now that the currently tracked production replication sub-items are landed
3. continue strengthening interop fixtures and negative tests as each remaining rule or follow-up slice lands
4. preserve the now-closed `M2` proof surface while future follow-up work lands around it

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
