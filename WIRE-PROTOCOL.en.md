# Mycel Wire Protocol v0.1 (Draft)

Language: English | [繁體中文](./WIRE-PROTOCOL.zh-TW.md)

## 0. Scope

This document defines the transport-level message format and minimum synchronization flow for Mycel nodes.

Goals for v0.1:

- Define a stable wire envelope
- Define normative fields for the v0.1 sync message set
- Keep implementation neutral, technical, and interoperable

## 1. Conformance

A node is v0.1 wire-compatible if it:

1. Produces and parses the envelope in Section 2
2. Implements `HELLO`, `MANIFEST`, `HEADS`, `WANT`, `OBJECT`, `BYE`, and `ERROR`
3. Verifies envelope signatures and object hash/signature before acceptance
4. Implements `SNAPSHOT_OFFER` if it advertises `snapshot-sync`
5. Implements `VIEW_ANNOUNCE` if it advertises `view-sync`

## 2. Message Envelope

All wire messages MUST use this envelope:

```json
{
  "type": "HELLO",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:5f0c...",
  "timestamp": "2026-03-08T20:00:00+08:00",
  "from": "node:alpha",
  "payload": {},
  "sig": "sig:..."
}
```

Required fields:

- `type`: message kind
- `version`: fixed as `mycel-wire/0.1`
- `msg_id`: unique message ID
- `timestamp`: RFC 3339 timestamp
- `from`: sender node ID
- `payload`: message-specific body
- `sig`: signature over canonicalized envelope without `sig`

The wire-message signature rules for every message kind are defined normatively in Section 3.1.
Canonicalization of the envelope MUST follow Appendix A of `PROTOCOL.en.md`.

## 3. Message Types

v0.1 defines the following message kinds:

- `HELLO`
- `MANIFEST`
- `HEADS`
- `WANT`
- `OBJECT`
- `SNAPSHOT_OFFER`
- `VIEW_ANNOUNCE`
- `BYE`
- `ERROR`

## 3.1 Wire Message Signature Matrix (Normative)

All v0.1 wire messages require an envelope signature.

| Message type | Envelope `sig` | Signed payload |
| --- | --- | --- |
| `HELLO` | required | canonical envelope with `sig` omitted |
| `MANIFEST` | required | canonical envelope with `sig` omitted |
| `HEADS` | required | canonical envelope with `sig` omitted |
| `WANT` | required | canonical envelope with `sig` omitted |
| `OBJECT` | required | canonical envelope with `sig` omitted |
| `SNAPSHOT_OFFER` | required | canonical envelope with `sig` omitted |
| `VIEW_ANNOUNCE` | required | canonical envelope with `sig` omitted |
| `BYE` | required | canonical envelope with `sig` omitted |
| `ERROR` | required | canonical envelope with `sig` omitted |

Rules:

1. A receiver MUST reject any v0.1 wire message that is missing `sig`.
2. The `from` node key MUST verify the envelope signature over the canonical envelope with `sig` omitted.
3. The envelope `sig` authenticates transport metadata only; it does not replace object-level signatures inside `OBJECT.body`.
4. The `sig` field itself MUST NOT be included in the signed envelope payload.

## 4. HELLO

`HELLO` starts a session and advertises capabilities.

```json
{
  "type": "HELLO",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:hello-001",
  "timestamp": "2026-03-08T20:00:00+08:00",
  "from": "node:alpha",
  "payload": {
    "node_id": "node:alpha",
    "agent": "mycel-node/0.1",
    "capabilities": ["patch-sync", "snapshot-sync", "view-sync"],
    "topics": ["text/core", "text/commentary"],
    "nonce": "n:01f4..."
  },
  "sig": "sig:..."
}
```

Required `payload` fields:

- `node_id`
- `capabilities`
- `nonce`

## 5. MANIFEST

`MANIFEST` advertises a node's currently served sync surface.
It is a wire message summary, not a content-addressed protocol object.

