# Mycel v0.1 Implementation Checklist

Status: `M1` minimal-client gate closed and retained below as a completed checklist; a post-`M1` follow-up checklist now tracks the still-open `M2` / `M3` / `M4` work, including broader governance persistence, broader peer interop, and production replication behavior

This checklist translates the v0.1 spec into an implementation-oriented build plan for a minimal interoperable client.

It now has two roles:

- Part A records the closed `M1` minimal-client gate and its completed proof points
- Part B tracks the remaining follow-up work for `M2`, `M3`, and `M4`

## 0. Build Target

Target a constrained v0.1 client first:

- one local object store
- one fixed network hash algorithm
- canonical JSON only
- patch / revision / view / snapshot support
- `HELLO`, `MANIFEST`, `HEADS`, `WANT`, `OBJECT`, `BYE`, and `ERROR`
- `SNAPSHOT_OFFER` and `VIEW_ANNOUNCE` if the client advertises those capabilities
- replay-based revision verification
- deterministic, profile-locked head selection
- conservative merge generation profile

Defer if needed:

- rich editor UX
- advanced policy UI
- automatic key discovery
- non-JSON encodings
- custom merge plugins

## Part A. Closed `M1` Minimal-Client Gate

The following sections remain as the historical record of the closed minimal-client gate.

## 1. Repo and Build Setup

- [x] Choose one implementation language and package layout.
- [x] Fix one canonical hash algorithm for the network profile.
- [x] Fix one signature algorithm set for the client profile.
- [x] Finish extending the shared canonical JSON utility across hash, signature, the remaining wire-validation paths, and future wire code.
- [x] Add fixture loading for protocol examples and regression tests.

## 2. Object Types and IDs

- [x] Implement `document` parsing with `doc_id` treated as a logical ID.
- [x] Implement `block` parsing with `block_id` treated as a logical ID.
- [x] Implement `patch` parsing with derived `patch_id`.
- [x] Implement `revision` parsing with derived `revision_id`.
- [x] Implement `view` parsing with derived `view_id`.
- [x] Implement `snapshot` parsing with derived `snapshot_id`.
- [x] Reject any content-addressed object whose embedded derived ID does not match the recomputed canonical ID.
- [x] Reject unknown top-level typed-object fields and invalid required field types in shared parsing and verification.
- [x] Finish the remaining malformed field-shape depth, semantic edge-case, and role-model closure still left after the recent strictness-surface expansion, replay-dependency CLI smoke growth, and ancestry-context proof expansion.
- [x] Model editor-maintainer and view-maintainer role assignment independently for mixed-role and shared-key cases.

## 3. Canonical Serialization and Hashing

- [x] Canonicalize all protocol objects as UTF-8 JSON with no extra whitespace.
- [x] Enforce object-key lexicographic ordering.
- [x] Preserve array order exactly.
- [x] Reject duplicate keys.
- [x] Reject unsupported value types such as `null` or floating-point numbers.
- [x] Omit derived ID fields and `signature` when recomputing object IDs.
- [x] Finish reusing the same canonicalization rules for the remaining wire-validation paths and future wire envelope signatures.

## 4. Signature Verification

- [x] Implement the object signature matrix.
- [x] Forbid signatures on `document` and `block`.
- [x] Require signatures on `patch`, `revision`, `view`, and `snapshot`.
- [x] Verify signatures only after canonical ID checks pass.
- [x] Implement wire envelope signature verification for all v0.1 message types.
- [x] Reject any object or message that fails the profile's required signature checks.

## 5. Patch and Revision Engine

- [x] Implement the v0.1 patch operations:
- [x] `insert_block`
- [x] `insert_block_after`
- [x] `delete_block`
- [x] `replace_block`
- [x] `move_block`
- [x] `annotate_block`
- [x] `set_metadata`
- [x] Enforce that non-genesis patch `base_revision` equals the execution-base revision.
- [x] Support the genesis sentinel `rev:genesis-null`.
- [x] Apply revision `patches` strictly in array order.
- [x] Treat `parents[0]` as the only execution base state.
- [x] Treat `parents[1..]` as ancestry-only unless content is materialized by explicit patch operations.
- [x] Recompute and verify `state_hash` for every received revision.
- [x] Keep revision publication authority separate from accepted-head governance weight.

## 6. Local State and Storage

