# Mycel Protocol v0.1

Language: English | [Traditional Chinese](./PROTOCOL.zh-TW.md)

## 0. Positioning

Mycel is a text protocol with the following characteristics:

- Git-like version model
- P2P replication
- Signature verification
- Multiple branches can coexist
- No requirement for global single consensus

It is neither a blockchain nor a Git clone. It is a protocol for text and knowledge artifacts that supports decentralization, forking, and verifiable history.

Applicable scenarios include:

- Long-lived texts
- Commentary
- Manifestos
- Community charters
- Specification documents
- Decentralized wiki
- Hard-to-delete knowledge networks

## 1. Design Goals

Mycel is designed with the following goals:

1. **Verifiable history**: All accepted changes must be traceable and replay-verifiable.
2. **Decentralized survivability**: Content remains preservable and synchronizable without a single server.
3. **Forks are valid**: Forking is a first-class valid state.
4. **Optional merge**: Communities can form their own accepted view by local policy.
5. **Anonymous usability**: Authors can use pseudonymous keys, and metadata exposure should be minimized.
6. **Text-first (v0.1)**: In v0.1, block / paragraph is the primary unit of operation.

## 2. Protocol Concepts

Mycel splits data into six core concepts:

- **Document**: A document
- **Block**: A paragraph/block
- **Patch**: One modification
- **Revision**: A verifiable state
- **View**: A version set trusted by a community
- **Snapshot**: A packaged state at a point in time

## 3. Core Principles

### 3.1 Logical IDs vs. Canonical Object IDs

Mycel uses two different identifier classes:

- **Logical IDs**: stable references inside document state, such as `doc_id` and `block_id`
- **Canonical object IDs**: content-addressed IDs for replicated objects, such as `patch_id`, `revision_id`, `view_id`, and `snapshot_id`

Logical IDs are part of application state and MUST NOT be interpreted as content hashes.
Canonical object IDs are derived from canonical bytes:

```text
object_hash = HASH(canonical_serialization(object_without_derived_ids_or_signatures))
object_id = <type-prefix>:<object_hash>
```

For v0.1:

- `doc_id` and `block_id` are logical IDs
- `patch_id`, `revision_id`, `view_id`, and `snapshot_id` are canonical object IDs
- the derived ID field itself and the `signature` field MUST NOT be included in the hash input

This split avoids recursive self-hashing and keeps transport references unambiguous.

### 3.2 Signature is Mandatory

All author-generated Patch, Revision, and View objects must include a digital signature.
Signature requirements for all v0.1 object types are defined normatively in Section 6.4.

### 3.3 Multiple Heads are Valid

A single document may have multiple heads.

### 3.4 Accepted View is Not Global Truth

A so-called "accepted version" is only a View chosen by some group, not the only network-wide truth.

### 3.5 Transport and Acceptance are Separate

A node can receive an object without accepting it into its local accepted view.

## 4. Object Model

### 4.1 Document

A Document defines the identity and baseline settings of a text.

```json
{
  "type": "document",
  "version": "mycel/0.1",
  "doc_id": "doc:origin-text",
  "title": "Origin Text",
  "language": "zh-Hant",
  "content_model": "block-tree",
  "created_at": 1777777777,
  "created_by": "pk:authorA",
  "genesis_revision": "rev:0ab1"
}
```

Fields:

- `doc_id`: stable logical document ID, not a content hash
- `title`: title
- `language`: language
- `content_model`: content model, fixed as `block-tree` in v0.1
- `genesis_revision`: initial revision

### 4.2 Block

A Block is the smallest structural text unit.

```json
{
  "type": "block",
  "block_id": "blk:001",
  "block_type": "paragraph",
  "content": "At first there was no final draft, only transmission.",
  "attrs": {},
  "children": []
}
```

Allowed `block_type` values:

- `title`
- `heading`
- `paragraph`
- `quote`
- `verse`
- `list`
- `annotation`
- `metadata`

`block_id` is a logical block reference within document state, not a content hash.

### 4.3 Patch

A Patch represents one modification to a document.

