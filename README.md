# Mycel

Language: English | [繁體中文](./README.zh-TW.md)

Mycel is a neutral, technical protocol for verifiable text history and decentralized replication.

## Overview

Mycel is designed for text-first collaboration in distributed environments:

- Verifiable change history
- P2P replication
- Digital-signature validation
- Multi-branch coexistence without requiring global single consensus

## Neutrality

Mycel can be used in many content domains. The protocol itself remains neutral and purely technical.

## Current Status

- Protocol stage: `v0.1` conceptual specification
- Current focus: spec consistency, implementation readiness, and governance simplification

## Documentation

### Specs

- [PROTOCOL.en.md](./PROTOCOL.en.md): Full protocol specification in English
- [PROTOCOL.zh-TW.md](./PROTOCOL.zh-TW.md): Full protocol specification in Traditional Chinese
- [WIRE-PROTOCOL.en.md](./WIRE-PROTOCOL.en.md): Wire protocol draft in English
- [WIRE-PROTOCOL.zh-TW.md](./WIRE-PROTOCOL.zh-TW.md): Wire protocol draft in Traditional Chinese
- [IMPLEMENTATION-CHECKLIST.en.md](./IMPLEMENTATION-CHECKLIST.en.md): Implementation checklist for a minimal v0.1 client
- [IMPLEMENTATION-CHECKLIST.zh-TW.md](./IMPLEMENTATION-CHECKLIST.zh-TW.md): Traditional Chinese implementation checklist
- [PROFILE.fund-auto-disbursement-v0.1.en.md](./PROFILE.fund-auto-disbursement-v0.1.en.md): Narrow v0.1 profile draft for automatic fund disbursement
- [PROFILE.fund-auto-disbursement-v0.1.zh-TW.md](./PROFILE.fund-auto-disbursement-v0.1.zh-TW.md): Traditional Chinese profile draft for the same flow
- [PROFILE.mycel-over-tor-v0.1.en.md](./PROFILE.mycel-over-tor-v0.1.en.md): Narrow v0.1 deployment profile for Tor-routed Mycel transport
- [PROFILE.mycel-over-tor-v0.1.zh-TW.md](./PROFILE.mycel-over-tor-v0.1.zh-TW.md): Traditional Chinese profile draft for the same Tor-oriented deployment

### Design Notes

