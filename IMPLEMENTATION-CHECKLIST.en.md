# Mycel v0.1 Implementation Checklist

Status: late partial progress, refreshed after the recent canonical-helper convergence, peer-store sync-driver, CLI peer-sync, simulator integration, and optional-flow sync coverage batch; implementation state now includes replay `state_hash` helper convergence plus early `M4` peer-driven sync coverage and capability-gated optional-message handling, while broader peer interop and production replication behavior remain open

This checklist translates the v0.1 spec into an implementation-oriented build plan for a minimal interoperable client.

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

## 1. Repo and Build Setup

- [x] Choose one implementation language and package layout.
- [x] Fix one canonical hash algorithm for the network profile.
- [x] Fix one signature algorithm set for the client profile.
- [ ] Finish extending the shared canonical JSON utility across hash, signature, the remaining wire-validation paths, and future wire code.
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
- [ ] Finish the remaining malformed field-shape depth, semantic edge-case, and role-model closure still left after the recent strictness-surface expansion, replay-dependency CLI smoke growth, and ancestry-context proof expansion.
- [ ] Model editor-maintainer and view-maintainer role assignment independently.

## 3. Canonical Serialization and Hashing

- [x] Canonicalize all protocol objects as UTF-8 JSON with no extra whitespace.
- [x] Enforce object-key lexicographic ordering.
- [x] Preserve array order exactly.
- [x] Reject duplicate keys.
- [x] Reject unsupported value types such as `null` or floating-point numbers.
- [x] Omit derived ID fields and `signature` when recomputing object IDs.
- [ ] Finish reusing the same canonicalization rules for the remaining wire-validation paths and future wire envelope signatures.

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
- [ ] Persist local transport and safety policy separately from replicated protocol objects.
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
- [x] If multiple fixed profiles are supported, enumerate them explicitly rather than allowing ad hoc local profiles.
- [x] Ensure editor-maintainer status alone never grants selector weight.
- [ ] If dual-role keys are supported, validate editor-maintainer and view-maintainer admission separately.

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

- [ ] all required object types parse and validate
- [x] canonical IDs and signatures are reproducible
- [x] revision replay and `state_hash` verification pass
- [ ] minimal wire sync succeeds end-to-end
- [x] deterministic head selection produces stable output
- [x] merge generation can emit valid replayable patch operations
- [x] the local store can be rebuilt from canonical objects alone