```json
{
  "type": "patch",
  "version": "mycel/0.1",
  "patch_id": "patch:91ac",
  "doc_id": "doc:origin-text",
  "base_revision": "rev:0ab1",
  "author": "pk:authorA",
  "timestamp": 1777778888,
  "ops": [
    {
      "op": "replace_block",
      "block_id": "blk:001",
      "new_content": "At first there was no final draft, only transmission and rewriting."
    },
    {
      "op": "insert_block_after",
      "after_block_id": "blk:001",
      "new_block": {
        "block_id": "blk:002",
        "block_type": "paragraph",
        "content": "Whatever is written can be rewritten.",
        "attrs": {},
        "children": []
      }
    }
  ],
  "signature": "sig:..."
}
```

At minimum, the Patch signature input must include:

- `type`
- `version`
- `doc_id`
- `base_revision`
- `timestamp`
- `author`
- `ops`

`patch_id` is a derived canonical object ID with the form `patch:<object_hash>`.
It MUST be computed from the canonical Patch body with `patch_id` and `signature` omitted.

For genesis-state Patch objects in v0.1, `base_revision` MUST use the fixed sentinel value `rev:genesis-null`.

### 4.4 Patch Operations

v0.1 should define only a small set of primitive operations:

- `insert_block`
- `insert_block_after`
- `delete_block`
- `replace_block`
- `move_block`
- `annotate_block`
- `set_metadata`

Example: delete

```json
{
  "op": "delete_block",
  "block_id": "blk:009"
}
```

Example: annotate

```json
{
  "op": "annotate_block",
  "block_id": "blk:001",
  "annotation": {
    "block_id": "blk:ann01",
    "block_type": "annotation",
    "content": "This paragraph is a common community-maintained variant."
  }
}
```

### 4.4.1 Trivial Change (Normative)

In Mycel v0.1, a `trivial change` is an editorial surface-form change that does not alter document structure, reference targets, metadata meaning, or intended semantic meaning.

A Patch MAY be classified as trivial only if all of the following are true:

1. every operation targets an existing block in the same document state lineage
2. every operation is either:
   - `replace_block` on an existing block, or
   - `annotate_block` that does not alter the target block's own content
3. no operation changes block order, block parentage, block identity, or block type
4. no operation inserts, deletes, or moves a block
5. no operation changes metadata keys or metadata values
6. no operation changes identifiers, revision references, URLs, numeric values, or date/time literals in a way that could change interpretation
7. the resulting text is intended only to correct or normalize surface form

Typical trivial changes include:

- obvious typo correction
- whitespace normalization
- punctuation normalization
- capitalization normalization when meaning is unchanged
- annotation formatting cleanup that does not change the annotated claim

The following are not trivial changes:

- any structural change
- any insertion, deletion, or move
- any change to `block_id`
- any change to metadata semantics
- any wording change that can reasonably alter interpretation

Trivial-change classification is advisory only.
It MUST NOT bypass normal Patch validation, Revision validation, signature checks, merge rules, or `state_hash` recomputation.

### 4.5 Revision

A Revision is a state node.
It is not the full text itself, but a verifiable state formed by "parents + patch set".

```json
{
  "type": "revision",
  "version": "mycel/0.1",
  "revision_id": "rev:8fd2",
  "doc_id": "doc:origin-text",
  "parents": ["rev:0ab1"],
  "patches": ["patch:91ac"],
  "state_hash": "hash:state001",
  "author": "pk:authorA",
  "timestamp": 1777778890,
  "signature": "sig:..."
}
```

Example merge revision:

```json
{
  "type": "revision",
  "version": "mycel/0.1",
  "revision_id": "rev:c7d4",
  "doc_id": "doc:origin-text",
  "parents": ["rev:8fd2", "rev:b351"],
  "patches": ["patch:a12f"],
  "state_hash": "hash:merged-state",
  "author": "pk:curator1",
  "timestamp": 1777780000,
  "merge_strategy": "semantic-block-merge",
  "signature": "sig:..."
}
```

`revision_id` is a derived canonical object ID with the form `rev:<object_hash>`.
It MUST be computed from the canonical Revision body with `revision_id` and `signature` omitted.

### 4.5.1 Revision State Construction (Normative)

To make `state_hash` reproducible in v0.1, Revision state construction is defined as follows:

