# Mycel as a Signed On-Demand Runtime Substrate

Status: design draft

This note explores a more ambitious interpretation of Mycel: not only as a protocol for verifiable text and governance, but as a substrate where most application logic can be fetched, verified, and executed on demand.

The core idea is:

- keep a very small trusted local runtime
- model executable parts as signed, content-addressed modules
- fetch missing modules only when needed
- allow unused modules to be absent from local storage

This is not a proposal to turn Mycel into a monolithic operating system kernel.

It is a proposal to treat Mycel as a signed, distributed runtime substrate for higher-layer application behavior.

Related notes:

- `DESIGN-NOTES.dynamic-module-loading.*` for the narrower module-loading model
- `DESIGN-NOTES.mycel-app-layer.*` for the split between app state and side-effect execution
- `DESIGN-NOTES.app-signing-model.*` for the distinction between object signing, artifact signing, and runtime evidence

## 0. Goal

Enable a Mycel-based system where:

- executable functionality may be absent locally until needed
- missing functionality can be fetched in real time
- every fetched executable artifact is signed and content-addressed
- the local host keeps only the minimum trusted runtime needed to verify and run those artifacts

Preserve:

- explicit trust boundaries
- auditability of what code ran
- deterministic identity for code artifacts
- local ability to refuse execution even when an artifact is valid

## 1. What This Is and Is Not

This model is closest to:

- a signed application substrate
- a distributed runtime environment
- a content-addressed module host

It is not necessarily:

- a full replacement for a hardware-facing OS kernel
- a claim that literally all local code can disappear
- a promise that execution becomes network-transparent in every case

Some local bootstrap must remain.

## 2. The Small Trusted Local Base

A fully empty local machine cannot execute fetched modules safely, because some part of the system must already be present to:

- boot the device
- establish network access
- verify signatures and hashes
- enforce local policy
- provide sandboxing
- host the execution runtime

So the practical model is:

- a small trusted local base stays resident
- higher-layer logic becomes fetchable and replaceable

The smaller this trusted base is, the closer the system gets to a true on-demand runtime substrate.

## 3. Execution Model

The system should separate three classes of code.

### 3.1 Resident Base

Always local.

Responsibilities:

- bootstrapping
- verification
- module cache management
- policy enforcement
- sandbox runtime hosting

### 3.2 On-Demand Modules

Fetched when needed.

Examples:

- renderers
- transformers
- app-specific logic
- policy helpers
- protocol-adjacent extension logic

### 3.3 Optional Cached Artifacts

Kept locally only for speed or offline continuity.

These should be deletable without changing module identity, because identity comes from content hash and signature rather than from local installation state.

## 4. Why Sign Everything Fetchable

If the system fetches executable artifacts at runtime, signature checks are not optional metadata.

They are part of the execution boundary.

Every fetchable executable part should therefore have:

- a stable artifact identity
- a content hash
- a valid signature
- a declared runtime target
- a declared capability request

This reduces the risk that runtime fetch becomes an arbitrary remote-code-execution channel.

## 5. Content Addressing and Absence by Default

The desired storage rule is:

- code that is not currently needed does not need to exist locally

This implies:

- modules are referenced by identity, not by installation path alone
- local absence is normal, not an error state
- fetch is a standard resolution step

The host should be able to say:

- "this module is required"
- "it is not present locally"
- "fetch and verify it now"

without treating that workflow as exceptional.

## 6. Runtime Fetch Flow

Suggested flow:

1. accepted app state or a local action requires a module
2. the local runtime resolves the required module identity
3. if the module is not cached locally, the runtime fetches it from approved sources
4. the runtime verifies signature, hash, runtime target, and local policy
5. the runtime loads it inside a sandbox
6. execution metadata is recorded for later audit

If verification fails, the artifact should remain non-executable even if it is syntactically valid.

## 7. Why This Resembles a Distributed Operating Model

This model begins to look like a distributed operating model because:

- executable behavior is not assumed to be permanently installed
- the host resolves code over a network or content graph
- execution depends on remote artifact availability
- local storage acts more like a verified cache than a full install image

But it still differs from a classical distributed OS in one important way:

- trust and artifact verification are first-class parts of the design rather than secondary packaging concerns

## 8. Recommended Artifact Model

This substrate should not fetch raw native code fragments by default.

Prefer a structured model:

- signed module metadata object
- signed or hash-bound module blob
- explicit runtime target
- explicit capability request

The safest first version is:

- `WASM` modules
- content-addressed blobs
- host-mediated capability APIs

That keeps the substrate portable and easier to audit.

## 9. Capability and Policy Boundary

A valid signature should mean:

- "this is the artifact that the signer intended"

It should not automatically mean:

- "the local machine must run it"

The host still needs local policy checks:

- is this signer trusted?
- is this module family allowed?
- are these requested capabilities acceptable?
- is execution allowed in the current local state?

This preserves local sovereignty over execution.

## 10. Cache Instead of Installation

Under this model, local storage behaves more like a verified execution cache than a traditional software installation.

Recommended properties:

- cache entries keyed by content hash
- safe eviction of unused modules
- exact-version reuse when a module is needed again
- offline execution possible only for artifacts already cached locally

This means the system can be mostly stateless without becoming permanently network-dependent for every repeat execution.

## 11. Where Determinism Still Matters

Not every fetched module has the same semantic weight.

Three broad classes matter:

### 11.1 Pure Presentation Modules

Example:

- renderers

Determinism is useful but less critical to accepted-state derivation.

### 11.2 State-Interpreting Modules

Example:

- policy helpers
- transformation logic

Determinism matters much more, because different outputs may change application behavior.

### 11.3 Side-Effecting Modules

Example:

- modules that trigger HTTP calls or local actions through host capabilities

These require the strongest audit and policy controls.

The host should distinguish these categories rather than treating all modules as equivalent.

## 12. Why Native Code Should Not Be the Default

If the substrate fetches native binaries or dynamic libraries by default, it inherits:

- platform-specific packaging problems
- harder sandboxing
- broader privilege surfaces
- weaker portability

That pushes the design toward a risky remote-install model instead of a conservative signed runtime substrate.

For that reason, the default direction should remain:

- a small local host
- a sandbox runtime
- signed portable modules

## 13. Suggested First Practical Form

A practical first form of this idea would be:

1. a local Mycel host runtime
2. signed `WASM` modules loaded on demand
3. content-addressed module blobs
4. local capability policy
5. execution audit logs
6. optional local cache eviction when modules are not in use

This is enough to approximate a signed, fetch-on-demand runtime substrate without pretending that Mycel already replaces the full operating system stack.

## 14. Open Questions

- How small can the trusted local base become before the system becomes impractical?
- Should module signer policy be local-only, profile-bound, or governance-assisted?
- Which module classes, if any, may influence accepted-state derivation?
- Should offline mode require explicit pinning of critical modules?
- When should a fetched module be cached, and when should it be discarded immediately after execution?
