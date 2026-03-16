# Test Matrix

## Positive

- `first-sync-empty-reader`: empty reader syncs from `minimal-valid` (covered by `three-peer-consistency`)
- `three-peer-consistency`: two readers converge on the same verified object set
  Reference JSON: `sim/tests/three-peer-consistency.example.json`
- `incremental-sync`: reader already has genesis revision, receives follow-up revision via HEADS-based incremental sync
  Reference JSON: `sim/tests/incremental-sync.example.json`

## Negative

- `reject-hash-mismatch`: reject invalid object body hash
  Reference JSON: `sim/tests/hash-mismatch.example.json`
- `reject-signature-mismatch`: reject invalid object or wire signature
  Reference JSON: `sim/tests/signature-mismatch.example.json`

## Recovery

- `recover-missing-objects`: recover missing objects via `WANT`
  Reference JSON: `sim/tests/partial-want-recovery.example.json`
- `mixed-reader-recovery`: mixed reader set converges after WANT-based recovery
  Reference JSON: `sim/tests/mixed-reader-recovery.example.json`

## Capability-Gated

- `snapshot-catchup`: reader receives snapshot objects via SNAPSHOT_OFFER from a snapshot-capable seed
  Reference JSON: `sim/tests/snapshot-catchup.example.json`
- `view-sync`: reader receives governance view objects via VIEW_ANNOUNCE from a view-capable seed
  Reference JSON: `sim/tests/view-sync.example.json`

## Multi-Process

- `localhost-multi-process`: two OS processes exchange wire messages via stdin/stdout pipe (mycel sync stream | mycel sync pull)
  Reference JSON: `sim/tests/localhost-multi-process.example.json`

## Scalability

- `four-reader-multi-doc`: four readers each start empty and converge on a two-document verified object set from a single seed
  Reference JSON: `sim/tests/four-reader-multi-doc.example.json`
