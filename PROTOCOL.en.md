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
  "genesis_revision": "rev:abc123"
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
  "base_revision": "rev:old001",
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

### 4.5 Revision

A Revision is a state node.
It is not the full text itself, but a verifiable state formed by "parents + patch set".

```json
{
  "type": "revision",
  "version": "mycel/0.1",
  "revision_id": "rev:new001",
  "doc_id": "doc:origin-text",
  "parents": ["rev:old001"],
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
  "revision_id": "rev:merge001",
  "doc_id": "doc:origin-text",
  "parents": ["rev:branchA", "rev:branchB"],
  "patches": ["patch:mergeA"],
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
  "view_id": "view:community-curation-v3",
  "maintainer": "pk:community-curator",
  "documents": {
    "doc:origin-text": "rev:merge001",
    "doc:governance-rules": "rev:law220"
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
  "snapshot_id": "snap:weekly-2026-03-08",
  "documents": {
    "doc:origin-text": "rev:merge001"
  },
  "included_objects": [
    "rev:merge001",
    "patch:91ac",
    "patch:mergeA"
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

Before hashing, all objects must be transformed into a fixed canonical form:

- fixed key order
- UTF-8
- no unnecessary whitespace
- fixed array order
- fixed number format

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
    "doc:origin-text": ["rev:merge001", "rev:branchB"]
  },
  "snapshots": ["snap:weekly-2026-03-08"],
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

### 9.3 Multi-Variant Block Example

```json
{
  "type": "variant_block",
  "block_id": "blk:001",
  "variants": [
    {
      "from_revision": "rev:branchA",
      "content": "At first there was no final draft, only transmission."
    },
    {
      "from_revision": "rev:branchB",
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

1. A client MUST request by `view_id` (optionally with a time boundary), and MUST NOT force `head_id`.
2. A node MUST compute `selected_head` in real time from eligible heads under the requested view policy.
3. The selector MUST be deterministic for the same input state and policy.
4. The response MUST include `selected_head` and a machine-readable decision trace (score components and tie-break reason).
5. Tie-break order MUST be fixed as:
   1. higher `selector_score`
   2. newer `revision_timestamp`
   3. lexicographically smaller `revision_id`

### 10.2 Maintainer Set + Weights Admission (Normative)

Mycel uses pseudonymous, identity-blind maintainer governance.
Maintainers are identified by keys; real-world identity and mutual acquaintance are not required.

Admission and weighting rules:

1. A maintainer candidate MUST be evaluated only by verifiable protocol behavior, not claimed real identity.
2. Minimum admission criteria MUST include:
   1. valid signing history over a required observation window
   2. no unresolved critical verification violations in that window
   3. sustained protocol activity above a local minimum threshold
3. A node MUST store and publish its local admission policy for auditability.
4. Each maintainer key MUST have a bounded maximum influence (`weight_cap_per_key`).
5. Weight updates MUST be step-limited per epoch to prevent abrupt governance capture.
6. A key with repeated invalid or malicious actions MUST be downgraded, quarantined, or removed by policy.
7. Head selection MUST use weighted maintainer signals, and MUST NOT rely on raw hit count alone.

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
- `mycel://rev/merge001`
- `mycel://patch/91ac`
- `mycel://view/community-curation-v3`
- `mycel://snap/weekly-2026-03-08`

## 14. CLI Prototype

Future tools may include:

```bash
mycel init
mycel create-doc origin-text
mycel patch origin-text
mycel commit origin-text
mycel branch create community-mainline
mycel merge rev:branchA rev:branchB
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

This version is a conceptual spec. The three most valuable next additions are:

1. **Wire protocol**: concrete fields for `HELLO`, `WANT`, and `OBJECT`
2. **Canonical serialization spec**: prevent hash mismatch across implementations
3. **Merge semantics**: explicit block-based auto-merge rules

The next direct extension can be: **Mycel wire protocol v0.1**, defining packet formats and synchronization details.
