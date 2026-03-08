# Mycel Spec Issue List

Status: draft

This document tracks protocol-spec issues that are likely to cause interoperability problems, ambiguous validation behavior, or incompatible implementations.

## Priority Guide

- P0: blocks interoperable implementations
- P1: high risk of divergence or unsafe behavior
- P2: important gap, but can be deferred after core compatibility is fixed

## Issue 1: Object identity is underspecified

- Priority: P0
- Affected docs:
  - `PROTOCOL.en.md:51`
  - `PROTOCOL.en.md:77`
  - `PROTOCOL.en.md:129`
  - `PROTOCOL.en.md:209`
  - `PROTOCOL.en.md:247`
  - `PROTOCOL.en.md:273`
  - `WIRE-PROTOCOL.en.md:114`

Problem:

- The spec says all objects are identified by content hash.
- The object model also defines `doc_id`, `block_id`, `patch_id`, `revision_id`, `view_id`, and `snapshot_id` as explicit IDs.
- The wire `OBJECT` message carries both `object_id` and `hash`, but the receiver is only required to verify `hash(body)`.
- The spec does not say whether `object_id` must equal a canonical content hash, a typed hash, or a logical identifier.

Risk:

- Two nodes can accept the same object body under different IDs.
- Object stores, references, and deduplication can diverge permanently.
- Independent implementations may not agree on what an object ID means.

Recommended decision:

- Define three separate concepts explicitly:
  - logical ID: stable document or block identity where needed
  - object hash: content-addressed digest of canonical bytes
  - wire object ID: either equal to object hash, or removed entirely
- State which fields are content-addressed and which are logical references.
- Add a receiver rule that rejects any mismatch between declared ID and recomputed canonical hash.

## Issue 2: Core protocol and wire protocol define incompatible message shapes

- Priority: P0
- Affected docs:
  - `PROTOCOL.en.md:397`
  - `WIRE-PROTOCOL.en.md:23`
  - `WIRE-PROTOCOL.en.md:91`
  - `WIRE-PROTOCOL.en.md:114`

Problem:

- `PROTOCOL.en.md` shows lowercase `want` and `object` messages without the wire envelope.
- `WIRE-PROTOCOL.en.md` requires uppercase `WANT` and `OBJECT` and a full envelope with `version`, `msg_id`, `timestamp`, `from`, `payload`, and `sig`.

Risk:

- Implementers following different sections will build incompatible peers.
- It is unclear which document is normative for network behavior.

Recommended decision:

- Make `WIRE-PROTOCOL.en.md` the sole normative source for transport message format.
- Replace the message examples in `PROTOCOL.en.md` with a short reference to the wire spec.
- Keep only conceptual sync flow in the core protocol document.

## Issue 3: `state_hash` is not reproducible from the current rules

- Priority: P0
- Affected docs:
  - `PROTOCOL.en.md:211`
  - `PROTOCOL.en.md:297`
  - `PROTOCOL.en.md:429`
  - `PROTOCOL.en.md:686`

Problem:

- A revision is defined as a verifiable state formed by parents plus patches.
- The spec does not define:
  - patch application order when multiple patches exist
  - whether parent order is semantically significant
  - what happens when patch application conflicts
  - the canonical form of the resulting state tree used by `state_hash`

Risk:

- Honest nodes can compute different `state_hash` values for the same revision.
- Merge verification cannot be implemented consistently.

Recommended decision:

- Add a normative state-construction algorithm.
- Define ordered inputs, conflict behavior, and canonical state serialization.
- If merge semantics are not ready, constrain v0.1 to single-parent non-merge revisions, or mark merge revisions as provisional.

## Issue 4: Deterministic head selection is declared but not specified enough

- Priority: P1
- Affected docs:
  - `PROTOCOL.en.md:483`
  - `PROTOCOL.en.md:496`

Problem:

- The spec requires deterministic `selected_head`.
- The spec also requires weighted maintainer signals.
- But it does not define `selector_score`, eligible-head calculation, epoch boundaries, weight update math, or the machine-readable decision trace schema.

Risk:

- Different nodes can produce different "deterministic" results from the same data.
- Governance logic becomes implementation-defined instead of protocol-defined.

Recommended decision:

- Either fully specify the selector inputs and scoring formula in v0.1, or demote the current text to non-normative guidance.
- Define a minimal decision-trace schema if the field remains mandatory.

## Issue 5: Signature requirements are incomplete across object types

- Priority: P1
- Affected docs:
  - `PROTOCOL.en.md:59`
  - `PROTOCOL.en.md:273`
  - `PROTOCOL.en.md:360`
  - `WIRE-PROTOCOL.en.md:146`
  - `WIRE-PROTOCOL.en.md:191`

Problem:

- Patch, Revision, and View are explicitly required to be signed.
- Snapshot examples include signatures, but the normative rule is not stated.
- Manifest examples do not include signatures.
- The wire spec says to verify object-level signatures "by object type rules", but those rules are not fully enumerated.

Risk:

- Nodes can disagree on whether unsigned objects or metadata are acceptable.
- Acceptance policy can drift across implementations.

Recommended decision:

- Add an explicit signature matrix for every object and message type:
  - required
  - optional
  - forbidden
- Define the exact signed payload for each signed type.

## Issue 6: Minimal sync flow depends on message types with no normative schema

- Priority: P1
- Affected docs:
  - `PROTOCOL.en.md:360`
  - `PROTOCOL.en.md:378`
  - `WIRE-PROTOCOL.en.md:49`
  - `WIRE-PROTOCOL.en.md:181`

Problem:

- The sync flow uses `MANIFEST`, `HEADS`, `SNAPSHOT_OFFER`, `VIEW_ANNOUNCE`, and `BYE`.
- Only `HELLO`, `WANT`, `OBJECT`, and `ERROR` have enough structure to guide implementation.

Risk:

- v0.1 cannot support fully independent peer implementations yet.
- Every implementation will invent local extensions for required sync steps.

Recommended decision:

- Either add normative schemas for all message types used in the minimal sync flow, or reduce the v0.1 claim to the subset that is actually specified.

## Suggested Resolution Order

1. Lock object identity, hash, and canonical serialization rules.
2. Unify transport definitions so only one document is normative for wire format.
3. Define reproducible `state_hash` construction.
4. Add a complete signature matrix.
5. Either fully specify head selection or move it to a later version.
6. Finish schemas for the remaining sync messages.
