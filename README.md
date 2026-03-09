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

- Protocol: [EN](./PROTOCOL.en.md) | [繁中](./PROTOCOL.zh-TW.md)
- Wire Protocol: [EN](./WIRE-PROTOCOL.en.md) | [繁中](./WIRE-PROTOCOL.zh-TW.md)
- Implementation Checklist: [EN](./IMPLEMENTATION-CHECKLIST.en.md) | [繁中](./IMPLEMENTATION-CHECKLIST.zh-TW.md)
- Fund Auto-disbursement Profile v0.1: [EN](./PROFILE.fund-auto-disbursement-v0.1.en.md) | [繁中](./PROFILE.fund-auto-disbursement-v0.1.zh-TW.md)
- Mycel over Tor Profile v0.1: [EN](./PROFILE.mycel-over-tor-v0.1.en.md) | [繁中](./PROFILE.mycel-over-tor-v0.1.zh-TW.md)

### Design Notes

- Client Non-discretionary Multi-view: [EN](./DESIGN-NOTES.client-non-discretionary-multi-view.en.md) | [繁中](./DESIGN-NOTES.client-non-discretionary-multi-view.zh-TW.md)
- Two-Maintainer-Role Model: [EN](./DESIGN-NOTES.two-maintainer-role.en.md) | [繁中](./DESIGN-NOTES.two-maintainer-role.zh-TW.md)
- Mycel App Layer: [EN](./DESIGN-NOTES.mycel-app-layer.en.md) | [繁中](./DESIGN-NOTES.mycel-app-layer.zh-TW.md)
- Q&A App Layer: [EN](./DESIGN-NOTES.qa-app-layer.en.md) | [繁中](./DESIGN-NOTES.qa-app-layer.zh-TW.md)
- Q&A Minimal Schema: [EN](./DESIGN-NOTES.qa-minimal-schema.en.md) | [繁中](./DESIGN-NOTES.qa-minimal-schema.zh-TW.md)
- Donation App Layer: [EN](./DESIGN-NOTES.donation-app-layer.en.md) | [繁中](./DESIGN-NOTES.donation-app-layer.zh-TW.md)
- Canonical Text Profile: [EN](./DESIGN-NOTES.canonical-text-profile.en.md) | [繁中](./DESIGN-NOTES.canonical-text-profile.zh-TW.md)
- Interpretation Dispute Model: [EN](./DESIGN-NOTES.interpretation-dispute-model.en.md) | [繁中](./DESIGN-NOTES.interpretation-dispute-model.zh-TW.md)
- Auto-signer Consent Model: [EN](./DESIGN-NOTES.auto-signer-consent-model.en.md) | [繁中](./DESIGN-NOTES.auto-signer-consent-model.zh-TW.md)
- Policy-driven Threshold Custody: [EN](./DESIGN-NOTES.policy-driven-threshold-custody.en.md) | [繁中](./DESIGN-NOTES.policy-driven-threshold-custody.zh-TW.md)
- Mycel Anonymity Model: [EN](./DESIGN-NOTES.mycel-anonymity-model.en.md) | [繁中](./DESIGN-NOTES.mycel-anonymity-model.zh-TW.md)
- First-client Scope v0.1: [EN](./DESIGN-NOTES.first-client-scope-v0.1.en.md) | [繁中](./DESIGN-NOTES.first-client-scope-v0.1.zh-TW.md)
- Mycel Full-stack Map: [EN](./DESIGN-NOTES.mycel-full-stack-map.en.md) | [繁中](./DESIGN-NOTES.mycel-full-stack-map.zh-TW.md)
- Mycel Protocol Upgrade Philosophy: [EN](./DESIGN-NOTES.mycel-protocol-upgrade-philosophy.en.md) | [繁中](./DESIGN-NOTES.mycel-protocol-upgrade-philosophy.zh-TW.md)
- Peer Discovery Model: [EN](./DESIGN-NOTES.peer-discovery-model.en.md) | [繁中](./DESIGN-NOTES.peer-discovery-model.zh-TW.md)
- Sensor-triggered Donation: [EN](./DESIGN-NOTES.sensor-triggered-donation.en.md) | [繁中](./DESIGN-NOTES.sensor-triggered-donation.zh-TW.md)
- Governance History Security: [EN](./DESIGN-NOTES.governance-history-security.en.md) | [繁中](./DESIGN-NOTES.governance-history-security.zh-TW.md)

### Meta

- Project Intent: [EN](./PROJECT-INTENT.md) | [繁中](./PROJECT-INTENT.zh-TW.md)
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