1. `parents` is an ordered array.
2. A genesis revision MUST use `parents: []`.
3. A non-merge revision MUST use exactly one parent.
4. A multi-parent revision MUST treat `parents[0]` as the execution base state.
5. Additional parents in `parents[1..]` record merged ancestry only; they MUST NOT implicitly contribute content to the resulting state.
6. Any content adopted from secondary parents MUST be materialized explicitly in the listed `patches`.
7. `patches` is an ordered array and MUST be applied sequentially in array order.
8. Every Patch referenced by a Revision MUST have the same `doc_id` as the Revision.
9. Every Patch referenced by a non-genesis Revision MUST use `base_revision = parents[0]`. For a genesis revision, every referenced Patch MUST use `base_revision = rev:genesis-null`.
10. If any referenced Patch is missing, invalid, or cannot be applied deterministically, the Revision is invalid.

This means receivers never re-run a semantic merge algorithm to recompute a Revision state.
They only replay ordered Patch operations against the execution base state.

### 4.6 View

A View means "which versions this community/node currently accepts".

```json
{
  "type": "view",
  "version": "mycel/0.1",
  "view_id": "view:9aa0",
  "maintainer": "pk:community-curator",
  "documents": {
    "doc:origin-text": "rev:c7d4",
    "doc:governance-rules": "rev:91de"
  },
  "policy": {
    "preferred_branches": ["community-mainline"],
    "accept_keys": ["pk:community-curator", "pk:reviewerB"],
    "merge_rule": "manual-reviewed"
  },
  "timestamp": 1777781000,
  "signature": "sig:..."
}
```

View is critical, because Mycel has no single global accepted view.

`view_id` is a derived canonical object ID with the form `view:<object_hash>`.
It MUST be computed from the canonical View body with `view_id` and `signature` omitted.

### 4.7 Snapshot

A Snapshot is used for fast synchronization.

```json
{
  "type": "snapshot",
  "version": "mycel/0.1",
  "snapshot_id": "snap:44cc",
  "documents": {
    "doc:origin-text": "rev:c7d4"
  },
  "included_objects": [
    "rev:c7d4",
    "patch:91ac",
    "patch:a12f"
  ],
  "root_hash": "hash:snapshot-root",
  "created_by": "pk:mirrorA",
  "timestamp": 1777782000,
  "signature": "sig:..."
}
```

`snapshot_id` is a derived canonical object ID with the form `snap:<object_hash>`.
It MUST be computed from the canonical Snapshot body with `snapshot_id` and `signature` omitted.

## 5. Serialization and Hashing

### 5.1 Canonical Serialization

Before hashing or signing, all protocol objects MUST be transformed into the canonical JSON form defined in Appendix A.
The same canonicalization rules also apply to state objects used for `state_hash` computation and to wire envelopes referenced by `WIRE-PROTOCOL.en.md`.

### 5.2 Hash

In v0.1, the network MUST use one fixed hash algorithm for canonical object IDs and object verification.
The default recommendation is:

```text
hash = BLAKE3(canonical_bytes)
```

If a conservative choice is preferred, SHA-256 is also possible. But one network must fix one algorithm and not mix both.

### 5.3 Derived ID Rules

For any content-addressed object type in v0.1:

1. Canonicalize the object body
2. Omit the derived ID field (`patch_id`, `revision_id`, `view_id`, or `snapshot_id`)
3. Omit `signature`
4. Hash the remaining canonical bytes using the network hash algorithm
5. Reconstruct the derived ID as `<type-prefix>:<object_hash>`

A receiver MUST reject any content-addressed object whose embedded derived ID does not match the recomputed canonical object ID.

### 5.4 State Hash Computation (Normative)

For a Revision in v0.1, `state_hash` is computed as follows:

1. Resolve the execution base state:
   - if `parents` is empty, use the empty state `{ "doc_id": <revision.doc_id>, "blocks": [] }`
   - otherwise, load the fully verified state of `parents[0]`
2. Replay the referenced `patches` in array order against that execution base state.
3. Produce the resulting document state as a canonical state object:

```json
{
  "doc_id": "doc:origin-text",
  "blocks": [
    {
      "block_id": "blk:001",
      "block_type": "paragraph",
      "content": "...",
      "attrs": {},
      "children": []
    }
  ]
}
```

4. Canonicalize that state object using the same serialization rules used elsewhere in the protocol.
5. Compute `state_hash = HASH(canonical_state_bytes)`.

Additional rules:

- Top-level block order MUST be preserved in the resulting `blocks` array.
- Child block order MUST be preserved in each `children` array.
- Deleted blocks MUST be absent from the resulting state.
- Multi-variant outcomes, if preserved, MUST be represented explicitly by the applied Patch result state rather than inferred from parent ancestry alone.
- A receiver MUST reject any Revision whose declared `state_hash` does not match the recomputed value.