```json
{
  "type": "MANIFEST",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:manifest-001",
  "timestamp": "2026-03-08T20:00:10+08:00",
  "from": "node:alpha",
  "payload": {
    "node_id": "node:alpha",
    "capabilities": ["patch-sync", "snapshot-sync", "view-sync"],
    "topics": ["text/core", "text/commentary"],
    "heads": {
      "doc:origin-text": ["rev:c7d4", "rev:b351"]
    },
    "snapshots": ["snap:44cc"],
    "views": ["view:9aa0"]
  },
  "sig": "sig:..."
}
```

Required `payload` fields:

- `node_id`
- `capabilities`
- `heads`

Field rules:

- `heads` is a map of `doc_id -> non-empty array of canonical revision IDs`
- each head list MUST contain unique revision IDs
- each head list SHOULD be sent in lexicographic ascending order for stable replay
- `snapshots`, if present, MUST contain canonical snapshot IDs
- `views`, if present, MUST contain canonical view IDs

## 6. HEADS

`HEADS` announces current heads for one or more documents.

```json
{
  "type": "HEADS",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:heads-001",
  "timestamp": "2026-03-08T20:00:30+08:00",
  "from": "node:alpha",
  "payload": {
    "documents": {
      "doc:origin-text": ["rev:c7d4", "rev:b351"],
      "doc:governance-rules": ["rev:91de"]
    },
    "replace": true
  },
  "sig": "sig:..."
}
```

Required `payload` fields:

- `documents`
- `replace`

Field rules:

- `documents` is a non-empty map of `doc_id -> non-empty array of canonical revision IDs`
- each head list MUST contain unique revision IDs
- each head list SHOULD be sent in lexicographic ascending order for stable replay
- if `replace` is `true`, the sender declares that the listed head sets replace its prior head advertisement for the same listed documents
- if `replace` is `false`, the sender declares that the listed head sets are additive hints only

## 7. WANT

`WANT` requests missing objects by canonical object ID.
In v0.1, these IDs are typed content-addressed IDs such as `rev:<object_hash>` or `patch:<object_hash>`.
Logical IDs such as `doc_id` and `block_id` are not valid `WANT` targets.

```json
{
  "type": "WANT",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:want-001",
  "timestamp": "2026-03-08T20:01:00+08:00",
  "from": "node:beta",
  "payload": {
    "objects": ["rev:c7d4", "patch:a12f"],
    "max_items": 256
  },
  "sig": "sig:..."
}
```

Required `payload` fields:

- `objects`: non-empty list of canonical object IDs

## 8. OBJECT

`OBJECT` transmits one object payload.

```json
{
  "type": "OBJECT",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:obj-001",
  "timestamp": "2026-03-08T20:01:02+08:00",
  "from": "node:alpha",
  "payload": {
    "object_id": "patch:a12f",
    "object_type": "patch",
    "encoding": "json",
    "hash_alg": "blake3",
    "hash": "hash:...",
    "body": {"type": "patch", "...": "..."}
  },
  "sig": "sig:..."
}
```

Required `payload` fields:

- `object_id`
- `object_type`
- `encoding`
- `hash_alg`
- `hash`
- `body`

Field meaning:

- `object_id`: canonical typed object ID, reconstructed as `<object_type-prefix>:<hash>`
- `hash`: raw digest of the canonicalized `body`
- `body`: canonical object body before any transport wrapping

For content-addressed v0.1 object types:

- `patch` uses `patch_id`
- `revision` uses `revision_id`
- `view` uses `view_id`
- `snapshot` uses `snapshot_id`

Receiver MUST:

1. Recompute `hash(body)` and compare with `hash`
2. Reconstruct the expected `object_id` from `object_type` and `hash`, and compare with `object_id`
3. If `body` contains a derived object-ID field for its type, verify that it matches `object_id`
4. Verify object-level signature according to the normative object signature rules in `PROTOCOL.en.md`
5. Store object only when verification passes

