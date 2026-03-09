# Policy-driven m-of-n Custody

Status: design draft

This note describes a custody model where Mycel carries the policy and governance history for fund movements, while an m-of-n signer network executes approved transactions automatically under fixed policy constraints.

In this note, `m-of-n = members + threshold`.

The main design principle is:

- Mycel carries signer enrollment state, signer-set versions, policy bundles, trigger records, execution intent, and audit history
- signer-pool members knowingly enroll, but do not manually approve each transaction
- transaction execution is authorized in advance by accepted policy, then executed automatically by m-of-n signers
- the core protocol remains neutral and purely technical

## 0. Goal

Enable decentralized custody without requiring a human per-transaction approval step.

Keep in Mycel:

- signer enrollment and revocation state
- signer-set definitions and rotation history
- policy bundles and trigger conditions
- execution intents and receipts
- audit and dispute history

Keep outside Mycel core:

- raw private-key material
- partial-signature assembly internals
- hardware-wallet or HSM specific logic
- irreversible settlement side effects

## 1. Model Summary

This model is not traditional human-reviewed multisig.

It is:

- policy-authorized
- m-of-n-executed
- audit-preserving
- signer-aware but not per-transaction interactive

The practical rule is:

1. a signer knowingly joins a signer pool
2. the signer accepts a fixed custody policy scope
3. when an accepted trigger matches that policy scope, the signer node may sign automatically
4. once the threshold is reached, the transaction may be broadcast

## 2. Four Layers

### 2.1 Client Layer

The client is the user-facing layer.

Responsibilities:

- display fund policy state
- display signer-pool membership and signer-set versions
- display execution intents, receipts, pauses, revocations, and disputes
- explain why an execution was or was not allowed

Non-responsibilities:

- do not hold replicated private keys by default
- do not redefine accepted custody policy
- do not bypass accepted trigger state

### 2.2 Governance Layer

The governance layer is carried by Mycel.

Responsibilities:

- define signer-set membership
- define policy bundles
- define execution eligibility rules
- define pause, revoke, and rotation records
- preserve a full audit trail

Non-responsibilities:

- do not directly produce raw signatures
- do not hold custody secrets as replicated state

### 2.3 Threshold Signer Layer

The threshold signer layer is a signer network or signer runtime.

Responsibilities:

- verify accepted policy state
- verify trigger bundles
- verify amount caps, cooldowns, allowlists, and signer-set version
- produce partial signatures when policy conditions are met

Non-responsibilities:

- do not reinterpret governance rules locally
- do not sign outside the accepted policy scope
- do not silently continue when the system is paused or revoked

### 2.4 Execution Layer

The execution layer assembles threshold signatures and settles the transaction.

Responsibilities:

- assemble valid partial signatures
- broadcast the transaction or submit to the settlement rail
- publish execution receipts
- surface mismatch or failure records

Non-responsibilities:

- do not invent approval state
- do not settle transactions that fail policy verification

## 3. Signer Enrollment and Consent

This model requires explicit enrollment, even if later execution is automatic.

Recommended rules:

- a signer MUST know that the key or key share is part of a signer pool
- a signer MUST know the policy scope under which automatic signing may occur
- a signer MUST be able to pause, revoke, or rotate participation
- a signer SHOULD NOT be asked to approve each transaction manually

This separates:

- enrollment consent
- from per-transaction manual approval

The signer agrees to the policy envelope, not to each later execution event individually.

## 4. Core Custody Objects

### 4.1 Signer Enrollment Record

Defines that a signer joined the custody system.

Suggested fields:

- `enrollment_id`
- `signer_id`
- `signer_key_ref`
- `role`
- `status`
- `joined_at`
- `policy_scope_ref`

Typical `status` values:

- `active`
- `paused`
- `revoked`
- `retired`

### 4.2 Signer Set

Defines one m-of-n signer group.

Suggested fields:

- `signer_set_id`
- `fund_id`
- `members`
- `threshold`
- `version`
- `status`
- `created_at`

### 4.3 Policy Bundle

Defines what automatic execution is allowed to do.

Suggested fields:

- `policy_id`
- `fund_id`
- `signer_set_id`
- `allowed_trigger_types`
- `max_amount_per_execution`
- `max_amount_per_day`
- `cooldown_seconds`
- `destination_allowlist_ref`
- `asset_scope`
- `pause_state`
- `effective_from`
- `effective_until`

### 4.4 Trigger Record

