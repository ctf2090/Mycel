# Governance History Security

Status: design draft

This note describes how a Mycel-based system should secure governance history so that decisions remain verifiable, replayable, and recoverable over time.

The main design principle is:

- governance history must be tamper-evident
- governance history must be attributable
- governance history must be replayable
- governance history must be replicated
- governance history must remain auditable after failures or node loss

## 0. Goal

Enable governance records to survive adversarial modification, local loss, partial replication, and later re-audit without introducing a global mandatory consensus layer.

## 1. Security Objectives

A secure governance-history design should preserve five properties.

1. Integrity: records cannot be modified silently.
2. Attribution: records can be tied to signer identities.
3. Ordering: the relationship between proposals, approvals, resolutions, and receipts is reconstructable.
4. Rebuildability: accepted state can be recomputed from stored objects.
5. Availability: history survives beyond a single machine or operator.

## 2. Threat Model

Governance history should assume at least the following risks:

- a node stores corrupted or incomplete objects
- a maintainer key signs conflicting records
- a local operator hides superseded history
- a client shows current state without preserving why it became current
- a runtime publishes receipts that do not match an accepted proposal
- a subset of replicas disappears or becomes unavailable

## 3. Object-level Protections

Every governance-relevant object should be protected at the object layer.

Recommended rules:

- use canonical serialization
- derive content-addressed IDs where the protocol requires them
- require signatures for signed governance objects
- verify signatures only after canonical ID checks pass
- reject malformed or incomplete objects before indexing them

For app-layer governance records, the implementation should apply equivalent validation discipline even if the record family is not a core protocol primitive.

## 4. History-level Protections

A secure system must preserve not only current state, but the path by which that state was reached.

Recommended rules:

- keep proposal, approval, resolution, and receipt records distinct
- make later records reference earlier ones explicitly
- preserve superseded and rejected records rather than deleting them
- store signer-set version references alongside approvals
- preserve decision traces for accepted outcomes

Example governance chain:

```text
proposal
-> signer approvals
-> accepted resolution
-> execution receipt
-> balance or state update
```

## 5. Acceptance-level Protections

The client should not be free to reinterpret governance history ad hoc.

Recommended rules:

- derive accepted state from fixed profiles only
- treat signed governance signals as selector inputs
- preserve decision-trace outputs with accepted results
- do not let discretionary local policy silently rewrite accepted governance state

This keeps current governance output tied to reproducible rules rather than local preference.

## 6. Replication and Retention

Governance history is not secure if it exists on only one machine.

Recommended retention strategy:

- replicate governance documents across multiple independent nodes
- keep at least one archival replica
- preserve object-store copies and rebuildable indexes
- support snapshot-assisted recovery only as an optimization, not as the only source of truth

Recommended role separation:

- reader nodes may inspect current and historical governance state
- mirror or archivist nodes retain long-term copies
- governance-maintainer nodes publish signed decisions

## 7. Rebuild and Audit Procedures

A secure design should define routine rebuild and audit behavior.

Minimum rebuild procedure:

1. load stored governance-related objects
2. validate signatures and references
3. replay object and revision history
4. recompute accepted outcomes under the fixed profile
5. compare recomputed state with stored indexes and receipts

Recommended audit outputs:

- missing-object report
- invalid-signature report
- conflicting-approval report
- unresolved-reference report
- accepted-state mismatch report

## 8. Fund-specific Governance History

For fund or treasury workflows, the system should preserve at least these record families:

- fund manifest
- inflow record
- allocation proposal
- signer approval or attestation
- accepted allocation resolution
- disbursement receipt
- balance snapshot or replayable ledger state

Critical requirement:

- on-chain settlement evidence must be linked back to the exact accepted governance proposal that authorized it

Without that link, the fund has payment history but not secure governance history.

## 9. Failure and Recovery Cases

A secure governance-history model should define behavior for common failures.

### 9.1 Missing Replica

- recover objects from another replica
- rebuild indexes locally
- verify that accepted state is unchanged

### 9.2 Lost Signer Key

- preserve prior approvals as historical fact
- rotate to a new signer-set version through normal governance
- never rewrite old signer identity history

### 9.3 Conflicting Approvals

- preserve all conflicting records
- mark the conflict explicitly in audit state
- require accepted-state recomputation under the fixed profile

### 9.4 Execution Mismatch

- if a receipt does not match an accepted proposal, preserve it as a mismatch record
- do not silently merge it into accepted governance state

## 10. Minimal First-client Rules

For a first interoperable client, I recommend the following minimum:

- verify all governance-object signatures
- preserve proposal -> approval -> resolution -> receipt links
- persist decision traces
- rebuild governance state from object storage
- replicate to at least one additional node
- expose an audit view for accepted-state reasoning

## 11. Open Questions

- Should app-layer governance records gain a uniform signer envelope across all apps?
- Should mismatch and conflict records be first-class app records or local audit artifacts?
- How much of rebuild and audit output should be replicated versus kept local?
