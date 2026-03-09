# partial-want-recovery

Purpose:

- verify recovery when a reader initially lacks part of the required object graph

Expected use:

- one peer advertises heads
- the reader discovers missing canonical object IDs
- the reader uses `WANT` to recover the missing objects

Expected outcomes:

- missing objects are requested explicitly
- recovery succeeds without rebuilding the whole store manually
