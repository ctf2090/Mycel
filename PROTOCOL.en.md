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

### 3.1 Content Addressing

All objects are identified by content hash:

```text
object_id = hash(canonical_serialization(object))
```

### 3.2 Signature is Mandatory

All author-generated Patch, Revision, and View objects must include a digital signature.

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

- `doc_id`: stable document ID
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

## 5. Serialization and Hashing

### 5.1 Canonical Serialization

Before hashing, all objects must be transformed into a fixed canonical form:

- fixed key order
- UTF-8
- no unnecessary whitespace
- fixed array order
- fixed number format

### 5.2 Hash

In v0.1, hash can be defined as:

```text
hash = BLAKE3(canonical_bytes)
```

If a conservative choice is preferred, SHA-256 is also possible. But one network must fix one algorithm and not mix both.

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
3. Fetch missing objects by object hash
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

`WANT`:

```json
{
  "type": "want",
  "objects": ["rev:merge001", "patch:91ac"]
}
```

`OBJECT`:

```json
{
  "type": "object",
  "object_id": "patch:91ac",
  "payload": { "...": "..." }
}
```

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