Represents one accepted trigger that may enable execution.

Suggested fields:

- `trigger_id`
- `trigger_type`
- `trigger_ref`
- `fund_id`
- `policy_id`
- `amount_requested`
- `asset`
- `created_at`

### 4.5 Execution Intent

Represents one concrete spend attempt derived from an accepted trigger.

Suggested fields:

- `intent_id`
- `fund_id`
- `policy_id`
- `signer_set_id`
- `trigger_id`
- `outputs`
- `total_amount`
- `intent_hash`
- `status`
- `created_at`

Typical `status` values:

- `pending`
- `eligible`
- `blocked`
- `signed`
- `broadcast`
- `failed`

### 4.6 Signer Attestation

Represents one signer-side confirmation that the policy checks passed and a signature share was produced.

Suggested fields:

- `attestation_id`
- `intent_id`
- `signer_id`
- `signer_set_version`
- `intent_hash`
- `outcome`
- `created_at`

Typical `outcome` values:

- `signed`
- `rejected`
- `skipped-paused`
- `skipped-revoked`
- `skipped-policy-mismatch`

### 4.7 Execution Receipt

Represents the final settlement result.

Suggested fields:

- `receipt_id`
- `intent_id`
- `executor`
- `settlement_ref`
- `status`
- `submitted_at`
- `confirmed_at`
- `error_summary`

## 5. Automatic Approval Flow

Recommended flow:

1. an accepted policy bundle exists for one fund
2. an accepted trigger record is created
3. the system derives one execution intent
4. signer nodes validate:
   - signer enrollment state
   - signer-set version
   - policy bundle validity
   - pause and revoke state
   - amount, rate, and destination constraints
5. eligible signer nodes produce partial signatures automatically
6. once the threshold is reached, the executor broadcasts the transaction
7. the system writes an execution receipt

This keeps the approval boundary at policy acceptance time, not at transaction-click time.

## 6. Committee Selection Options

There are two practical committee models.

### 6.1 Fixed Signer Set

All active members of one signer set may participate.

Tradeoff: simplest to implement, but signer exposure is more static.

### 6.2 Large Pool with Derived Signing Committee

A larger signer pool exists, and a smaller signing committee is derived for each intent or epoch.

Possible derivation inputs:

- `intent_hash`
- `epoch_id`
- `signer_set_version`
- a deterministic random beacon or VRF output

Tradeoff: more decentralized and less predictable, but materially more complex.

For a first implementation, I recommend the fixed signer-set model.

## 7. Guardrails

Automatic threshold custody needs hard limits.

Recommended minimum guardrails:

1. every signer MUST enroll explicitly
2. every execution MUST match one accepted policy bundle
3. every policy bundle MUST have amount and rate limits
4. the system MUST support `pause`
5. the system MUST support `revoke`
6. the system MUST support signer-set rotation
7. the system MUST preserve failed and blocked intents as audit records
8. local runtime policy MUST NOT silently widen accepted policy scope

## 8. Pause, Revoke, and Rotation

The custody model should define three operational controls.

### 8.1 Pause

Temporarily stop automatic signing without removing the signer or policy.

### 8.2 Revoke

Remove a signer, policy bundle, or trigger class from future execution eligibility.

### 8.3 Rotation

Create a new signer-set version and migrate future execution to it.

Rotation should preserve:

- old signer identity history
- old intent and receipt references
- old policy applicability windows

## 9. Failure Cases

### 9.1 Intent-policy mismatch

- do not sign
- preserve a blocked intent record

### 9.2 Threshold not reached

- preserve collected signer attestations
- mark the intent as incomplete or expired

### 9.3 Paused signer pool

- produce explicit `skipped-paused` results
- do not silently ignore the pause state

### 9.4 Rotated signer set

- new intents MUST bind to the new signer-set version
- old intents keep the old signer-set reference

## 10. Minimal First-client Rules

For a first interoperable client, I recommend:

- show the active policy bundle for each fund
- show the active signer-set version
- show whether automatic signing is enabled, paused, or revoked
- preserve trigger -> intent -> attestation -> receipt links
- reject execution when accepted policy state is incomplete
- keep signer enrollment and signer-set history visible

## 11. Open Questions

- Should automatic signing remain fund-specific, or should a signer be reusable across multiple funds?
- Should committee derivation be fixed-set first, or should VRF-based committee selection be added early?
- Which app-layer record families should be mandatory to replicate across reader nodes versus signer nodes?
