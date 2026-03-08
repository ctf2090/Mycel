# Mycel Full-stack Map

Status: design draft

This note maps the current Mycel document set into a full-stack system view.

The main design principle is:

- Mycel should be understood as a layered system, not a single document format
- the protocol core is only one part of the eventual stack
- profiles, app-layer models, governance, and deployment each add distinct system responsibilities
- implementation planning should follow these layers rather than treating all documents as one undifferentiated scope

## 0. Goal

Give the repo a system-level map that answers:

- what major layers Mycel currently implies
- which documents belong to which layer
- what responsibilities each layer carries
- what kind of project Mycel becomes if all current layers are implemented

## 1. Stack Overview

The current Mycel document set implies at least seven major layers:

1. protocol core
2. object verification and local state
3. synchronization and transport
4. governance and accepted-state selection
5. canonical text and application models
6. fund, custody, and execution systems
7. deployment, privacy, and operational layers

These layers are separable in analysis, but they interact heavily in implementation.

## 2. Protocol Core

This is the narrowest and most interoperable layer.

Responsibilities:

- define core object families
- define canonical serialization
- define hashing and derived IDs
- define signature expectations
- define revision replay
- define wire message structure

Primary documents:

- `PROTOCOL.*`
- `WIRE-PROTOCOL.*`

If this layer is unstable, the rest of the stack cannot interoperate reliably.

## 3. Object Verification and Local State

This layer turns canonical objects into a working local node state.

Responsibilities:

- object parsing
- derived-ID verification
- signature verification
- revision replay
- state reconstruction
- index rebuild
- local object store management

Primary documents:

- `IMPLEMENTATION-CHECKLIST.*`
- parts of `PROTOCOL.*`

This is the first concrete implementation layer for a real client or node.

## 4. Synchronization and Transport

This layer lets peers exchange objects and current state.

Responsibilities:

- session setup
- manifest and heads exchange
- object fetch and validation
- transport constraints
- bounded peer communication

Primary documents:

- `WIRE-PROTOCOL.*`
- `PROFILE.mycel-over-tor-v0.1.*`
- `DESIGN-NOTES.peer-discovery-model.*`

This is where Mycel stops being only a text model and becomes an actual networked system.

## 5. Governance and Accepted-state Selection

This layer determines which state a conforming reader or deployment should treat as active.

Responsibilities:

- store governance signals
- define accepted-head or accepted-reading selection
- separate publication roles
- preserve governance history
- keep decision traces auditable

Primary documents:

- `PROTOCOL.*`
- `DESIGN-NOTES.client-non-discretionary-multi-view.*`
- `DESIGN-NOTES.two-maintainer-role.*`
- `DESIGN-NOTES.governance-history-security.*`

This is the layer that distinguishes Mycel from a plain replicated document system.

## 6. Canonical Text Layer

This layer models long-lived reference corpora.

Responsibilities:

- stable citation anchors
- witness handling
- accepted reading profiles
- commentary separation
- alignment between witnesses

Primary documents:

- `DESIGN-NOTES.canonical-text-profile.*`

This layer is especially important if Mycel is used for deeply referenced texts rather than only short-form collaboration.

## 7. General App Layer

This layer allows Mycel to carry application definitions without making the protocol core itself execute side effects.

Responsibilities:

- app manifest modeling
- app state modeling
- intent modeling
- effect request and receipt modeling
- runtime separation

Primary documents:

- `DESIGN-NOTES.mycel-app-layer.*`
- `DESIGN-NOTES.qa-app-layer.*`
- `DESIGN-NOTES.qa-minimal-schema.*`
- `DESIGN-NOTES.donation-app-layer.*`

This is where Mycel starts looking like a platform rather than only a document protocol.

## 8. Fund, Custody, and Execution Layer

This layer handles governed economic flows and delegated execution.

Responsibilities:

- donation modeling
- fund disbursement policy
- execution-intent generation
- signer enrollment and consent boundaries
- threshold custody
- execution receipts

Primary documents:

- `PROFILE.fund-auto-disbursement-v0.1.*`
- `DESIGN-NOTES.policy-driven-threshold-custody.*`
- `DESIGN-NOTES.auto-signer-consent-model.*`
- `DESIGN-NOTES.sensor-triggered-donation.*`

This is one of the most operationally sensitive parts of the full stack.

## 9. Anonymity and Privacy Layer

This layer handles identity leakage and deployment privacy posture.

Responsibilities:

- transport-anonymity posture
- metadata minimization
- role separation
- local hardening
- runtime hardening
- deployment-tier boundaries

Primary documents:

- `DESIGN-NOTES.mycel-anonymity-model.*`
- `PROFILE.mycel-over-tor-v0.1.*`

This layer matters because Mycel's verifiable history can otherwise become highly linkable over time.

## 10. Client Surface Layer

This layer is what users actually interact with.

Responsibilities:

- accepted-text reading
- history inspection
- branch visibility
- source and citation browsing
- Q&A and commentary navigation
- sync state display
- policy/profile visibility

Primary documents:

- `IMPLEMENTATION-CHECKLIST.*`
- reader-oriented parts of the various design notes

This layer is not yet fully formalized in one UI note, but the current docs imply a reader-first client model.

## 11. Meta and Direction Layer

This layer does not define interoperability.

Instead, it defines:

- project direction
- document boundaries
- upgrade philosophy
- repository working rules

Primary documents:

- `PROJECT-INTENT.*`
- `DESIGN-NOTES.mycel-protocol-upgrade-philosophy.*`
- `AGENTS.md`

This layer is important because it prevents project-intent language from leaking into the technical layers.

## 12. Dependency Shape

The rough dependency direction is:

`protocol core`
-> `verification and local state`
-> `sync and transport`
-> `governance and accepted-state selection`
-> `canonical text and app-layer models`
-> `fund / custody / execution`
-> `client and deployment behavior`

Not every deployment needs every upper layer.

For example:

- a minimal reader might stop at accepted text and sync
- a richer deployment may add canonical text, Q&A, and commentary
- a highly ambitious deployment may add fund automation and threshold custody

## 13. Three Realistic Build Shapes

### 13.1 Minimal Mycel

Includes:

- protocol core
- local object store
- revision replay
- wire sync
- accepted-head rendering

This is a real protocol client, but not yet a broad platform.

### 13.2 Reader-plus-governance Mycel

Includes:

- minimal Mycel
- canonical text handling
- citations
- Q&A
- commentary
- accepted-reading profiles

This is a serious knowledge or reference-text system.

### 13.3 Full-stack Mycel

Includes:

- reader-plus-governance Mycel
- app layer
- donation and fund systems
- threshold custody
- automatic execution paths
- anonymity-aware deployment profiles

This is no longer a narrow protocol project.
It becomes a full distributed text, governance, and application ecosystem.

## 14. What This Means for Planning

The full document set should not be mistaken for one immediate implementation target.

Instead, planning should choose:

- which layers are in scope now
- which layers remain profile-only
- which layers stay in design-note form

The main practical conclusion is:

- Mycel already describes a large eventual system
- the first implementation should still be deliberately narrow

## 15. Recommended Next Step

After this map, the next planning step should be to classify work into:

- `minimal`
- `reader-plus-governance`
- `full-stack`

This would turn the conceptual stack into a build roadmap.
