# Mycel Blind-address Threat Model

Status: design draft

This note describes the main threat surfaces, trust boundaries, and control requirements for a Mycel deployment that uses a blind-address custody model.

In this note, a blind-address model means:

- signers know the accepted `fund_id`, `policy_id`, `signer_set_id`, and signing conditions
- signers do not necessarily know the externally visible settlement address or full address mapping
- coordinator and executor roles may know more address-level information than signers

The goal is not perfect secrecy.

The goal is to reduce unnecessary address knowledge while preserving explicit consent, enforceable policy, and auditable execution.

## 0. Scope

This note applies to deployments where:

- Mycel carries accepted governance and custody state
- an m-of-n signer layer produces signatures or signature shares
- a coordinator or execution layer handles address mapping and settlement

This note does not define:

- one mandatory custody architecture
- one mandatory blockchain or payment rail
- one complete anonymity system

## 1. Security Goals

A blind-address deployment should try to preserve all of these goals:

- reduce the number of parties that know the full address mapping
- prevent signers from learning more address-level information than necessary
- preserve signer consent at the fund and policy scope
- prevent unauthorized execution even if some role metadata leaks
- preserve dispute resolution and post-event auditability

## 2. Protected Assets

The system should treat at least these as protected assets:

- signer key shares or signing authority
- `fund_id -> address` mappings
- signer-set membership and rotation history
- execution intents and their settlement targets
- runtime logs, receipts, and monitoring output
- timing and behavioral metadata that may reveal address identity indirectly

## 3. Roles and Knowledge Boundaries

### 3.1 Signer

The signer should know:

- accepted fund and policy scope
- signing conditions
- whether the system is paused, revoked, or rotated

The signer should not automatically know:

- the full settlement address map
- every other signer identity
- executor-internal wallet topology

### 3.2 Coordinator

The coordinator assembles a signing flow.

The coordinator may know:

- which signer set is required
- which intent is pending
- enough metadata to route requests

The coordinator should not require unrestricted access to:

- raw signer secrets
- unnecessary long-term address inventory

### 3.3 Executor

The executor performs the external settlement step.

The executor may know:

- the actual address or settlement target
- the final assembled signature or settlement authorization

The executor is therefore a high-value target.

### 3.4 Peer and Governance Nodes

Peer nodes and governance state maintain accepted records.

They should preserve:

- policy history
- signer-set history
- receipts and disputes

They should not replicate:

- raw custody secrets
- unnecessary address-revealing runtime internals

## 4. Trust Boundaries

At minimum, the design should treat these as separate trust boundaries:

- governance state vs signer runtime
- signer runtime vs coordinator
- coordinator vs executor
- accepted records vs local runtime logs
- fund identity vs settlement address identity

If those boundaries collapse operationally, the blind-address model weakens sharply.

## 5. Main Threats

### 5.1 Address-mapping leakage

If `fund_id -> address` mapping leaks through APIs, logs, dashboards, receipts, or support workflows, the blind-address property largely collapses.

Typical sources:

- verbose runtime logs
- support tickets or operator chat
- monitoring labels
- payment processor callbacks
- static reports that reveal both fund and address references

### 5.2 Coordinator concentration risk

If the coordinator sees too much fund, signer, and intent metadata, it becomes a correlation hub.

Risks:

- internal abuse
- targeted compromise
- legal or operational coercion
- cross-fund graph reconstruction

### 5.3 Executor concentration risk

The executor may be the only layer that knows the real settlement address.

If the executor is compromised, an attacker may gain:

- address knowledge
- settlement timing knowledge
- target selection knowledge
- possibly execution capability

Blindness for signers does not protect against executor compromise.

### 5.4 Signer-side inference

Even if the signer never sees the raw address, the signer may infer it from:

- repeated transaction timing
- stable amounts
- fixed allowlists
- known counterparties
- recurring settlement behavior

Blindness is therefore probabilistic, not absolute.

### 5.5 Weak consent

If blind-address design hides too much, a signer may no longer understand the real risk scope of participation.

This creates:

- weak informed consent
- governance disputes
- poor post-incident accountability

A signer should be blind to unnecessary address detail, not blind to policy scope.

### 5.6 Metadata forgery and phishing

If a signer relies on summarized metadata instead of strong verified context, an attacker may attempt to forge:

- intent summaries
- policy references
- signer-set references
- pause or revoke state

Blind-address systems increase dependence on trustworthy metadata verification.

### 5.7 Rotation mismatch

If signer rotation, policy rotation, and address rotation are not synchronized, the system may drift into ambiguous authority.

Examples:

- a signer believes one policy is active while the executor uses another mapping
- a rotated signer set still routes to an old settlement address
- an address changes without corresponding governance visibility

### 5.8 Audit failure

If the system hides too much address information and retains poor sealed audit material, post-event reconstruction may fail.

That creates risk in:

- disputes
- incident response
- external compliance review
- internal governance review

## 6. Threat Actors

The design should assume at least these actors:

- external attackers targeting signers, coordinator, or executor
- malicious or careless operators
- compromised runtime hosts
- colluding insiders across layers
- observers correlating timing and settlement behavior
- governance participants who later dispute authorization scope

## 7. Required Controls

### 7.1 Mapping isolation

Keep address mapping isolated from ordinary signer-visible state.

Recommended controls:

- restrict mapping access to the minimal executor boundary
- avoid copying raw mappings into operator dashboards
- avoid address-bearing logs by default

### 7.2 Strong signer verification

A signer should verify accepted state, not just human-readable summaries.

Recommended controls:

- verify `fund_id`, `policy_id`, `signer_set_id`, and intent digest
- verify pause, revoke, and rotation state
- reject unsigned or unverifiable coordinator prompts

### 7.3 Limited execution authority

Blindness is not a substitute for policy restriction.

Recommended controls:

- per-intent limits
- per-day limits
- allowlists
- cooldowns
- timelocks
- mandatory mismatch receipts

### 7.4 Audit-capable secrecy

The system should preserve sealed audit material without making it universally visible.

Recommended controls:

- retain receipts that can later prove which mapping was used
- separate public audit records from restricted forensic records
- ensure disputes can reveal enough evidence without exposing all mappings by default

### 7.5 Rotation discipline

Signer, policy, and address rotation should be explicit and linked.

Recommended controls:

- publish rotation records
- expire stale mappings
- reject execution against superseded signer-set versions

### 7.6 Role separation

Do not assume peer, coordinator, and executor roles are safe to merge by default.

Recommended controls:

- separate deployment roles when stakes are high
- minimize standing privileges
- review what each role can infer even without direct address access

## 8. Residual Limits

Blind-address design cannot guarantee:

- perfect signer ignorance
- perfect anonymity
- safety against executor compromise
- safety against strong metadata correlation

It mainly reduces unnecessary address exposure.

It does not remove the need for:

- narrow policy scope
- signer independence
- monitoring
- incident response
- governance clarity

## 9. Practical Rule

A blind-address deployment is defensible only if:

1. signers remain clearly informed about policy scope
2. address knowledge is minimized but not unauditable
3. coordinator and executor power is explicitly constrained
4. the system can reconstruct disputed executions later

If any of those conditions fail, the design becomes either unsafe or theatrically private rather than operationally secure.