## 9. SNAPSHOT_OFFER

`SNAPSHOT_OFFER` advertises that a snapshot is available for fetch by `WANT`.

```json
{
  "type": "SNAPSHOT_OFFER",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:snap-001",
  "timestamp": "2026-03-08T20:02:00+08:00",
  "from": "node:alpha",
  "payload": {
    "snapshot_id": "snap:44cc",
    "root_hash": "hash:snapshot-root",
    "documents": ["doc:origin-text"],
    "object_count": 3912,
    "size_bytes": 1048576
  },
  "sig": "sig:..."
}
```

Required `payload` fields:

- `snapshot_id`
- `root_hash`
- `documents`

Field rules:

- `snapshot_id` MUST be a canonical snapshot ID
- `documents` MUST be a non-empty array of `doc_id`
- `object_count`, if present, MUST be a non-negative integer
- `size_bytes`, if present, MUST be a non-negative integer
- when the receiver later fetches the referenced Snapshot object, its `snapshot_id` and `root_hash` MUST match this offer

## 10. VIEW_ANNOUNCE

`VIEW_ANNOUNCE` advertises that a signed View object is available for fetch by `WANT`.

```json
{
  "type": "VIEW_ANNOUNCE",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:view-001",
  "timestamp": "2026-03-08T20:02:05+08:00",
  "from": "node:alpha",
  "payload": {
    "view_id": "view:9aa0",
    "maintainer": "pk:community-curator",
    "documents": {
      "doc:origin-text": "rev:c7d4"
    }
  },
  "sig": "sig:..."
}
```

Required `payload` fields:

- `view_id`
- `maintainer`
- `documents`

Field rules:

- `view_id` MUST be a canonical view ID
- `documents` MUST be a non-empty map of `doc_id -> canonical revision ID`
- the fetched View object's `view_id`, `maintainer`, and `documents` MUST match the announcement

## 11. BYE

`BYE` closes a session gracefully.

```json
{
  "type": "BYE",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:bye-001",
  "timestamp": "2026-03-08T20:02:10+08:00",
  "from": "node:alpha",
  "payload": {
    "reason": "normal-close"
  },
  "sig": "sig:..."
}
```

Required `payload` fields:

- `reason`

Suggested `reason` values:

- `normal-close`
- `shutdown`
- `idle-timeout`
- `policy-reject`

## 12. Error Handling

On parse/validation failure, send `ERROR`:

```json
{
  "type": "ERROR",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:err-001",
  "timestamp": "2026-03-08T20:01:03+08:00",
  "from": "node:beta",
  "payload": {
    "in_reply_to": "msg:obj-001",
    "code": "INVALID_HASH",
    "detail": "Hash mismatch for object patch:a12f"
  },
  "sig": "sig:..."
}
```

Required `payload` fields:

- `in_reply_to`
- `code`

Suggested codes:

- `UNSUPPORTED_VERSION`
- `INVALID_SIGNATURE`
- `INVALID_HASH`
- `MALFORMED_MESSAGE`
- `OBJECT_NOT_FOUND`
- `RATE_LIMITED`

## 13. Minimal Sync Flow

1. `HELLO` exchange
2. `MANIFEST` / `HEADS` exchange
3. Receiver sends `WANT` for missing IDs
4. Sender replies with one or more `OBJECT`
5. Receiver verifies and stores
6. Optional `SNAPSHOT_OFFER` / `VIEW_ANNOUNCE`
7. `BYE` on graceful close

## 14. Security Notes

- Envelope signature does not replace object-level signature checks
- Reject unsigned or invalidly signed control messages by local policy
- Apply rate limits for repeated invalid traffic
- Keep transport and acceptance decisions separate

## 15. Next Extensions

Planned for later versions:

1. Streaming/chunked large objects
2. Compression negotiation
3. Capability-scoped authorization tokens
4. Replay-protection windows and nonce policy
