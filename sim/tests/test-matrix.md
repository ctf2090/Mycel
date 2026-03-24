# Test Matrix

## Positive

- `first-sync-empty-reader`: empty reader syncs from `minimal-valid` (covered by `three-peer-consistency`)
- `three-peer-consistency`: two readers converge on the same verified object set
  Reference JSON: `sim/tests/three-peer-consistency.example.json`
- `incremental-sync`: reader already has genesis revision, receives follow-up revision via HEADS-based incremental sync
  Reference JSON: `sim/tests/incremental-sync.example.json`

### Product-Layer Positive Notes

| Simulator case | Product-layer counterpart | Coverage status |
|---|---|---|
| `first-sync-empty-reader` | `sync_peer_store_json_runs_first_time_sync_into_local_store` | both layers |
| `three-peer-consistency` | `sync_peer_store_json_converges_two_empty_readers_on_same_store_state` | both layers |
| `incremental-sync` | `sync_pull_json_replays_incremental_transcript_into_existing_store` | both layers |

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
- `session-stale-root-want-after-heads-replace`: reject a withdrawn root revision `WANT` after `HEADS replace=true` swaps out the old root set
  Reference JSON: `sim/tests/session-stale-root-want-after-heads-replace.example.json`
- `session-stale-root-object-after-heads-replace`: reject a withdrawn root revision `OBJECT` after `HEADS replace=true` clears the old pending request
  Reference JSON: `sim/tests/session-stale-root-object-after-heads-replace.example.json`
- `session-stale-dependency-object-after-heads-replace`: reject a withdrawn dependency `OBJECT` after `HEADS replace=true` clears the pending request set for the old root set
  Reference JSON: `sim/tests/session-stale-dependency-object-after-heads-replace.example.json`
- `session-stale-snapshot-want-after-heads-replace`: reject a stale snapshot `WANT` after `HEADS replace=true` withdraws the old root set that previously made the snapshot reachable
  Reference JSON: `sim/tests/session-stale-snapshot-want-after-heads-replace.example.json`
- `session-stale-view-want-after-heads-replace`: reject a stale view `WANT` after `HEADS replace=true` withdraws the old root set that previously made the announced view reachable
  Reference JSON: `sim/tests/session-stale-view-want-after-heads-replace.example.json`

### Product-Layer Coverage Notes

The simulator matrix above tracks the canonical negative sequencing cases. The
current `apps/mycel-cli/tests/sync_pull_smoke.rs` product-layer transcript tests
cover the same message-ordering rules for the pre-session and head-context
families below.

| Simulator case | Product-layer counterpart | Coverage status |
|---|---|---|
| `view-sync-without-capability` | `sync_pull_json_rejects_view_announce_without_advertised_capability` | both layers |
| `snapshot-sync-without-capability` | `sync_pull_json_rejects_snapshot_offer_without_advertised_capability` | both layers |
| `session-messages-after-bye` | `sync_pull_json_rejects_messages_after_bye` | both layers |
| `session-bye-before-hello` | `sync_pull_json_rejects_bye_before_hello` | both layers |
| `session-snapshot-offer-before-hello` | `sync_pull_json_rejects_snapshot_offer_before_hello` | both layers |
| `session-view-announce-before-hello` | `sync_pull_json_rejects_view_announce_before_hello` | both layers |
| `session-manifest-before-hello` | `sync_pull_json_rejects_manifest_before_hello` | both layers |
| `session-heads-before-hello` | `sync_pull_json_rejects_heads_before_hello` | both layers |
| `session-duplicate-hello` | `sync_pull_json_rejects_duplicate_hello` | both layers |
| `session-want-before-hello` | `sync_pull_json_rejects_want_before_hello` | both layers |
| `session-want-before-manifest` | `sync_pull_json_rejects_want_before_manifest_or_heads` | both layers |
| `session-snapshot-want-before-manifest` | `sync_pull_json_snapshot_offer_before_manifest_does_not_unlock_want` | both layers |
| `session-view-announce-want-before-manifest` | `sync_pull_json_view_announce_before_manifest_does_not_unlock_want` | both layers |
| `session-object-before-manifest` | `sync_pull_json_rejects_unrequested_object_before_manifest_or_heads` | both layers |
| `session-stale-root-want-after-heads-replace` | `sync_pull_json_rejects_stale_root_want_after_heads_replace` | both layers |
| `session-stale-root-object-after-heads-replace` | `sync_pull_json_rejects_stale_root_object_after_heads_replace` | both layers |
| `session-stale-dependency-object-after-heads-replace` | `sync_pull_json_rejects_stale_dependency_object_after_heads_replace` | both layers |
| `session-stale-snapshot-want-after-heads-replace` | `sync_pull_json_rejects_stale_snapshot_want_after_heads_replace` | both layers |
| `session-stale-view-want-after-heads-replace` | `sync_pull_json_rejects_stale_view_want_after_heads_replace` | both layers |
| `reject-hash-mismatch` | `sync_pull_json_rejects_invalid_object_hash_without_storing_objects` | both layers |
| `reject-signature-mismatch` | `sync_pull_json_rejects_invalid_wire_signature_without_storing_objects` | both layers |

