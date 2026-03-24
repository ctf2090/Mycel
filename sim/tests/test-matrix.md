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
- `view-sync-without-capability`: reject VIEW_ANNOUNCE when the seed omitted the required `view-sync` capability
  Reference JSON: `sim/tests/view-sync-without-capability.example.json`
- `snapshot-sync-without-capability`: reject SNAPSHOT_OFFER when the seed omitted the required `snapshot-sync` capability
  Reference JSON: `sim/tests/snapshot-sync-without-capability.example.json`
- `session-messages-after-bye`: reject the remaining sync transcript after the seed closes the session with an early `BYE`
  Reference JSON: `sim/tests/session-messages-after-bye.example.json`
- `session-bye-before-hello`: reject a sync transcript that emits `BYE` before the seed establishes the session with `HELLO`
  Reference JSON: `sim/tests/session-bye-before-hello.example.json`
- `session-snapshot-offer-before-hello`: reject a sync transcript that emits `SNAPSHOT_OFFER` before the seed establishes the session with `HELLO`
  Reference JSON: `sim/tests/session-snapshot-offer-before-hello.example.json`
- `session-snapshot-want-before-manifest`: reject a sync transcript that emits `WANT` for an offered snapshot before `MANIFEST` or `HEADS` establishes accepted sync roots
  Reference JSON: `sim/tests/session-snapshot-want-before-manifest.example.json`
- `session-view-announce-before-hello`: reject a sync transcript that emits `VIEW_ANNOUNCE` before the seed establishes the session with `HELLO`
  Reference JSON: `sim/tests/session-view-announce-before-hello.example.json`
- `session-view-announce-want-before-manifest`: reject a sync transcript that emits `WANT` for an announced view before `MANIFEST` or `HEADS` establishes accepted sync roots
  Reference JSON: `sim/tests/session-view-announce-want-before-manifest.example.json`
- `session-heads-before-hello`: reject a sync transcript that emits `HEADS` before the seed establishes the session with `HELLO`
  Reference JSON: `sim/tests/session-heads-before-hello.example.json`
- `session-manifest-before-hello`: reject a sync transcript that emits `MANIFEST` before the seed establishes the session with `HELLO`
  Reference JSON: `sim/tests/session-manifest-before-hello.example.json`
- `session-duplicate-hello`: reject a sync transcript that emits `HELLO` twice in one wire session
  Reference JSON: `sim/tests/session-duplicate-hello.example.json`
- `session-want-before-hello`: reject a sync transcript that emits `WANT` before the seed establishes the session with `HELLO`
  Reference JSON: `sim/tests/session-want-before-hello.example.json`
- `session-want-before-manifest`: reject a sync transcript that emits `WANT` after `HELLO` but before `MANIFEST` or `HEADS` establishes accepted sync roots
  Reference JSON: `sim/tests/session-want-before-manifest.example.json`
- `session-object-before-manifest`: reject a sync transcript that emits `OBJECT` immediately after `HELLO`, before any `WANT` request or accepted sync roots exist
  Reference JSON: `sim/tests/session-object-before-manifest.example.json`
- `session-stale-dependency-object-after-heads-replace`: reject a withdrawn dependency `OBJECT` after `HEADS replace=true` clears the pending request set for the old root set
  Reference JSON: `sim/tests/session-stale-dependency-object-after-heads-replace.example.json`

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

## Production Replication

- `resync-idempotency`: reader syncs once to get current, then syncs again; the second pass must write zero new objects and produce no errors
  Reference JSON: `sim/tests/resync-idempotency.example.json`
- `depth-3-catchup`: reader at depth 2 catches up to seed at depth 3 in a single HEADS/WANT pass; only the delta revision is fetched
  Reference JSON: `sim/tests/depth-3-catchup.example.json`
