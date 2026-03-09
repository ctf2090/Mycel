# Mycel

Language: English | [繁體中文](./README.zh-TW.md)

Mycel is a neutral, technical protocol stack for verifiable text history, governed reading state, and decentralized replication.

## Overview

Mycel is designed for text-first collaboration and reference-text systems in distributed environments:

- Verifiable change history
- P2P replication
- Digital-signature validation
- Multi-branch coexistence without requiring global single consensus
- Profile-governed accepted reading and state selection
- Extensible app-layer models built on a stable protocol core

## Neutrality

Mycel can be used in many content domains. The protocol itself remains neutral and purely technical.

## Current Status

- Protocol stage: `v0.1` conceptual specification with a growing profile and design-note layer
- Current focus: first-client scoping, implementation readiness, and narrowing the path from design notes to concrete profiles
- Current Rust CLI status: suitable for internal validation and deterministic simulator-harness workflows, but not yet a production Mycel client or node

## Documentation

### Specs

- [PROTOCOL.en.md](./PROTOCOL.en.md): Full protocol specification
- [WIRE-PROTOCOL.en.md](./WIRE-PROTOCOL.en.md): Wire protocol draft
- [IMPLEMENTATION-CHECKLIST.en.md](./IMPLEMENTATION-CHECKLIST.en.md): Implementation checklist for a minimal v0.1 client
- [PROFILE.fund-auto-disbursement-v0.1.en.md](./PROFILE.fund-auto-disbursement-v0.1.en.md): Narrow v0.1 profile draft for automatic fund disbursement with m-of-n custody
- [PROFILE.mycel-over-tor-v0.1.en.md](./PROFILE.mycel-over-tor-v0.1.en.md): Narrow v0.1 deployment profile for Tor-routed Mycel transport

### Design Notes

