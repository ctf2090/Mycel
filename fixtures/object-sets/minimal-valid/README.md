# minimal-valid

Purpose:

- provide the smallest valid fixture set for first sync testing

Expected use:

- seed peer starts with one valid document chain
- one or more reader peers start empty
- readers fetch, verify, and index all required objects

Expected outcomes:

- sync completes without rejection
- canonical object IDs match expected values
- replay succeeds
- all readers converge on the same stored object set