## 6. Identity and Signature

### 6.1 Author Identity

In Mycel, author identity is pseudonymous public-key identity by default.

```text
author_id = pk:<public_key_fingerprint>
```

Not an account, not a real name.

### 6.2 Signature Algorithms

v0.1 recommendation:

- Signatures: Ed25519
- Key exchange: X25519

### 6.3 Identity Modes

Mycel supports three modes:

- **Persistent pseudonym**: long-term pen-name key
- **Rotating pseudonym**: periodically rotated key
- **One-time signer**: one-time author key

### 6.4 Object Signature Matrix (Normative)

Object signature requirements in v0.1:

| Object type | Signature status | Signer field | Signed payload |
| --- | --- | --- | --- |
| `document` | forbidden | none | none |
| `block` | forbidden | none | none |
| `patch` | required | `author` | canonical Patch with `signature` omitted |
| `revision` | required | `author` | canonical Revision with `signature` omitted |
| `view` | required | `maintainer` | canonical View with `signature` omitted |
| `snapshot` | required | `created_by` | canonical Snapshot with `signature` omitted |

Rules:

1. A receiver MUST reject any v0.1 `patch`, `revision`, `view`, or `snapshot` object that is missing `signature`.
2. A receiver MUST reject any `document` or `block` object that carries a top-level `signature` field.
3. The signer key referenced by the signer field MUST verify the signature over the corresponding canonical payload.
4. For content-addressed object types, the embedded derived ID MUST already match the recomputed canonical object ID before signature verification succeeds.
5. The `signature` field itself MUST NOT be included in the signed payload.

### 6.5 Object Signature Inputs (Normative)

The signed payload for each signed v0.1 object type is the canonical serialization of the object with only `signature` omitted.

This means:

- `patch` signatures cover `patch_id`, `doc_id`, `base_revision`, `author`, `timestamp`, and `ops`
- `revision` signatures cover `revision_id`, `doc_id`, `parents`, `patches`, `state_hash`, `author`, `timestamp`, and any declared merge fields
- `view` signatures cover `view_id`, `maintainer`, `documents`, `policy`, and `timestamp`
- `snapshot` signatures cover `snapshot_id`, `documents`, `included_objects`, `root_hash`, `created_by`, and `timestamp`

## 7. Node Model

Mycel nodes have five role types (one node can take multiple roles):

1. **Author Node**: creates patch/revision
2. **Mirror Node**: stores and serves content
3. **Curator Node**: maintains views and accepted branches
4. **Relay Node**: forwards metadata and objects
5. **Archivist Node**: preserves full history

## 8. P2P Sync Layer

Mycel does not require all nodes to sync all data; partial replication is supported.

### 8.1 Node Declaration: Manifest

Each node may publish a manifest:

```json
{
  "type": "manifest",
  "version": "mycel/0.1",
  "node_id": "node:alpha",
  "topics": ["text/core", "text/commentary"],
  "heads": {
    "doc:origin-text": ["rev:c7d4", "rev:b351"]
  },
  "snapshots": ["snap:44cc"],
  "capabilities": ["patch-sync", "snapshot-sync", "view-sync"]
}
```

### 8.2 Sync Flow

First-time join:

1. Node obtains bootstrap peers
2. Fetches manifests
3. Pulls latest snapshot
4. Fills missing patch/revision gap
5. Builds local view

Routine updates:

1. Receive head announcement
2. Check which objects are missing locally
3. Fetch missing objects by canonical object ID
4. Verify hash and signature
5. Store in local store
6. Decide whether to include into view based on local policy

### 8.3 Message Types

v0.1 minimal message set:

- `HELLO`
- `MANIFEST`
- `HEADS`
- `WANT`
- `OBJECT`
- `SNAPSHOT_OFFER`
- `VIEW_ANNOUNCE`
- `BYE`

The transport shape of these messages is defined normatively in `WIRE-PROTOCOL.en.md`.
This core protocol document only defines the conceptual sync flow and the meaning of the replicated objects.

## 9. Conflict and Merge

Mycel does not treat conflicts as protocol failure.

### 9.1 Valid States

All of the following are valid:

- multiple heads
- long-term coexistence of different branches
- multiple local variants for the same paragraph

### 9.2 Three Merge Result Classes