- [DESIGN-NOTES.client-non-discretionary-multi-view.en.md](./DESIGN-NOTES.client-non-discretionary-multi-view.en.md): Design draft for a protocol-governed multi-view reader model
- [DESIGN-NOTES.client-non-discretionary-multi-view.zh-TW.md](./DESIGN-NOTES.client-non-discretionary-multi-view.zh-TW.md): Traditional Chinese design draft for the same model
- [DESIGN-NOTES.two-maintainer-role.en.md](./DESIGN-NOTES.two-maintainer-role.en.md): Design draft for splitting editor and view maintainer authority
- [DESIGN-NOTES.two-maintainer-role.zh-TW.md](./DESIGN-NOTES.two-maintainer-role.zh-TW.md): Traditional Chinese design draft for the same split-role model
- [DESIGN-NOTES.mycel-app-layer.en.md](./DESIGN-NOTES.mycel-app-layer.en.md): Design draft for separating client, runtime, and effect responsibilities
- [DESIGN-NOTES.mycel-app-layer.zh-TW.md](./DESIGN-NOTES.mycel-app-layer.zh-TW.md): Traditional Chinese design draft for the same App Layer model
- [DESIGN-NOTES.qa-app-layer.en.md](./DESIGN-NOTES.qa-app-layer.en.md): Design draft for a Q&A app carried by Mycel
- [DESIGN-NOTES.qa-app-layer.zh-TW.md](./DESIGN-NOTES.qa-app-layer.zh-TW.md): Traditional Chinese design draft for the same Q&A app model
- [DESIGN-NOTES.qa-minimal-schema.en.md](./DESIGN-NOTES.qa-minimal-schema.en.md): Minimal schema draft for a Q&A app
- [DESIGN-NOTES.qa-minimal-schema.zh-TW.md](./DESIGN-NOTES.qa-minimal-schema.zh-TW.md): Traditional Chinese minimal schema draft for the same Q&A app
- [DESIGN-NOTES.donation-app-layer.en.md](./DESIGN-NOTES.donation-app-layer.en.md): Design draft for a donation-oriented app carried by Mycel
- [DESIGN-NOTES.donation-app-layer.zh-TW.md](./DESIGN-NOTES.donation-app-layer.zh-TW.md): Traditional Chinese design draft for the same donation app model
- [DESIGN-NOTES.canonical-text-profile.en.md](./DESIGN-NOTES.canonical-text-profile.en.md): Design draft for a neutral canonical-text profile with witnesses, anchors, commentary, and accepted readings
- [DESIGN-NOTES.canonical-text-profile.zh-TW.md](./DESIGN-NOTES.canonical-text-profile.zh-TW.md): Traditional Chinese design draft for the same canonical-text profile
- [DESIGN-NOTES.auto-signer-consent-model.en.md](./DESIGN-NOTES.auto-signer-consent-model.en.md): Design draft for enrollment and consent boundaries in automatic signer systems
- [DESIGN-NOTES.auto-signer-consent-model.zh-TW.md](./DESIGN-NOTES.auto-signer-consent-model.zh-TW.md): Traditional Chinese design draft for the same consent model
- [DESIGN-NOTES.policy-driven-threshold-custody.en.md](./DESIGN-NOTES.policy-driven-threshold-custody.en.md): Design draft for policy-authorized automatic threshold custody
- [DESIGN-NOTES.policy-driven-threshold-custody.zh-TW.md](./DESIGN-NOTES.policy-driven-threshold-custody.zh-TW.md): Traditional Chinese design draft for the same custody model
- [DESIGN-NOTES.mycel-anonymity-model.en.md](./DESIGN-NOTES.mycel-anonymity-model.en.md): Design draft for analyzing anonymity across transport, metadata, client, runtime, and replication layers
- [DESIGN-NOTES.mycel-anonymity-model.zh-TW.md](./DESIGN-NOTES.mycel-anonymity-model.zh-TW.md): Traditional Chinese design draft for the same anonymity model
- [DESIGN-NOTES.first-client-scope-v0.1.en.md](./DESIGN-NOTES.first-client-scope-v0.1.en.md): Design draft for the narrow first-client target and what it must deliberately defer
- [DESIGN-NOTES.first-client-scope-v0.1.zh-TW.md](./DESIGN-NOTES.first-client-scope-v0.1.zh-TW.md): Traditional Chinese design draft for the same first-client scope
- [DESIGN-NOTES.mycel-full-stack-map.en.md](./DESIGN-NOTES.mycel-full-stack-map.en.md): Design draft mapping the current Mycel documents into one layered full-stack system view
- [DESIGN-NOTES.mycel-full-stack-map.zh-TW.md](./DESIGN-NOTES.mycel-full-stack-map.zh-TW.md): Traditional Chinese design draft for the same full-stack map
- [DESIGN-NOTES.mycel-protocol-upgrade-philosophy.en.md](./DESIGN-NOTES.mycel-protocol-upgrade-philosophy.en.md): Design draft for keeping protocol core changes conservative while moving faster in profiles and design notes
- [DESIGN-NOTES.mycel-protocol-upgrade-philosophy.zh-TW.md](./DESIGN-NOTES.mycel-protocol-upgrade-philosophy.zh-TW.md): Traditional Chinese design draft for the same upgrade philosophy
- [DESIGN-NOTES.peer-discovery-model.en.md](./DESIGN-NOTES.peer-discovery-model.en.md): Design draft for bounded peer discovery across public, restricted, and Tor-oriented deployments
- [DESIGN-NOTES.peer-discovery-model.zh-TW.md](./DESIGN-NOTES.peer-discovery-model.zh-TW.md): Traditional Chinese design draft for the same peer-discovery model
- [DESIGN-NOTES.sensor-triggered-donation.en.md](./DESIGN-NOTES.sensor-triggered-donation.en.md): Design draft for a sensor-triggered donation flow
- [DESIGN-NOTES.sensor-triggered-donation.zh-TW.md](./DESIGN-NOTES.sensor-triggered-donation.zh-TW.md): Traditional Chinese design draft for the same flow
- [DESIGN-NOTES.governance-history-security.en.md](./DESIGN-NOTES.governance-history-security.en.md): Design draft for securing governance history
- [DESIGN-NOTES.governance-history-security.zh-TW.md](./DESIGN-NOTES.governance-history-security.zh-TW.md): Traditional Chinese design draft for the same topic

### Meta

- [PROJECT-INTENT.md](./PROJECT-INTENT.md): Project intent and protocol-boundary notes
- [PROJECT-INTENT.zh-TW.md](./PROJECT-INTENT.zh-TW.md): Traditional Chinese version of the same project-intent note
- [AGENTS.md](./AGENTS.md): Repository collaboration rules

## Near-Term Priorities

1. Use the implementation checklist to scope a minimal reference client
2. Decide whether v0.1 governance parameters should be reduced further before stabilization
3. Turn the minimal checklist into a narrower reference profile if we want stricter first-client constraints

## Project Scope

Mycel is not:

- a blockchain with global mandatory consensus
- a plain file transfer system
- a Git clone

Mycel is a protocol for verifiable, evolvable, decentralized text history.
