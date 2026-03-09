# Test Matrix

## Positive

- `first-sync-empty-reader`: empty reader syncs from `minimal-valid`
- `three-peer-consistency`: two readers converge on the same verified object set

## Negative

- `reject-hash-mismatch`: reject invalid object body hash
- `reject-signature-mismatch`: reject invalid object or wire signature

## Recovery

- `recover-missing-objects`: recover missing objects via `WANT`

## Deferred

- snapshot-assisted catch-up
- localhost multi-process runs
- accepted-head comparison reports