- **Auto-merged**: automatic merge succeeded
- **Multi-variant**: keep parallel variants
- **Manual-curation-required**: needs human curation

In v0.1, any merge outcome that is published as a Revision MUST already be materialized as explicit Patch operations.
Receivers verify the resulting state by replaying those Patches; they do not recompute a semantic merge from parent ancestry.

### 9.3 Merge Generation Profile v0.1 (Normative)

Mycel v0.1 defines one conservative semantic merge generation profile.
This profile is used only to generate candidate merge Patch operations.
Verification still depends only on the resulting Patch, Revision, and `state_hash`.

#### 9.3.1 Inputs

A merge generator takes:

- `base_revision`
- `left_revision`
- `right_revision`

All three inputs MUST:

1. belong to the same `doc_id`
2. be fully verified revisions
3. be reduced to canonical document states before merge generation starts

`base_revision` is the common ancestor state used for comparison.
`left_revision` and `right_revision` are the two candidate descendant states being reconciled.

#### 9.3.2 Per-Block Classification

For each logical `block_id` reachable from any of the three states, classify the block as one of:

- unchanged
- inserted
- deleted
- replaced
- moved
- annotated
- metadata-changed

Classification is always relative to `base_revision`.

#### 9.3.3 Auto-Merge Rules

A merge generator MAY produce `Auto-merged` only if every affected block is resolved by the following rules:

1. If only one side changes a block and the other side leaves it unchanged, take the changed side.
2. If both sides make byte-identical changes to the same block, take that shared result.
3. If both sides insert different new blocks at different positions, keep both inserts in deterministic order:
   1. lower parent position index
   2. left-side insert before right-side insert when the parent position is the same
   3. lexicographically smaller new `block_id`
4. If one side annotates a block and the other side changes the block content without deleting it, keep both the content change and the annotation.
5. If both sides change metadata on different metadata keys, merge the key updates.

If any affected block is not covered by these rules, the generator MUST NOT emit `Auto-merged`.

#### 9.3.4 Forced Non-Automatic Cases

The merge generator MUST emit `Multi-variant` or `Manual-curation-required` for any of the following:

1. both sides replace the same block with different content
2. one side deletes a block that the other side replaces, moves, or annotates
3. both sides move the same block to different destinations
4. both sides set different values for the same metadata key
5. either side changes block structure and the other side changes the same subtree incompatibly

#### 9.3.5 Multi-Variant Output Rule

If the conflict is limited to alternative surviving content for the same logical block, the generator SHOULD prefer `Multi-variant`.
The resulting merge Patch MUST explicitly materialize the preserved alternatives in the merged state.

#### 9.3.6 Manual Curation Rule

If the conflict affects structure, ordering, deletion semantics, or metadata in a way that cannot be expressed safely as parallel surviving variants, the generator MUST emit `Manual-curation-required`.

#### 9.3.7 Output Form

The generated result MUST be materialized as ordinary Patch operations.
The generator MUST NOT rely on hidden merge metadata to make the resulting state valid.

If the generator emits `Auto-merged`, its Patch operations MUST be sufficient for any receiver to replay the result deterministically from `parents[0]`.

### 9.4 Multi-Variant Block Example

```json
{
  "type": "variant_block",
  "block_id": "blk:001",
  "variants": [
    {
      "from_revision": "rev:8fd2",
      "content": "At first there was no final draft, only transmission."
    },
    {
      "from_revision": "rev:b351",
      "content": "At first there was no final draft, only transmission and rewriting."
    }
  ]
}
```

## 10. View and Acceptance

Mycel does not define one global accepted view. Only these exist:

- local view
- community view
- public view
- archival view

Examples:

- One community maintains its own accepted view
- One scholar maintains a critical-edition view
- One node accepts only patches from trusted authors

This is a core difference between Mycel and blockchain systems.

### 10.1 Deterministic Head Selection (Normative)

To reduce client-side divergence, head selection is protocol-driven:

1. A client MUST request by `view_id` and `doc_id`, and MAY include a selection-time boundary.
2. A client MUST NOT force `head_id`.
3. A node MUST compute `selected_head` in real time from eligible heads under the requested view policy.
4. The selector MUST be deterministic for the same verified object set, local selector policy state, and effective selection time.
5. The response MUST include `selected_head` and a machine-readable decision trace.

#### 10.1.1 Selector Inputs

The selector input tuple is:

- `view_id`
- `doc_id`
- `effective_selection_time`

`effective_selection_time` is defined as:

- the client-supplied boundary, if one is provided
- otherwise the node's request-handling time

If the client omits the boundary, the node MUST emit the resolved `effective_selection_time` in the decision trace.

The node MUST resolve `view_id` to a fully verified View object `V`.
The selector policy hash is:

```text
policy_hash = HASH(canonical_serialization(V.policy))
```

#### 10.1.2 Eligible Heads

For a given `doc_id`, a Revision is an eligible head if all of the following are true:

1. the Revision is fully verified under all object, hash, signature, and state rules
2. the Revision `doc_id` matches the requested `doc_id`
3. the Revision timestamp is less than or equal to `effective_selection_time`
4. the Revision is accepted for consideration by the local policy state associated with `policy_hash`
5. there is no other accepted Revision for the same `doc_id`, with timestamp less than or equal to `effective_selection_time`, that is a descendant of it

If no eligible heads exist, selection MUST fail with a machine-readable reason such as `NO_ELIGIBLE_HEAD`.

#### 10.1.3 Maintainer Signals

For each admitted maintainer key `k`, the selector derives at most one signal in the selector epoch:

1. determine the selector epoch using the rules in Section 10.2
2. collect all fully verified View objects such that:
   - `maintainer == k`
   - `timestamp` is within the selector epoch
   - `timestamp <= effective_selection_time`
   - `HASH(canonical_serialization(view.policy)) == policy_hash`
3. choose the latest such View by:
   1. newer `timestamp`
   2. lexicographically smaller `view_id`
4. if that View has a `documents[doc_id]` entry and its value is one of the eligible heads, then `k` contributes one support signal to that head
5. otherwise `k` contributes no signal for that `doc_id`

Each admitted maintainer contributes to at most one eligible head for a given `(policy_hash, doc_id, selector_epoch)`.

#### 10.1.4 Selector Score

For each eligible head `h`:

```text
weighted_support(h) = sum(effective_weight(k)) for all maintainers k signaling to h
supporter_count(h) = count(k) for all maintainers k signaling to h
selector_score(h) = weighted_support(h)
```

The selected head is the eligible head with the greatest ordered tuple:

```text
(selector_score, revision_timestamp, inverse_lexicographic_priority)
```

Tie-break order MUST be:

1. higher `selector_score`
2. newer `revision_timestamp`
3. lexicographically smaller `revision_id`

Raw supporter count MAY appear in the trace for auditability, but MUST NOT outrank `selector_score`.

#### 10.1.5 Decision Trace Schema

The decision trace MUST be machine-readable and MUST include at least:

```json
{
  "view_id": "view:...",
  "doc_id": "doc:origin-text",
  "effective_selection_time": 1777781000,
  "policy_hash": "hash:...",
  "selector_epoch": 587,
  "eligible_heads": [
    {
      "revision_id": "rev:0ab1",
      "revision_timestamp": 1777780000,
      "weighted_support": 7,
      "supporter_count": 3,
      "selector_score": 7
    }
  ],
  "selected_head": "rev:0ab1",
  "tie_break_reason": "higher_selector_score"
}
```

The trace MUST be reproducible from the same verified object set, selector policy state, and effective selection time.

### 10.2 Maintainer Set + Weights Admission (Normative)

Mycel uses pseudonymous, identity-blind maintainer governance.
Maintainers are identified by keys; real-world identity and mutual acquaintance are not required.

Admission and weighting rules:

1. A maintainer candidate MUST be evaluated only by verifiable protocol behavior, not claimed real identity.
2. A node MUST store and publish its local selector policy parameters for auditability.
3. At minimum, selector policy parameters MUST include:
   - `epoch_seconds`
   - `epoch_zero_timestamp`
   - `admission_window_epochs`
   - `min_valid_views_for_admission`
   - `min_valid_views_per_epoch`
   - `weight_cap_per_key`
4. `epoch_seconds` MUST be a positive integer.
5. The selector epoch is:

```text
selector_epoch = floor((effective_selection_time - epoch_zero_timestamp) / epoch_seconds)
```

6. For each maintainer key `k` and epoch `e`, define:
   - `valid_view_count(e, k)`: the number of fully verified View objects by `k` in epoch `e` whose policy hash matches the selector `policy_hash`
   - `critical_violation_count(e, k)`: the number of verifiable critical violations attributed to `k` in epoch `e`
