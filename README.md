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

- [PROTOCOL.en.md](./PROTOCOL.en.md): Full protocol specification in English
- [PROTOCOL.zh-TW.md](./PROTOCOL.zh-TW.md): Full protocol specification in Traditional Chinese
- [WIRE-PROTOCOL.en.md](./WIRE-PROTOCOL.en.md): Wire protocol draft in English
- [WIRE-PROTOCOL.zh-TW.md](./WIRE-PROTOCOL.zh-TW.md): Wire protocol draft in Traditional Chinese
- [IMPLEMENTATION-CHECKLIST.en.md](./IMPLEMENTATION-CHECKLIST.en.md): Implementation checklist for a minimal v0.1 client
- [IMPLEMENTATION-CHECKLIST.zh-TW.md](./IMPLEMENTATION-CHECKLIST.zh-TW.md): Traditional Chinese implementation checklist
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
- [DESIGN-NOTES.neuro-triggered-donation.en.md](./DESIGN-NOTES.neuro-triggered-donation.en.md): Design draft for a neuro-triggered donation flow
- [DESIGN-NOTES.neuro-triggered-donation.zh-TW.md](./DESIGN-NOTES.neuro-triggered-donation.zh-TW.md): Traditional Chinese design draft for the same flow
- [MYCEL-PROJECT-NOTES.md](./MYCEL-PROJECT-NOTES.md): Short project-specific notes and constraints
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