- [x] Store all received objects by canonical `object_id`.
- [x] Maintain an index for `doc_id -> revisions`.
- [x] Maintain an index for `revision_id -> parents`.
- [x] Maintain an index for `author -> patches`.
- [x] Maintain an index for `view_id -> governance signal contents`.
- [x] Maintain an index for `profile_id -> selected document heads`.
- [x] Persist local transport and safety policy separately from replicated protocol objects.
- [x] Keep discretionary local policy out of the active accepted-head path.
- [x] Support rebuilding indexes from the object store alone.

## 7. Wire Protocol

- [x] Implement the canonical wire envelope.
- [x] Validate `type`, `version`, `msg_id`, `timestamp`, `from`, `payload`, and `sig`.
- [x] Enforce RFC 3339 timestamps on wire messages.
- [x] Implement `HELLO`.
- [x] Implement `MANIFEST`.
- [x] Implement `HEADS`.
- [x] Implement `WANT`.
- [x] Implement `OBJECT`.
- [x] Implement `BYE`.
- [x] Implement `ERROR`.
- [x] Implement `SNAPSHOT_OFFER` only if `snapshot-sync` is advertised.
- [x] Implement `VIEW_ANNOUNCE` only if `view-sync` is advertised.
- [x] Recompute `hash(body)` for every `OBJECT`.
- [x] Reconstruct expected `object_id` from `object_type` and `hash`.
- [x] Reject any `OBJECT` whose embedded derived ID disagrees with the envelope `object_id`.

## 8. Sync Workflow

- [x] Support first-time sync end-to-end between peers: `HELLO` -> `MANIFEST` / `HEADS` -> `WANT` -> `OBJECT`.
- [x] Support incremental sync from updated `HEADS` between peers.
- [x] Fetch missing objects only by canonical object ID.
- [x] Verify objects before indexing or exposing them to readers.
- [x] Support snapshot-assisted catch-up if snapshots are advertised.
- [x] Support fetching announced views if `view-sync` is enabled.
- [x] Treat fetched View objects as governance signals rather than user preference state.

## 9. Views and Head Selection

- [x] Store verified `view` objects as governance signals, separately from local transport/safety policy state.
- [x] Group selector inputs by `profile_id`, `doc_id`, and `effective_selection_time`.
- [x] Resolve `profile_id` as a fixed `policy_hash` for the active reader profile.
- [x] Compute eligible heads exactly as specified.
- [x] Use only verified View objects with matching `policy_hash` as view-maintainer signals.
- [x] Implement selector epoch calculation exactly.
- [x] Implement the normative `selector_score`.
- [x] Implement the normative tie-break order.
- [x] Emit or persist the minimum decision trace schema.
- [x] Do not expose discretionary local policy controls that alter the active accepted head.
- [x] If multiple fixed profiles are supported, enumerate and surface them explicitly rather than allowing ad hoc local profiles.
- [x] Ensure editor-maintainer status alone never grants selector weight.
- [x] If viewer signals can influence `selector_score`, model them as bounded, typed score channels with capped viewer bonus / penalty paths rather than raw popularity counts.
- [x] If viewer signals can influence `selector_score`, define typed `approval`, `objection`, and `challenge` signals with evidence and expiry semantics.
- [x] If viewer signals can influence `selector_score`, gate eligibility and effective signal weight through explicit anti-Sybil, admission, or reputation rules.
- [x] If viewer signals can influence `selector_score`, expose viewer contribution in stable typed arrays and traces without collapsing maintainer governance into raw public preference.
- [x] If dual-role keys are supported, validate editor-maintainer and view-maintainer admission separately.

## 10. Merge Generation

- [x] Keep revision verification replay-based; do not require receivers to rerun merge generation.
- [x] Implement the conservative merge generation profile for local authoring tools.
- [x] Distinguish `Auto-merged`, `Multi-variant`, and `Manual-curation-required`.
- [x] Materialize merge results as ordinary patch operations.
- [x] Reject hidden merge metadata as a substitute for explicit state changes.

## 11. CLI or API Surface

