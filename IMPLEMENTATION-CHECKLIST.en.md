# Mycel v0.1 Implementation Checklist

Status: draft

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

- [ ] Choose one implementation language and package layout.
- [ ] Fix one canonical hash algorithm for the network profile.
- [ ] Fix one signature algorithm set for the client profile.
- [ ] Add a canonical JSON utility shared by hash, signature, and wire code.
- [ ] Add fixture loading for protocol examples and regression tests.

## 2. Object Types and IDs

- [ ] Implement `document` parsing with `doc_id` treated as a logical ID.
- [ ] Implement `block` parsing with `block_id` treated as a logical ID.
- [ ] Implement `patch` parsing with derived `patch_id`.
- [ ] Implement `revision` parsing with derived `revision_id`.
- [ ] Implement `view` parsing with derived `view_id`.
- [ ] Implement `snapshot` parsing with derived `snapshot_id`.
- [ ] Reject any content-addressed object whose embedded derived ID does not match the recomputed canonical ID.
- [ ] Reject unknown required fields or invalid field types according to the chosen strictness policy.

## 3. Canonical Serialization and Hashing

- [ ] Canonicalize all protocol objects as UTF-8 JSON with no extra whitespace.
- [ ] Enforce object-key lexicographic ordering.
- [ ] Preserve array order exactly.
- [ ] Reject duplicate keys.
- [ ] Reject unsupported value types such as `null` or floating-point numbers.
- [ ] Omit derived ID fields and `signature` when recomputing object IDs.
- [ ] Reuse the same canonicalization rules for `state_hash` and wire envelope signatures.

## 4. Signature Verification

- [ ] Implement the object signature matrix.
- [ ] Forbid signatures on `document` and `block`.
- [ ] Require signatures on `patch`, `revision`, `view`, and `snapshot`.
- [ ] Verify signatures only after canonical ID checks pass.
- [ ] Implement wire envelope signature verification for all v0.1 message types.
- [ ] Reject any object or message that fails the profile's required signature checks.

## 5. Patch and Revision Engine

- [ ] Implement the v0.1 patch operations:
- [ ] `insert_block`
- [ ] `insert_block_after`
- [ ] `delete_block`
- [ ] `replace_block`
- [ ] `move_block`
- [ ] `annotate_block`
- [ ] `set_metadata`
- [ ] Enforce that non-genesis patch `base_revision` equals the execution-base revision.
- [ ] Support the genesis sentinel `rev:genesis-null`.
- [ ] Apply revision `patches` strictly in array order.
- [ ] Treat `parents[0]` as the only execution base state.
- [ ] Treat `parents[1..]` as ancestry-only unless content is materialized by explicit patch operations.
- [ ] Recompute and verify `state_hash` for every received revision.

## 6. Local State and Storage

- [ ] Store all received objects by canonical `object_id`.
- [ ] Maintain an index for `doc_id -> revisions`.
- [ ] Maintain an index for `revision_id -> parents`.
- [ ] Maintain an index for `author -> patches`.
- [ ] Maintain an index for `view_id -> governance signal contents`.
- [ ] Maintain an index for `profile_id -> selected document heads`.
- [ ] Persist local transport and safety policy separately from replicated protocol objects.
- [ ] Keep discretionary local policy out of the active accepted-head path.
- [ ] Support rebuilding indexes from the object store alone.

## 7. Wire Protocol

- [ ] Implement the canonical wire envelope.
- [ ] Validate `type`, `version`, `msg_id`, `timestamp`, `from`, `payload`, and `sig`.
- [ ] Enforce RFC 3339 timestamps on wire messages.
- [ ] Implement `HELLO`.
- [ ] Implement `MANIFEST`.
- [ ] Implement `HEADS`.
- [ ] Implement `WANT`.
- [ ] Implement `OBJECT`.
- [ ] Implement `BYE`.
- [ ] Implement `ERROR`.
- [ ] Implement `SNAPSHOT_OFFER` only if `snapshot-sync` is advertised.
- [ ] Implement `VIEW_ANNOUNCE` only if `view-sync` is advertised.
- [ ] Recompute `hash(body)` for every `OBJECT`.
- [ ] Reconstruct expected `object_id` from `object_type` and `hash`.
- [ ] Reject any `OBJECT` whose embedded derived ID disagrees with the envelope `object_id`.

## 8. Sync Workflow

- [ ] Support first-time sync: `HELLO` -> `MANIFEST` / `HEADS` -> `WANT` -> `OBJECT`.
- [ ] Support incremental sync from updated `HEADS`.
- [ ] Fetch missing objects only by canonical object ID.
- [ ] Verify objects before indexing or exposing them to readers.
- [ ] Support snapshot-assisted catch-up if snapshots are advertised.
- [ ] Support fetching announced views if `view-sync` is enabled.
- [ ] Treat fetched View objects as governance signals rather than user preference state.

## 9. Views and Head Selection

- [ ] Store verified `view` objects as governance signals, separately from local transport/safety policy state.
- [ ] Group selector inputs by `profile_id`, `doc_id`, and `effective_selection_time`.
- [ ] Resolve `profile_id` as a fixed `policy_hash` for the active reader profile.
- [ ] Compute eligible heads exactly as specified.
- [ ] Use only verified View objects with matching `policy_hash` as maintainer signals.
- [ ] Implement selector epoch calculation exactly.
- [ ] Implement the normative `selector_score`.
- [ ] Implement the normative tie-break order.
- [ ] Emit or persist the minimum decision trace schema.
- [ ] Do not expose discretionary local policy controls that alter the active accepted head.
- [ ] If multiple fixed profiles are supported, enumerate them explicitly rather than allowing ad hoc local profiles.

## 10. Merge Generation

- [ ] Keep revision verification replay-based; do not require receivers to rerun merge generation.
- [ ] Implement the conservative merge generation profile for local authoring tools.
- [ ] Distinguish `Auto-merged`, `Multi-variant`, and `Manual-curation-required`.
- [ ] Materialize merge results as ordinary patch operations.
- [ ] Reject hidden merge metadata as a substitute for explicit state changes.

## 11. CLI or API Surface

- [ ] Provide a local init command or API.
- [ ] Provide object verification tooling.
- [ ] Provide document creation and patch authoring entry points.
- [ ] Provide revision commit entry points.
- [ ] Provide sync pull entry points.
- [ ] Provide view inspection or head-inspection entry points.
- [ ] Separate reader-facing accepted-head inspection from curator-facing View publication workflows.
- [ ] Provide store-rebuild or reindex entry points for recovery.

## 12. Interop Test Minimum

- [ ] Load all normative example objects and ensure they parse.
- [ ] Recompute derived IDs for example `patch`, `revision`, `view`, and `snapshot` objects.
- [ ] Recompute `state_hash` for at least one single-parent revision and one merge revision.
- [ ] Verify example wire envelopes and `OBJECT` validation behavior.
- [ ] Add negative tests for hash mismatch, signature mismatch, and invalid parent ordering.
- [ ] Add a round-trip test for canonical serialization.
- [ ] Add a replay test that rebuilds document state from stored objects only.

## 13. Ready-to-Build Gate

Treat the client as ready for a first interoperable build when all of the following are true:

- [ ] all required object types parse and validate
- [ ] canonical IDs and signatures are reproducible
- [ ] revision replay and `state_hash` verification pass
- [ ] minimal wire sync succeeds end-to-end
- [ ] deterministic head selection produces stable output
- [ ] merge generation can emit valid replayable patch operations
- [ ] the local store can be rebuilt from canonical objects alone
