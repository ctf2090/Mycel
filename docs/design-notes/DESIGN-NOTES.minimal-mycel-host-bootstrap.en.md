# Minimal Mycel Host Bootstrap

Status: design draft

This note describes the smallest practical local bootstrap that could host Mycel as a signed, on-demand runtime substrate.

The goal is not to define a full operating system distribution.

It is to define the minimum always-resident host that can:

- boot safely
- establish trust
- fetch missing signed modules
- verify them
- execute them inside a constrained runtime

Related notes:

- `DESIGN-NOTES.signed-on-demand-runtime-substrate.*` for the broader execution model
- `DESIGN-NOTES.dynamic-module-loading.*` for the narrower signed-module model
- `DESIGN-NOTES.app-signing-model.*` for artifact and execution trust layers

## 0. Goal

Define the smallest local resident host that still lets a Mycel-based system behave like a fetch-on-demand runtime substrate.

Keep resident:

- only the code needed to boot, verify, fetch, cache, sandbox, and execute

Allow to remain absent until needed:

- application logic
- renderers
- transformers
- higher-layer policy helpers
- most app-specific runtime behavior

## 1. Why a Local Bootstrap Is Unavoidable

A fully empty local machine cannot safely participate in a signed on-demand execution model.

Some trusted code must already exist locally in order to:

- start the machine
- bring up networking
- verify signatures and hashes
- enforce local execution policy
- host the execution sandbox

So the correct design target is not:

- no local runtime at all

It is:

- the smallest trustworthy local runtime that can safely admit remote artifacts

## 2. The Three-Layer Host Model

The minimal host can be understood as three layers.

### 2.1 Boot Layer

Responsibilities:

- firmware handoff
- bootloader execution
- root-of-trust initialization

This layer establishes:

- what local code is trusted first
- which public keys or trust anchors are built in

### 2.2 Resident Host Layer

Responsibilities:

- networking
- verification
- fetch and cache
- sandbox runtime hosting
- module policy enforcement

This is the core of the minimal Mycel host bootstrap.

### 2.3 On-Demand Module Layer

Responsibilities:

- application-specific behavior
- rendering
- transformation
- optional extension logic

This layer should be replaceable, fetchable, and mostly absent until needed.

## 3. Smallest Practical Resident Components

The resident host should probably include only six always-present components.

### 3.1 Boot and Trust Anchor

Needed for:

- secure startup
- built-in signer trust roots
- update-chain continuity

At minimum this layer should define:

- trusted public keys
- version or rollback policy
- local host identity if one exists

### 3.2 Tiny Host Core

Needed for:

- process or task isolation
- memory management
- local device and filesystem mediation
- network stack access

This does not need to be a full-featured desktop operating system.

It only needs to be capable enough to host the verifier, fetcher, cache, and runtime.

### 3.3 Verifier

Needed for:

- hash verification
- signature verification
- artifact metadata validation
- runtime-target compatibility checks

This must remain resident and trusted.

If the verifier itself becomes an on-demand module too early, the trust boundary collapses.

### 3.4 Fetcher

Needed for:

- resolving module identities
- downloading missing artifacts
- retry and mirror logic
- local artifact staging

The fetcher may be simple, but it must be reliable and policy-aware.

### 3.5 Cache Manager

Needed for:

- content-addressed blob storage
- offline reuse
- eviction of unused modules
- pinning of critical modules

This lets the system behave more like a verified execution cache than a traditional software installation.

### 3.6 Sandboxed Runtime

Needed for:

- loading verified portable modules
- enforcing resource limits
- exposing host capabilities through a narrow API

The recommended first runtime remains:

- `WASM`

## 4. Recommended First Runtime Shape

For a first implementation, the host should probably support:

- one portable module format
- one runtime engine
- one capability boundary

That means:

- avoid multiple execution formats at first
- avoid native binary plugins at first
- avoid direct unrestricted scripting environments at first

A single `WASM` runtime with strict host APIs is the cleanest starting point.

## 5. Suggested Boot Flow

The full host startup sequence could be:

1. firmware transfers control to the bootloader
2. bootloader verifies and loads the resident host image
3. resident host initializes trust anchors, networking, and local storage
4. host loads local pinned policy or bootstrap manifest
5. host determines which module identities are required
6. missing modules are fetched and staged
7. verifier checks signature, hash, runtime target, and policy
8. approved modules are cached and launched inside the sandbox runtime

This keeps execution admission explicit from the first moment a fetched artifact appears.

## 6. Minimal Capability Surface

The first host should expose very few capabilities.

Examples:

- `read_document`
- `read_view_state`
- `write_render_output`
- `write_local_cache`
- `request_network_fetch`
- `emit_diagnostics`

The first host should avoid exposing:

- arbitrary filesystem access
- arbitrary subprocess creation
- arbitrary outbound network access from inside modules
- direct native library loading

## 7. What the Host Should Not Do

The minimal bootstrap should not try to solve every systems problem at once.

It should not:

- replace a full general-purpose OS on day one
- execute unsigned native code
- allow partial code fragments to run before full verification
- assume network fetch implies execution approval
- mix verifier logic with untrusted fetched code

Its job is narrower:

- be the smallest safe admission and execution host

## 8. Storage Model

The local storage model should prefer:

- content-addressed blobs
- pinned critical modules
- evictable non-critical cached modules
- persistent audit logs

In this model, installation is no longer the main unit.

Instead, the main units are:

- trusted host image
- signed module metadata
- signed or hash-bound module blobs
- local cache entries

## 9. Offline and Recovery Behavior

A practical host should define three states clearly.

### 9.1 Online and Warm

The host can fetch missing modules and reuse cached ones.

### 9.2 Offline but Warm

The host cannot fetch new modules, but can still run anything already pinned or cached locally.

### 9.3 Offline and Cold

The host has no required module cached locally.

In this state, execution should fail safely rather than falling back to unsigned behavior.

## 10. Best First MVP

The smallest realistic MVP is probably not a new hardware-level OS image yet.

It is more likely:

1. a Linux-hosted `mycel-host` process
2. a built-in verifier
3. a built-in fetcher and content-addressed cache
4. a built-in `WASM` runtime
5. a narrow capability API

This gives most of the architecture benefit while avoiding the operational cost of building a full custom mini-OS too early.

## 11. Later Evolution Path

If the model proves useful, it can evolve in three steps.

### Step 1

Linux-hosted minimal Mycel runtime process

### Step 2

Dedicated appliance image with a smaller trusted host stack

### Step 3

More specialized mini-OS or unikernel-style deployment

This sequence reduces risk while preserving the long-term design direction.

## 12. Open Questions

- Which parts of the host image must be updateable, and which parts should be pinned?
- Should trust anchors be device-local, profile-local, or deployment-local?
- Which modules must be pinned for offline continuity?
- How much of the network stack belongs in the resident host versus an admitted system module?
- How small can the host core become before debugging and recovery become impractical?
