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
- Current focus: object model, signing model, replication flow, and merge behavior

## Documentation

- [PROTOCOL.en.md](./PROTOCOL.en.md): Full protocol specification in English
- [PROTOCOL.zh-TW.md](./PROTOCOL.zh-TW.md): Full protocol specification in Traditional Chinese
- [AGENTS.md](./AGENTS.md): Repository collaboration rules

## Near-Term Priorities

1. Finalize wire protocol fields (`HELLO`, `WANT`, `OBJECT`)
2. Lock canonical serialization rules for deterministic hashing
3. Define block-level merge semantics

## Project Scope

Mycel is not:

- a blockchain with global mandatory consensus
- a plain file transfer system
- a Git clone

Mycel is a protocol for verifiable, evolvable, decentralized text history.
