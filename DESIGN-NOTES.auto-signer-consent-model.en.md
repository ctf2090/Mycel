# Auto-signer Consent Model

Status: design draft

This note describes how a Mycel-based system should model consent for automatic threshold signers.

The main design principle is:

- a signer consents to enrollment and policy scope in advance
- a signer does not manually approve each later transaction
- automatic signing is valid only inside accepted and bounded policy conditions
- consent boundaries must remain visible, revocable, and auditable

## 0. Goal

Enable automatic threshold signing without pretending that "no human approval" means "no human consent."

This model separates:

- enrollment consent
- policy-scope consent
- operational state
- execution events

It does not treat later transaction execution as implicit unlimited authorization.

## 1. Consent Boundary

The signer's consent happens when the signer joins the signer pool and accepts a defined policy scope.

The signer does not need to:

- see each transaction before it is signed
- click approve for each transaction
- know whether a given execution used that signer's share in real time

The signer must still know:

- that the signer is enrolled
- which fund or signer pool the signer belongs to
- which trigger classes may activate automatic signing
- what amount, rate, and destination constraints apply
- how to pause, revoke, or rotate participation

## 2. Core Definitions

### 2.1 Enrollment Consent

Enrollment consent means the signer knowingly joins the signer pool.

Minimum meaning:

- "my key or key share is part of this custody system"
- "my signer node may sign automatically if policy conditions are met"

### 2.2 Policy-scope Consent

Policy-scope consent means the signer agrees to one bounded execution envelope.

Minimum boundaries:

- allowed trigger types
- maximum amount per execution
- maximum amount per time window
- allowed assets
- allowed destination classes or allowlists
- effective start and end time

### 2.3 Operational State

Operational state means whether automatic signing is currently allowed.

Typical values:

- `active`
- `paused`
- `revoked`
- `expired`
- `rotating`

### 2.4 Execution Event

An execution event is a later spend attempt evaluated against the accepted policy scope.

An execution event is not itself a new consent grant.

## 3. What Counts as Valid Consent

Valid consent requires all of the following:

1. the signer was explicitly enrolled
2. the signer had access to the applicable policy scope
3. the signer was not paused, revoked, or expired at evaluation time
4. the execution matched the accepted policy scope
5. the system preserved an audit trail linking signer state to execution result

If any of these fail, automatic signing should be treated as invalid or out of policy.

## 4. What Does Not Count as Consent

The following should not be treated as valid consent:

- silent inclusion in a signer pool
- inferred consent from general app usage
- unlimited automatic signing without policy bounds
- reuse of a signer outside the accepted fund or policy scope
- signing after revoke, expiry, or explicit pause
- hidden widening of local runtime policy

## 5. Consent Lifecycle

### 5.1 Join

The signer is enrolled into one signer pool.

Required outputs:

- signer enrollment record
- signer key reference
- signer pool or fund reference
- initial operational state

### 5.2 Activate

The signer becomes eligible for automatic signing under one accepted policy scope.

Required outputs:

- effective policy reference
- effective time window
- operational state set to `active`

### 5.3 Pause

The signer remains enrolled but automatic signing is temporarily disabled.

Expected behavior:

- no new signatures should be produced
- blocked or skipped outcomes should remain auditable

### 5.4 Revoke

The signer is removed from future eligibility.

Expected behavior:

- no new intents may use that signer as active authority
- older history remains preserved

### 5.5 Rotate

The signer moves to a new key, new signer-set version, or new policy scope.

Expected behavior:

- old and new identity links remain traceable
- future execution binds only to the new effective version

## 6. Recommended Records

### 6.1 Signer Enrollment Record

Suggested fields:

- `enrollment_id`
- `signer_id`
- `signer_key_ref`
- `fund_id`
- `signer_set_id`
- `status`
- `joined_at`

### 6.2 Consent Scope Record

Suggested fields:

- `consent_scope_id`
- `enrollment_id`
- `policy_id`
- `max_amount_per_execution`
- `max_amount_per_day`
- `allowed_trigger_types`
- `allowed_assets`
- `destination_allowlist_ref`
- `effective_from`
- `effective_until`

### 6.3 Consent State Record

Suggested fields:

- `state_id`
- `enrollment_id`
- `state`
- `reason`
- `created_at`

### 6.4 Consent Evidence Record

Suggested fields:

- `evidence_id`
- `enrollment_id`
- `consent_scope_id`
- `accepted_at`
- `source_ref`

This can remain deployment-specific, but the system should preserve some evidence that the signer knowingly accepted enrollment and scope.

### 6.5 Auto-sign Outcome Record

Suggested fields:

- `outcome_id`
- `intent_id`
- `signer_id`
- `consent_scope_id`
- `result`
- `reason`
- `created_at`

Typical `result` values:

- `signed`
- `blocked-paused`
- `blocked-revoked`
- `blocked-expired`
- `blocked-policy-mismatch`

## 7. Client Responsibilities

A conforming client should let the signer inspect:

- signer enrollment status
- current consent scope
- current operational state
- recent automatic-sign outcomes
- pause and revoke controls
- pending rotations

A conforming client should not present:

- hidden automatic signing
- ambiguous or unlimited policy scope
- a false impression that "automation" removed the need for consent

## 8. Runtime Responsibilities

The signer runtime should:

- refuse signing if enrollment is missing
- refuse signing if consent scope is missing or expired
- refuse signing if state is paused or revoked
- log explicit reasons for blocked signing
- bind each signing result to one signer-set version and one policy scope

The signer runtime should not:

- widen policy locally
- silently ignore consent-state changes
- keep signing after loss of synchronization with accepted state

## 9. Failure Cases

### 9.1 Signer never knowingly enrolled

Treat all automatic signing from that identity as invalid for future operation review.

### 9.2 Scope mismatch

Do not sign. Record a policy mismatch outcome.

### 9.3 Pause not propagated

Do not silently continue. Mark the event as a state-synchronization failure.

### 9.4 Rotated signer still signing under old scope

Block future signing under the old effective state and preserve the mismatch trail.

## 10. Minimal First-client Rules

For a first interoperable client, I recommend:

- explicit signer enrollment UI
- explicit policy-scope display
- visible `active / paused / revoked / expired` state
- visible history of auto-sign outcomes
- explicit revoke and pause controls
- no hidden or implicit enrollment path

## 11. Open Questions

- How strong should consent evidence be in a minimal deployment?
- Should pause and revoke be signer-local only, governance-driven only, or both?
- Should one signer be allowed to hold multiple concurrent consent scopes for the same fund?
