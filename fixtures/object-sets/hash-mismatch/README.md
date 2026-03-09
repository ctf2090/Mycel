# hash-mismatch

Purpose:

- verify that a peer rejects an `OBJECT` whose body hash does not match the envelope declaration

Expected outcomes:

- object rejection is recorded
- invalid object is not indexed
- session may continue if the remaining peer behavior is valid
