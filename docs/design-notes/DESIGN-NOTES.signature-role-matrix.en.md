# Mycel Signature Role Matrix

Status: design draft

This note maps the main object families in the current repo to the roles that should sign them.

The main design principle is:

- the signer of an object should match the authority the object claims to express
- one role should not silently inherit another role's signing power
- runtime authorship, governance authorship, and signer consent should remain distinguishable
- a practical first implementation should define a narrow default matrix before adding exceptions

Related notes:

- `DESIGN-NOTES.signature-priority.*` for which objects should require signatures first
- `DESIGN-NOTES.app-signing-model.*` for the three-layer signing model
- `DESIGN-NOTES.policy-driven-threshold-custody.*` for custody-specific object families
- `DESIGN-NOTES.auto-signer-consent-model.*` for signer consent boundaries

## 0. Goal

Define a first-pass answer to:

- who signs what
- which roles may co-sign
- which roles should not sign a given object family by default

This note focuses on Mycel-carried objects, not release artifacts.

## 1. Roles

This note uses the following role labels:

- `app-author`
- `app-maintainer`
- `governance-maintainer`
- `editor-maintainer`
- `view-maintainer`
- `signer`
- `signer-runtime`
- `runtime`
- `executor`
- `operator`

Not every app needs every role.

Some deployments may collapse multiple roles into one key, but that should be an explicit profile choice rather than an implicit assumption.

## 2. Matrix

### 2.1 `app_manifest`

- Primary signer: `app-author` or `app-maintainer`
- May co-sign: `governance-maintainer`
- Should not sign by default: `runtime`, `executor`, `operator`

Reason:

- this object defines app identity and scope, not runtime execution

### 2.2 Governance proposal

- Primary signer: `governance-maintainer`
- App-specific variants may use: `editor-maintainer` or `view-maintainer`, depending on profile
- Should not sign by default: `runtime`, `executor`

Reason:

- proposals express governance intent, not side-effect execution

### 2.3 Governance approval or selector signal

- Primary signer: `governance-maintainer` or `view-maintainer`
- May co-sign: other governance-authorized roles defined by profile
- Should not sign by default: `runtime`, `executor`, `operator`

Reason:

- selector authority should remain distinct from runtime behavior

### 2.4 `signer_enrollment`

- Primary signer: `signer`
- May co-sign: `governance-maintainer` or `operator` as confirmation
- Should not sign by default: unrelated `runtime`, `executor`

Reason:

- enrollment must prove that the signer knowingly joined

### 2.5 `consent_scope`

- Primary signer: `signer`
- May co-sign: `governance-maintainer` when the profile requires explicit acknowledgement
- Should not sign by default: `runtime`, `executor`

Reason:

- consent must come from the signer whose key or share is being bound

### 2.6 `signer_set`

- Primary signer: `governance-maintainer`
- May co-sign: authorized custody-governance role
- Should not sign by default: `signer-runtime`, `runtime`, `executor`

Reason:

- signer-set membership is a governance fact, not a runtime observation

### 2.7 `policy_bundle`

- Primary signer: `governance-maintainer`
- May co-sign: policy-authorizing role defined by profile
- Should not sign by default: `runtime`, `executor`

Reason:

- policy authorization should not be silently delegated to executors

### 2.8 `pause_or_revoke_record`

- Primary signer: `governance-maintainer`
- May also be signed by: `signer`, if the profile allows signer-local emergency control
- Should not sign by default: `runtime`, `executor`, `operator`

Reason:

- this object changes future execution eligibility and must not be a local runtime override

### 2.9 `trigger_record`

- Primary signer: the role that owns the trigger source
- Common cases:
  - `governance-maintainer` for governance-approved triggers
  - `runtime` for trusted runtime-derived triggers
- Should not sign by default: unrelated `executor`

Reason:

- a trigger must be attributable to the system component that observed or authorized the triggering condition

### 2.10 `execution_intent`

- Primary signer: authorized `runtime` or governance-derived execution authority
- May co-sign: `executor`, if the profile wants explicit executor acknowledgement before settlement
- Should not sign by default: general `operator`

Reason:

- the intent binds the actionable execution context and should come from an authority allowed to derive it from accepted state

### 2.11 `signer_attestation`

- Primary signer: `signer-runtime` or `signer`
- Should not sign by default: `runtime`, `executor`, `operator`

Reason:

- an attestation is the signer-side claim that checks passed and a signing result was produced

### 2.12 `execution_receipt`

- Primary signer: `executor` or execution `runtime`
- May co-sign: settlement-observer runtime if the profile separates execution from observation
- Should not sign by default: `governance-maintainer`, `signer`

Reason:

- the receipt should prove what actually happened in the execution layer

### 2.13 Generic `effect_receipt`

- Primary signer: `runtime`
- May co-sign: specialized executor when the runtime delegates to one
- Should not sign by default: governance roles

Reason:

- effect receipts are runtime evidence, not governance decisions

## 3. Default Separation Rules

The first-pass matrix should follow these rules:

1. governance roles sign governance records
2. signers sign enrollment and consent
3. signer runtimes sign signer attestations
4. runtimes and executors sign effect and settlement receipts
5. operators should not become default authority signers just because they run infrastructure

## 4. Dangerous Collapses

The following role collapses are possible, but risky unless explicitly profiled:

- `governance-maintainer` + `executor`
- `signer` + `governance-maintainer`
- `runtime` + `selector authority`
- `operator` + every signing role

These collapses increase convenience, but they weaken attribution clarity and can hide power concentration.

## 5. Minimal First-pass Matrix for the Current Repo

If the repo wants one narrow default now, the most defensible initial matrix is:

- `app_manifest` -> `app-author` or `app-maintainer`
- governance proposal / approval -> `governance-maintainer`
- `signer_enrollment` -> `signer`
- `consent_scope` -> `signer`
- `signer_set` -> `governance-maintainer`
- `policy_bundle` -> `governance-maintainer`
- `trigger_record` -> trigger-owning authority
- `execution_intent` -> authorized `runtime`
- `signer_attestation` -> `signer-runtime`
- `execution_receipt` -> `executor` or execution `runtime`

## 6. Practical Rule

For any object family, ask:

- whose authority does this object claim to express?

That role should sign first.

If a different role signs by default, the system should explain why that substitution is safe and visible.