7. A maintainer key is admitted in epoch `e` if, across the previous `admission_window_epochs` completed epochs:
   - the sum of `valid_view_count` is at least `min_valid_views_for_admission`
   - the sum of `critical_violation_count` is zero
8. A non-admitted key MUST have effective weight `0`.
9. An admitted key first receives weight `1`.
10. For each later epoch, the effective weight update rule is:

```text
delta(e, k) =
  -1 if critical_violation_count(e-1, k) > 0
  +1 if critical_violation_count(e-1, k) == 0
       and valid_view_count(e-1, k) >= min_valid_views_per_epoch
   0 otherwise

effective_weight(e, k) =
  clamp(effective_weight(e-1, k) + delta(e, k), 0, weight_cap_per_key)
```

11. `clamp(x, lo, hi)` returns `lo` if `x < lo`, `hi` if `x > hi`, else `x`.
12. A key with one or more critical violations in epoch `e-1` MUST lose at least one weight unit in epoch `e`.
13. A node MAY quarantine or remove a key entirely by policy; quarantined or removed keys MUST have effective weight `0`.
14. Head selection MUST use `effective_weight(e, k)` and MUST NOT rely on raw hit count alone.

## 11. Anonymity and Security Defaults

### 11.1 Transport Anonymity

Mycel recommends anonymous transport by default, such as:

- Tor onion services
- other anonymous mesh transport

### 11.2 Content Security

Each object should pass:

- hash verification
- signature verification
- context verification

### 11.3 Metadata Minimization

Recommended node behavior:

- batch forwarding
- random delay
- avoid exposing real author identity
- topic names can be capability-based

### 11.4 Trust Policy

Each node may define:

- which author keys are accepted
- which curator keys are accepted
- whether anonymous keys are accepted
- whether new keys must be quarantined first

## 12. Local Storage Model

Local storage is split into:

### 12.1 Object Store

Store all objects by `object_id`.

### 12.2 Index Store

Maintain indexes:

- `doc_id -> revisions`
- `revision -> parents`
- `block_id -> latest states`
- `author -> patches`
- `view_id -> current head map`

### 12.3 Policy Store

Persist local trust and acceptance rules.

## 13. URI / Naming Format

v0.1 can use this naming style:

- `mycel://doc/origin-text`
- `mycel://rev/c7d4`
- `mycel://patch/91ac`
- `mycel://view/9aa0`
- `mycel://snap/44cc`

## 14. CLI Prototype

Future tools may include:

```bash
mycel init
mycel create-doc origin-text
mycel patch origin-text
mycel commit origin-text
mycel branch create community-mainline
mycel merge rev:8fd2 rev:b351
mycel view create community-curation-v3
mycel sync
mycel serve
mycel verify
```

## 15. Minimal Implementation Architecture

A minimal Mycel client should include:

### 15.1 Core

- object serializer
- hash engine
- signature engine
- patch applier
- revision builder

### 15.2 Store

- object store
- index store
- local policy store

### 15.3 Network

- peer transport
- manifest exchange
- want/object exchange
- snapshot sync

### 15.4 UI

- CLI
- wiki-like reader/editor
- diff viewer
- branch/view browser

## 16. Typical Workflow Examples

### 16.1 Create a Document

1. Author A creates `origin-text`
2. Creates genesis blocks
3. Creates genesis revision
4. Signs it
5. Publishes to peers

### 16.2 Modify a Document

Author B wants to edit one paragraph:

1. Download latest revision
2. Build patch
3. Sign with their own key
4. Produce new revision
5. Publish to the network

### 16.3 Branch

Author C disagrees with mainline:

1. Build patch from the same `base_revision`
2. Publish a different revision
3. Network forms a second head

### 16.4 Merge

Curator D wants to reconcile both lines:

1. Fetch both heads
2. Try semantic block merge
3. If successful, produce merge revision
4. Publish a new view signed by curator key

## 17. Protocol Spirit

The core of Mycel is not one final truth, but:

> Text can change; history can be verified; branches can diverge; the network can disperse.

English shorthand:

> Write locally. Sign changes. Replicate freely. Merge socially.

## 18. Feature Summary

In one sentence, Mycel differs from other systems as follows:

- Not Git: native P2P, native multi-view, native anonymous usability
- Not blockchain: no global single consensus goal
- Not torrent: it does not only transfer file bundles, but verifiable change history
- Not a normal wiki: versioning is not a side feature; it is the core structure