Product-layer-only note:

- `sync_pull_json_allows_error_before_hello_but_still_requires_sync_messages`
  covers `ERROR` before `HELLO`; the simulator matrix does not currently define
  a dedicated `session-error-before-hello` case.

## Recovery

- `recover-missing-objects`: recover missing objects via `WANT`
  Reference JSON: `sim/tests/partial-want-recovery.example.json`
- `mixed-reader-recovery`: mixed reader set converges after WANT-based recovery
  Reference JSON: `sim/tests/mixed-reader-recovery.example.json`

### Product-Layer Recovery Notes

| Simulator case | Product-layer counterpart | Coverage status |
|---|---|---|
| `recover-missing-objects` | `sync_pull_json_recovers_missing_dependency_via_want_cycle` | both layers |
| `mixed-reader-recovery` | `sync_peer_store_json_converges_partial_and_empty_local_stores` | both layers |

## Capability-Gated

- `snapshot-catchup`: reader receives snapshot objects via SNAPSHOT_OFFER from a snapshot-capable seed
  Reference JSON: `sim/tests/snapshot-catchup.example.json`
- `view-sync`: reader receives governance view objects via VIEW_ANNOUNCE from a view-capable seed
  Reference JSON: `sim/tests/view-sync.example.json`

### Product-Layer Capability Notes

| Simulator case | Product-layer counterpart | Coverage status |
|---|---|---|
| `snapshot-catchup` | `sync_pull_json_accepts_snapshot_offer_when_capability_is_advertised` | both layers |
| `view-sync` | `sync_pull_json_accepts_view_announce_when_capability_is_advertised` | both layers |

## Multi-Process

- `localhost-multi-process`: two OS processes exchange wire messages via stdin/stdout pipe (mycel sync stream | mycel sync pull)
  Reference JSON: `sim/tests/localhost-multi-process.example.json`

### Product-Layer Multi-Process Notes

| Simulator case | Product-layer counterpart | Coverage status |
|---|---|---|
| `localhost-multi-process` | `sync_stream_to_pull_via_pipe_replays_peer_store_into_local_store` | both layers |

## Scalability

- `four-reader-multi-doc`: four readers each start empty and converge on a two-document verified object set from a single seed
  Reference JSON: `sim/tests/four-reader-multi-doc.example.json`

### Product-Layer Scalability Notes

| Simulator case | Product-layer counterpart | Coverage status |
|---|---|---|
| `four-reader-multi-doc` | `sync_peer_store_json_converges_four_readers_on_same_multi_doc_state` | both layers |

## Production Replication

- `resync-idempotency`: reader syncs once to get current, then syncs again; the second pass must write zero new objects and produce no errors
  Reference JSON: `sim/tests/resync-idempotency.example.json`
- `depth-3-catchup`: reader at depth 2 catches up to seed at depth 3 in a single HEADS/WANT pass; only the delta revision is fetched
  Reference JSON: `sim/tests/depth-3-catchup.example.json`
- `partial-doc-sync`: reader requests only a subset of the seed's documents and ends with a stable partial store plus correct accepted heads for the requested subset
  Reference JSON: `sim/tests/partial-doc-sync.example.json`

### Product-Layer Production Replication Notes

| Simulator case | Product-layer counterpart | Coverage status |
|---|---|---|
| `resync-idempotency` | `sync_peer_store_json_reports_noop_when_local_store_is_current` | both layers |
| `depth-3-catchup` | `sync_pull_json_replays_depth_3_catchup_transcript_into_existing_store` | both layers |
| `partial-doc-sync` | `sync_peer_store_json_limits_sync_to_requested_document_subset` | both layers |