- [docs/design-notes/DESIGN-NOTES.client-non-discretionary-multi-view.en.md](./docs/design-notes/DESIGN-NOTES.client-non-discretionary-multi-view.en.md): Design draft for a protocol-governed multi-view reader model
- [docs/design-notes/DESIGN-NOTES.two-maintainer-role.en.md](./docs/design-notes/DESIGN-NOTES.two-maintainer-role.en.md): Design draft for splitting editor and view maintainer authority
- [docs/design-notes/DESIGN-NOTES.maintainer-conflict-flow.en.md](./docs/design-notes/DESIGN-NOTES.maintainer-conflict-flow.en.md): Design draft for preserving and governing maintainer conflicts over rival document ideas
- [docs/design-notes/DESIGN-NOTES.mycel-app-layer.en.md](./docs/design-notes/DESIGN-NOTES.mycel-app-layer.en.md): Design draft for separating client, runtime, and effect responsibilities
- [docs/design-notes/DESIGN-NOTES.qa-app-layer.en.md](./docs/design-notes/DESIGN-NOTES.qa-app-layer.en.md): Design draft for a Q&A app carried by Mycel
- [docs/design-notes/DESIGN-NOTES.qa-minimal-schema.en.md](./docs/design-notes/DESIGN-NOTES.qa-minimal-schema.en.md): Minimal schema draft for a Q&A app
- [docs/design-notes/DESIGN-NOTES.commentary-citation-schema.en.md](./docs/design-notes/DESIGN-NOTES.commentary-citation-schema.en.md): Design draft for maintainer-authored commentary works that heavily cite source documents
- [docs/design-notes/DESIGN-NOTES.app-signing-model.en.md](./docs/design-notes/DESIGN-NOTES.app-signing-model.en.md): Design draft for separating object signing, release signing, and execution-evidence signing
- [docs/design-notes/DESIGN-NOTES.signature-priority.en.md](./docs/design-notes/DESIGN-NOTES.signature-priority.en.md): Design draft prioritizing which Mycel-carried objects should require signatures first
- [docs/design-notes/DESIGN-NOTES.signature-role-matrix.en.md](./docs/design-notes/DESIGN-NOTES.signature-role-matrix.en.md): Design draft mapping current object families to their default signing roles
- [docs/design-notes/DESIGN-NOTES.donation-app-layer.en.md](./docs/design-notes/DESIGN-NOTES.donation-app-layer.en.md): Design draft for a donation-oriented app carried by Mycel
- [docs/design-notes/DESIGN-NOTES.canonical-text-profile.en.md](./docs/design-notes/DESIGN-NOTES.canonical-text-profile.en.md): Design draft for a neutral canonical-text profile with witnesses, anchors, commentary, and accepted readings
- [docs/design-notes/DESIGN-NOTES.interpretation-dispute-model.en.md](./docs/design-notes/DESIGN-NOTES.interpretation-dispute-model.en.md): Design draft for preserving and governing rival interpretations without overwriting root text
- [docs/design-notes/DESIGN-NOTES.auto-signer-consent-model.en.md](./docs/design-notes/DESIGN-NOTES.auto-signer-consent-model.en.md): Design draft for enrollment and consent boundaries in automatic signer systems
- [docs/design-notes/DESIGN-NOTES.blind-address-threat-model.en.md](./docs/design-notes/DESIGN-NOTES.blind-address-threat-model.en.md): Threat model draft for blind-address custody designs
- [docs/design-notes/DESIGN-NOTES.signer-availability-emergency-response.en.md](./docs/design-notes/DESIGN-NOTES.signer-availability-emergency-response.en.md): Design draft for warning, critical, and emergency response to signer-availability decline
- [docs/design-notes/DESIGN-NOTES.signer-activity-model.en.md](./docs/design-notes/DESIGN-NOTES.signer-activity-model.en.md): Design draft for evaluating signer readiness and effective signer capacity
- [docs/design-notes/DESIGN-NOTES.policy-driven-threshold-custody.en.md](./docs/design-notes/DESIGN-NOTES.policy-driven-threshold-custody.en.md): Design draft for policy-authorized automatic m-of-n custody
- [docs/design-notes/DESIGN-NOTES.mycel-anonymity-model.en.md](./docs/design-notes/DESIGN-NOTES.mycel-anonymity-model.en.md): Design draft for analyzing anonymity across transport, metadata, client, runtime, and replication layers
- [docs/design-notes/DESIGN-NOTES.first-client-scope-v0.1.en.md](./docs/design-notes/DESIGN-NOTES.first-client-scope-v0.1.en.md): Design draft for the narrow first-client target and what it must deliberately defer
- [docs/design-notes/DESIGN-NOTES.mycel-full-stack-map.en.md](./docs/design-notes/DESIGN-NOTES.mycel-full-stack-map.en.md): Design draft mapping the current Mycel documents into one layered full-stack system view
- [docs/design-notes/DESIGN-NOTES.mycel-protocol-upgrade-philosophy.en.md](./docs/design-notes/DESIGN-NOTES.mycel-protocol-upgrade-philosophy.en.md): Design draft for keeping protocol core changes conservative while moving faster in profiles and design notes
- [docs/design-notes/DESIGN-NOTES.peer-discovery-model.en.md](./docs/design-notes/DESIGN-NOTES.peer-discovery-model.en.md): Design draft for bounded peer discovery across public, restricted, and Tor-oriented deployments
- [docs/design-notes/DESIGN-NOTES.peer-simulator-v0.en.md](./docs/design-notes/DESIGN-NOTES.peer-simulator-v0.en.md): Design draft for an early multi-peer simulator and test harness
- [docs/design-notes/DESIGN-NOTES.sensor-triggered-donation.en.md](./docs/design-notes/DESIGN-NOTES.sensor-triggered-donation.en.md): Design draft for a sensor-triggered donation flow
- [docs/design-notes/DESIGN-NOTES.governance-history-security.en.md](./docs/design-notes/DESIGN-NOTES.governance-history-security.en.md): Design draft for securing governance history

### Meta

- [PROJECT-INTENT.md](./PROJECT-INTENT.md): Project intent and protocol-boundary notes
- [AGENTS.md](./AGENTS.md): Repository collaboration rules

### Implementation Scaffold

- [fixtures/README.md](./fixtures/README.md): Language-neutral fixture sets for simulator and verification testing
- [sim/README.md](./sim/README.md): Language-neutral scaffold for peer simulator structure, topologies, tests, and reports
- [sim/SCHEMA-CROSS-CHECK.md](./sim/SCHEMA-CROSS-CHECK.md): Cross-check rules for how simulator schemas and IDs should line up
- [RUST-WORKSPACE.md](./RUST-WORKSPACE.md): Initial Rust workspace layout for the core, simulator library, and CLI

### CI

- [.github/workflows/ci.yml](./.github/workflows/ci.yml): GitHub Actions workflow for Rust checks and negative validation smoke coverage

## Near-Term Priorities

1. Build a narrow first client around sync, verification, accepted-head selection, and reader-first text rendering
2. Keep turning mature design areas into explicit profiles or schemas instead of expanding the protocol core too quickly
3. Expand upward one layer at a time: canonical-text reading first, then selective app-layer support

## Project Scope

Mycel is not:

- a blockchain with global mandatory consensus
- a plain file transfer system
- a Git clone

Mycel is a protocol stack for verifiable, evolvable, decentralized text history and governed text-bearing systems.
