# Mycel App-signing Model

Status: design draft

This note describes how a Mycel-based system should think about application signing as a layered model rather than one single signature decision.

The main design principle is:

- application state signing is not the same as release signing
- release signing is not the same as execution-evidence signing
- each signing layer protects a different class of trust
- a secure deployment should not assume one layer automatically replaces the others

## 0. Goal

Enable a Mycel-based application system to distinguish at least three separate signing needs:

- signing app-layer objects and governance state
- signing released software artifacts
- signing execution receipts or runtime attestations

This note does not define one mandatory signing toolchain.

It defines the trust boundaries that the signing model should preserve.

## 1. Why One Signature Is Not Enough

If a system says only "the app is signed," that statement is ambiguous.

It may mean:

- the app manifest is signed
- the downloadable binary is signed
- the runtime receipt is signed

Those protect different risks.

A deployment should therefore model them separately.

## 2. Layer 1: App-layer Object Signing

This layer protects Mycel-carried application records.

Typical signed objects:

- `app_manifest`
- app governance records
- policy objects
- proposal and approval records
- effect requests
- effect receipts when represented as Mycel objects

Primary purpose:

- prove authorship
- protect record integrity
- preserve governance and state history

This layer belongs inside the Mycel object and profile model.

It should align with:

- canonical serialization
- object-level signature verification
- accepted-state derivation rules

This layer does not by itself prove that the downloaded software artifact is authentic.

## 3. Layer 2: Release Artifact Signing

This layer protects distributed software artifacts.

Typical signed artifacts:

- CLI binaries
- application packages
- installers
- container images
- release manifests

Primary purpose:

- protect the software supply chain
- prove artifact origin
- detect substituted or tampered releases

This layer is important even if all Mycel objects are signed.

Reason:

- a user may still download a malicious client that verifies Mycel objects incorrectly or leaks secrets locally

This layer belongs in the build, release, and distribution pipeline rather than in the protocol core.

## 4. Layer 3: Execution-evidence Signing

This layer protects evidence about what the runtime actually did.

Typical signed evidence:

- execution receipts
- settlement receipts
- runtime attestations
- external effect confirmations

Primary purpose:

- prove which runtime or executor performed an action
- preserve post-event auditability
- distinguish intended action from completed side effect

This layer is especially important for:

- payment execution
- custody systems
- external effect systems
- disputes and incident review

This layer should not be confused with release signing.

The runtime may be authentic, but a particular execution receipt still needs its own verifiable authorship.

## 5. Core vs App vs Runtime Boundary

The protocol core should provide general signature-verification capability.

The app and profile layer should define:

- which Mycel objects require signatures
- whose signatures are valid for each object family
- how signed records participate in accepted-state selection

The runtime and release pipeline should define:

- how software artifacts are signed
- how effect receipts are signed
- how runtime identities are managed

This keeps protocol core stable while allowing higher-layer signing models to evolve.

## 6. Common Failure Cases

### 6.1 Object Signing Without Release Signing

All governance records are signed, but users download unsigned binaries.

Result:

- governance history may be valid
- client supply chain may still be compromised

### 6.2 Release Signing Without Object Signing

The shipped binary is authentic, but governance and app-state objects have weak authorship rules.

Result:

- software origin is protected
- app-layer authority and history remain weak

### 6.3 Authentic Runtime Without Signed Receipts

The runtime is trusted, but execution evidence is not signed or attributable.

Result:

- post-event audit and dispute handling become weak

### 6.4 Treating One Layer as a Substitute for All Layers

A system assumes one signature class solves every trust problem.

Result:

- hidden gaps remain in supply chain, governance history, or effect auditability

## 7. Recommended Baseline

A practical deployment should usually provide:

1. signed app-layer governance and state objects
2. signed release artifacts
3. signed execution receipts for high-risk runtimes

Minimal deployments may begin with the first layer.

Security-sensitive deployments should not stop there.

## 8. Practical Rule

The right question is not:

- "Is the app signed?"

It is:

- "Which part of the system is signed, by whom, and what trust boundary does that signature protect?"

If a deployment cannot answer that clearly, its signing model is probably underspecified.