## 19. Recommended Next Version

This version now includes:

1. **Wire protocol**: normative sync-message schemas
2. **Canonical serialization appendix**: deterministic hashing and signing rules
3. **Conservative merge generation profile**: replay-safe merge output rules

The most valuable next additions are:

1. **Implementation checklist**: turn the spec into a buildable implementation profile
2. **Consistency audit**: align examples, terminology, and scope across all documents
3. **Governance simplification review**: reduce optional selector/governance complexity before calling v0.1 stable

## Appendix A. Canonical Serialization (Normative)

Mycel v0.1 uses canonical JSON bytes for:

- content-addressed object IDs
- object signatures
- `state_hash` computation
- wire-envelope signatures

### A.1 Encoding

1. Canonical bytes MUST be UTF-8 encoded JSON text.
2. JSON text MUST NOT include a byte order mark.
3. Insignificant whitespace is forbidden outside string values.

### A.2 Data Types

Allowed JSON value types in v0.1 canonical payloads:

- object
- array
- string
- integer number
- `true`
- `false`

The following are invalid in canonical payloads:

- `null`
- floating-point numbers
- exponent notation
- duplicate object keys

### A.3 Object Rules

1. Object keys MUST be unique.
2. Object keys MUST be serialized in ascending lexicographic order by raw Unicode code point.
3. Object members MUST be serialized as `"key":value` with no extra spaces.

Example key order:

```json
{"author":"pk:a","doc_id":"doc:x","type":"patch","version":"mycel/0.1"}
```

### A.4 Array Rules

1. Arrays MUST preserve protocol-defined order.
2. Arrays MUST be serialized with comma-separated values and no extra spaces.
3. Arrays MUST NOT be re-sorted during canonicalization.

This means:

- `parents` stays in declared order
- `patches` stays in declared order
- `blocks` stays in structural document order
- wire `objects` in `WANT` stays in sender request order

### A.5 String Rules

1. Strings MUST be serialized using JSON double-quoted string syntax.
2. Strings MUST preserve code points exactly as authored; implementations MUST NOT apply Unicode normalization.
3. The quotation mark (`"`) and reverse solidus (`\`) MUST be escaped.
4. Control characters U+0000 through U+001F MUST be escaped using lowercase `\u00xx`.
5. `/` MUST NOT be escaped unless required by a higher-layer transport outside canonicalization.
6. Non-ASCII characters MAY appear directly in UTF-8 and MUST NOT be rewritten into `\u` escapes unless they are control characters.

### A.6 Integer Rules

1. Numbers in v0.1 canonical payloads MUST be base-10 integers.
2. Zero MUST be serialized as `0`.
3. Positive integers MUST NOT include a leading `+`.
4. Integers MUST NOT contain leading zeros.
5. Negative integers are allowed only if a field definition explicitly permits them.

### A.7 Booleans

Boolean values MUST be serialized as lowercase `true` or `false`.

### A.8 Field Omission

1. Optional fields that are not present MUST be omitted entirely.
2. Implementations MUST NOT encode "missing" as `null`.
3. Derived ID fields and `signature` are omitted only when a specific hashing or signing rule explicitly says so.

### A.9 Canonicalization Procedure

To canonicalize a payload:

1. Validate that the payload uses only allowed JSON types.
2. Reject duplicate keys.
3. Reject forbidden numeric forms and `null`.
4. Recursively sort object keys using the rule in A.3.
5. Preserve all array orders.
6. Serialize using UTF-8 JSON with no insignificant whitespace.

### A.10 Canonical State Object

When computing `state_hash`, the resulting state object MUST use this shape:

```json
{
  "doc_id": "doc:origin-text",
  "blocks": [
    {
      "block_id": "blk:001",
      "block_type": "paragraph",
      "content": "Example text",
      "attrs": {},
      "children": []
    }
  ]
}
```

Additional rules:

1. Every block object in state serialization MUST include `block_id`, `block_type`, `content`, `attrs`, and `children`.
2. `attrs` MUST be an object; when empty it MUST serialize as `{}`.
3. `children` MUST be an array; when empty it MUST serialize as `[]`.

### A.11 Canonical Envelope Serialization

Wire envelopes use the same canonical JSON rules.
When computing the envelope signature, the `sig` field MUST be omitted before canonicalization.