- [x] Provide a local init command or API.
- [x] Provide object verification tooling.
- [x] Provide document creation and patch authoring entry points.
- [x] Provide revision commit entry points.
- [x] Provide sync pull entry points.
- [x] Provide view inspection or head-inspection entry points.
- [x] Provide accepted-head render entry points from stored objects or explicit object bundles.
- [x] Separate reader-facing accepted-head inspection from curator-facing View publication workflows.
- [x] Provide distinct `human` and `debug` text output modes for accepted-head inspection/render while keeping JSON and typed arrays as the machine-stable detail surface.
- [x] Keep head-inspection `decision_trace` at a high-level summary layer only.
- [x] Put machine-consumable maintainer, weight, and violation details in typed arrays such as `effective_weights[]`, `maintainer_support[]`, and `critical_violations[]`, not in `decision_trace`.
- [x] Treat `decision_trace` as explanatory output for humans; treat typed arrays as the stable detail surface for tools and tests.
- [x] Separate editor-maintainer revision publication from view-maintainer governance publication workflows.
- [x] Provide store-rebuild or reindex entry points for recovery.

## 12. Interop Test Minimum

- [x] Load all normative example objects and ensure they parse.
- [x] Recompute derived IDs for example `patch`, `revision`, `view`, and `snapshot` objects.
- [x] Recompute `state_hash` for at least one single-parent revision and one merge revision.
- [x] Verify example wire envelopes and `OBJECT` validation behavior.
- [x] Add negative tests for hash mismatch, signature mismatch, and invalid parent ordering.
- [x] Add a round-trip test for canonical serialization.
- [x] Add a replay test that rebuilds document state from stored objects only.

## 13. Ready-to-Build Gate

Treat the client as ready for a first interoperable build when all of the following are true:

- [x] all required object types parse and validate
- [x] canonical IDs and signatures are reproducible
- [x] revision replay and `state_hash` verification pass
- [x] minimal wire sync succeeds end-to-end
- [x] deterministic head selection produces stable output
- [x] merge generation can emit valid replayable patch operations
- [x] the local store can be rebuilt from canonical objects alone

## Part B. Post-`M1` Follow-Up Checklist

Use this section as the active implementation checklist for the still-open post-`M1` lane.

## 14. `M2` Replay, Storage, and Rebuild Follow-Up

- [x] Broaden persisted-store index reuse across reader and recovery workflows so accepted-head and render paths rely less on ad hoc CLI-only glue.
- [x] Add stronger replay and store-rebuild fixture coverage beyond the current direct proof points, including more realistic multi-document and recovery-oriented fixture sets.
- [x] Move more authoring and replay helper ownership into `mycel-core` so storage-write and replay behavior are not disproportionately CLI-driven.
- [ ] Expand conservative merge-authoring coverage for richer nested and reparenting conflict cases that still fall back to manual curation.
- [x] Define and verify the intended narrow object-authoring and storage-write path that remains open after the closed minimal-client gate.

## 15. `M3` Reader and Governance Follow-Up

- [ ] Add broader governance persistence beyond the current initial reverse-index and inspect/list/publish surfaces.
- [ ] Extend governance tooling past the current initial `view inspect` / `view list` / `view publish` workflows.
- [ ] Keep improving reader profile ergonomics beyond the current available-profile summaries and profile-error feedback.
- [ ] Close the remaining independent dual-role role-assignment follow-up that still remains after separate admission validation landed.

## 16. `M4` Wire Sync and Peer Interop Follow-Up

- [x] Broaden peer-interop proof beyond the current peer-store-driven first-time and incremental sync coverage.
- [x] Add localhost multi-process or equivalent transport proof so the current sync path is not validated only through narrow transcript or simulator-controlled paths.
- [x] Define and test the missing production replication behavior that still sits outside the current minimal sync proof. Scope: three specific sub-items below.
  - [x] Re-sync idempotency: running sync twice when the reader is already current produces zero new stored objects, no errors, and stable accepted heads.
  - [x] Depth-N incremental catchup: a reader at revision depth 1 catches up to a seed at depth ≥ 3 in a single HEADS/WANT pass, verifying that only the delta is fetched.
  - [x] Partial-doc selective sync: a reader requests only a subset of the seed's documents, ends with a stable partial store, and accepted heads are correct for the requested subset only (PROTOCOL §8 states partial replication is supported).
- [ ] Expand session, capability, and error-path interop coverage past the current positive-path and optional-message proof set.

## 17. Cross-Surface Closure Rules

- [ ] Keep `ROADMAP.md`, `ROADMAP.zh-TW.md`, and `docs/PROGRESS.md` aligned whenever any post-`M1` checklist section changes status.
- [x] Open or refresh narrowly-scoped GitHub Issues for durable follow-up gaps rather than leaving post-`M1` work only as summary prose.
