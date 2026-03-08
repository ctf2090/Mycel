# Mycel Wire Protocol v0.1 (Draft)

Language: English | [繁體中文](./WIRE-PROTOCOL.zh-TW.md)

## 0. Scope

This document defines the transport-level message format and minimum synchronization flow for Mycel nodes.

Goals for v0.1:

- Define a stable wire envelope
- Define minimum fields for `HELLO`, `WANT`, and `OBJECT`
- Keep implementation neutral, technical, and interoperable

## 1. Conformance

A node is v0.1 wire-compatible if it:

1. Produces and parses the envelope in Section 2
2. Implements `HELLO`, `WANT`, and `OBJECT`
3. Verifies object hash and signature before acceptance

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

## 5. WANT

`WANT` requests missing objects by ID.

```json
{
  "type": "WANT",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:want-001",
  "timestamp": "2026-03-08T20:01:00+08:00",
  "from": "node:beta",
  "payload": {
    "objects": ["rev:merge001", "patch:91ac"],
    "max_items": 256
  },
  "sig": "sig:..."
}
```

Required `payload` fields:

- `objects`: non-empty list of object IDs

## 6. OBJECT

`OBJECT` transmits one object payload.

```json
{
  "type": "OBJECT",
  "version": "mycel-wire/0.1",
  "msg_id": "msg:obj-001",
  "timestamp": "2026-03-08T20:01:02+08:00",
  "from": "node:alpha",
  "payload": {
    "object_id": "patch:91ac",
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

Receiver MUST:

1. Recompute `hash(body)` and compare with `hash`
2. Verify object-level signature (if present by object type rules)
3. Store object only when verification passes

## 7. Error Handling

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
    "detail": "Hash mismatch for object patch:91ac"
  },
  "sig": "sig:..."
}
```

Suggested codes:

- `UNSUPPORTED_VERSION`
- `INVALID_SIGNATURE`
- `INVALID_HASH`
- `MALFORMED_MESSAGE`
- `OBJECT_NOT_FOUND`
- `RATE_LIMITED`

## 8. Minimal Sync Flow

1. `HELLO` exchange
2. `MANIFEST` / `HEADS` exchange
3. Receiver sends `WANT` for missing IDs
4. Sender replies with one or more `OBJECT`
5. Receiver verifies and stores
6. Optional `SNAPSHOT_OFFER` / `VIEW_ANNOUNCE`
7. `BYE` on graceful close

## 9. Security Notes

- Envelope signature does not replace object-level signature checks
- Reject unsigned or invalidly signed control messages by local policy
- Apply rate limits for repeated invalid traffic
- Keep transport and acceptance decisions separate

## 10. Next Extensions

Planned for later versions:

1. Streaming/chunked large objects
2. Compression negotiation
3. Capability-scoped authorization tokens
4. Replay-protection windows and nonce policy
