# Fund Auto-disbursement Profile v0.1

Status: profile draft

This profile defines a narrow interoperable model for automatic fund disbursement built on top of Mycel app-layer records, accepted-state selection, and policy-driven threshold custody.

This profile is intentionally conservative.

It constrains:

- how an accepted trigger becomes a disbursement candidate
- how policy checks are applied
- how automatic threshold signing may proceed
- which records must exist for audit and rebuild

It does not redefine the core protocol.

## 0. Scope

This profile assumes the underlying implementation already supports:

- the Mycel core protocol
- accepted-head selection
- app-layer records
- policy-driven threshold custody
- signer enrollment and consent tracking

This profile applies to:

- one `fund_id`
- one active signer-set version per execution intent
- one accepted policy bundle per execution path
- one concrete disbursement intent at a time

## 1. Profile Goals

The goals are:

1. make automatic disbursement predictable
2. keep the approval boundary explicit
3. preserve rebuildable governance history
4. keep the first client narrow enough to implement safely

## 2. Required Record Families

A conforming implementation must preserve at least these record families:

- `fund_manifest`
- `signer_enrollment`
- `signer_set`
- `policy_bundle`
- `consent_scope`
- `trigger_record`
- `execution_intent`
- `signer_attestation`
- `execution_receipt`
- `pause_or_revoke_record`

Optional records may exist, but they must not replace these minimum records.

## 3. Accepted Trigger Sources

This profile allows a disbursement path to begin only from an accepted trigger record.

Allowed trigger classes:

- `allocation-approved`
- `sensor-qualified`
- `pledge-matured`

A deployment may support fewer trigger classes, but not more in this profile version.

Each `trigger_record` must include:

- `trigger_id`
- `trigger_type`
- `trigger_ref`
- `fund_id`
- `policy_id`
- `amount_requested`
- `asset`
- `created_at`

## 4. Policy Constraints

Each disbursement attempt must bind to one accepted `policy_bundle`.

The active policy bundle must define:

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

A conforming implementation must reject execution if any required policy field is missing.

## 5. Execution Eligibility Rules

An execution intent is eligible only if all of the following are true:

1. the trigger record is accepted under the active profile
2. the trigger type is allowed by the active policy bundle
3. the requested amount does not exceed `max_amount_per_execution`
4. the requested amount does not push the fund over `max_amount_per_day`
5. the cooldown window has elapsed
6. the destination is inside the active allowlist
7. the active signer-set version matches the policy bundle
8. the signer set is not paused or revoked
9. the fund has sufficient available balance

If any rule fails, the system must create a blocked or rejected execution outcome, not silently continue.

## 6. Execution Intent

Each eligible disbursement path must produce one `execution_intent`.

Required fields:

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

Allowed `status` values in this profile:

- `pending`
- `eligible`
- `blocked`
- `signed`
- `broadcast`
- `failed`

The `intent_hash` must be stable for the exact outputs and amount being signed.

## 7. Automatic Threshold Signing

This profile allows automatic signing only under these rules:

1. all participating signers must have active enrollment
2. all participating signers must have a valid consent scope
3. each signer runtime must verify the same `intent_hash`
4. each signer runtime must verify the same `policy_id`
5. each signer runtime must bind its result to the same `signer_set_id` and version

A conforming signer runtime must never sign:

- when enrollment is missing
- when consent scope is missing or expired
- when state is paused or revoked
- when policy fields are incomplete
- when local runtime state is out of sync with accepted state

## 8. Signer Attestations

Each signer-side result must be preserved as one `signer_attestation`.

Required fields:

- `attestation_id`
- `intent_id`
- `signer_id`
- `signer_set_version`
- `intent_hash`
- `outcome`
- `created_at`

Allowed `outcome` values in this profile:

- `signed`
- `rejected`
- `skipped-paused`
- `skipped-revoked`
- `skipped-policy-mismatch`
- `skipped-insufficient-sync`

The implementation must preserve both successful and unsuccessful outcomes.

## 9. Threshold Rule

This profile assumes one fixed threshold per active signer-set version.

Required rule:

- `required_signatures = threshold(signer_set_id, version)`

The execution layer may broadcast only after collecting at least `required_signatures` valid results for the same `intent_hash`.

## 10. Receipt Requirements

Each broadcast or settlement attempt must produce one `execution_receipt`.

Required fields:

- `receipt_id`
- `intent_id`
- `executor`
- `settlement_ref`
- `status`
- `submitted_at`
- `confirmed_at` or `null`
- `error_summary`

Allowed `status` values in this profile:

- `submitted`
- `confirmed`
- `failed`
- `rejected-by-rail`

The receipt must be linkable back to:

- one `execution_intent`
- one `trigger_record`
- one `policy_bundle`
- one signer-set version

## 11. Pause, Revoke, and Rotation

This profile requires support for:

- signer pause
- signer revoke
- signer-set rotation
- policy pause

Required behavior:

- new execution intents must bind only to the current active signer-set version
- old intents keep the old signer-set reference
- pause or revoke must block future signing, not rewrite old history

## 12. Minimal Flow

The minimal conforming flow is:

1. accepted trigger record appears
2. implementation checks active policy bundle
3. implementation checks balance and rate limits
4. implementation creates `execution_intent`
5. signer runtimes verify eligibility and emit `signer_attestation`
6. execution layer reaches threshold and broadcasts
7. implementation writes `execution_receipt`

## 13. Non-goals

This profile does not define:

- raw payment processor integration
- raw sensor interpretation
- oracle trust models
- cross-fund aggregation
- dynamic weighted signer math
- committee derivation beyond one active signer set

## 14. Minimal First-client Requirements

For a first interoperable client, I recommend:

- one active `fund_id`
- one active signer-set version at a time
- one active policy bundle at a time
- no dynamic committee derivation
- no parallel partial-intent merging
- explicit blocked-intent and failed-receipt views

## 15. Open Questions

- Should a later version allow multiple active policy bundles per fund?
- Should a later version allow weighted rather than fixed-threshold signer math?
- Should `allocation-approved` and `sensor-qualified` remain the same profile, or be split into separate narrower profiles?
