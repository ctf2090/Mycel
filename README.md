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

## Documentation

### Specs

- [PROTOCOL.en.md](./PROTOCOL.en.md): Full protocol specification
- [WIRE-PROTOCOL.en.md](./WIRE-PROTOCOL.en.md): Wire protocol draft
- [IMPLEMENTATION-CHECKLIST.en.md](./IMPLEMENTATION-CHECKLIST.en.md): Implementation checklist for a minimal v0.1 client
- [PROFILE.fund-auto-disbursement-v0.1.en.md](./PROFILE.fund-auto-disbursement-v0.1.en.md): Narrow v0.1 profile draft for automatic fund disbursement
- [PROFILE.mycel-over-tor-v0.1.en.md](./PROFILE.mycel-over-tor-v0.1.en.md): Narrow v0.1 deployment profile for Tor-routed Mycel transport

### Design Notes

- [DESIGN-NOTES.client-non-discretionary-multi-view.en.md](./DESIGN-NOTES.client-non-discretionary-multi-view.en.md): Design draft for a protocol-governed multi-view reader model
- [DESIGN-NOTES.two-maintainer-role.en.md](./DESIGN-NOTES.two-maintainer-role.en.md): Design draft for splitting editor and view maintainer authority
- [DESIGN-NOTES.mycel-app-layer.en.md](./DESIGN-NOTES.mycel-app-layer.en.md): Design draft for separating client, runtime, and effect responsibilities
- [DESIGN-NOTES.qa-app-layer.en.md](./DESIGN-NOTES.qa-app-layer.en.md): Design draft for a Q&A app carried by Mycel
- [DESIGN-NOTES.qa-minimal-schema.en.md](./DESIGN-NOTES.qa-minimal-schema.en.md): Minimal schema draft for a Q&A app
- [DESIGN-NOTES.donation-app-layer.en.md](./DESIGN-NOTES.donation-app-layer.en.md): Design draft for a donation-oriented app carried by Mycel
- [DESIGN-NOTES.canonical-text-profile.en.md](./DESIGN-NOTES.canonical-text-profile.en.md): Design draft for a neutral canonical-text profile with witnesses, anchors, commentary, and accepted readings
- [DESIGN-NOTES.interpretation-dispute-model.en.md](./DESIGN-NOTES.interpretation-dispute-model.en.md): Design draft for preserving and governing rival interpretations without overwriting root text
- [DESIGN-NOTES.auto-signer-consent-model.en.md](./DESIGN-NOTES.auto-signer-consent-model.en.md): Design draft for enrollment and consent boundaries in automatic signer systems
- [DESIGN-NOTES.policy-driven-threshold-custody.en.md](./DESIGN-NOTES.policy-driven-threshold-custody.en.md): Design draft for policy-authorized automatic threshold custody
- [DESIGN-NOTES.mycel-anonymity-model.en.md](./DESIGN-NOTES.mycel-anonymity-model.en.md): Design draft for analyzing anonymity across transport, metadata, client, runtime, and replication layers
- [DESIGN-NOTES.first-client-scope-v0.1.en.md](./DESIGN-NOTES.first-client-scope-v0.1.en.md): Design draft for the narrow first-client target and what it must deliberately defer
- [DESIGN-NOTES.mycel-full-stack-map.en.md](./DESIGN-NOTES.mycel-full-stack-map.en.md): Design draft mapping the current Mycel documents into one layered full-stack system view
- [DESIGN-NOTES.mycel-protocol-upgrade-philosophy.en.md](./DESIGN-NOTES.mycel-protocol-upgrade-philosophy.en.md): Design draft for keeping protocol core changes conservative while moving faster in profiles and design notes
- [DESIGN-NOTES.peer-discovery-model.en.md](./DESIGN-NOTES.peer-discovery-model.en.md): Design draft for bounded peer discovery across public, restricted, and Tor-oriented deployments
- [DESIGN-NOTES.sensor-triggered-donation.en.md](./DESIGN-NOTES.sensor-triggered-donation.en.md): Design draft for a sensor-triggered donation flow
- [DESIGN-NOTES.governance-history-security.en.md](./DESIGN-NOTES.governance-history-security.en.md): Design draft for securing governance history

### Meta

- [PROJECT-INTENT.md](./PROJECT-INTENT.md): Project intent and protocol-boundary notes
- [AGENTS.md](./AGENTS.md): Repository collaboration rules

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
