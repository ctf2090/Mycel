# Mycel Grant Concept Note

## Project Title

**Mycel: Verifiable Text History, Governed Reading State, and Decentralized Replication**

## Summary

Mycel is an open protocol stack for text-bearing systems that require:

- verifiable revision history
- governance-aware default reading
- decentralized replication without mandatory global consensus

It is designed for long-lived texts, commentary traditions, governed reference corpora, archives, and other environments where history, interpretation, and auditability must remain durable across time and across multiple valid branches.

Mycel addresses a gap not well served by existing tools. Centralized collaboration systems provide convenience but weak auditability and portability. Code-oriented distributed tools preserve history well but are not designed for governed default reading. Blockchain-style systems provide strong consensus at a cost and with assumptions that are often unnecessary or undesirable for text-governance workflows.

Mycel proposes a different model: history, accepted reading, and replication should remain interoperable, but should not be collapsed into a single consensus mechanism.

In that sense, Mycel fills the gap between centralized collaboration platforms and global-consensus blockchains. It offers verifiable history, rule-derived default reading, and decentralized replication without forcing the whole network into one canonical truth.

It is addressing an awkward in-between problem that has historically been easy to fragment and hard to explain: stricter than ordinary document collaboration, less consensus-heavy than blockchains, more reader-governance-aware than Git, and more portability-focused than a typical application backend.

## Problem Statement

Many important text systems need more than document editing and less than blockchain consensus.

Examples include:

- governed legal or policy commentary
- institutional reference texts and archives
- scholarly annotation systems
- long-lived technical, normative, or cultural corpora
- policy-bound execution systems that depend on accepted textual state

In these settings, stakeholders need to know:

- what changed
- whether the history can be independently verified
- which reading is currently accepted by default
- why that reading was selected
- how alternative valid branches remain visible and auditable

Existing systems typically optimize for one requirement while weakening another. Mycel is intended to support all of these requirements together.

## Why This Gap Has Stayed Underserved

This layer has remained underbuilt partly because the need is distributed rather than concentrated.

Most users first ask for:

- a usable editor
- document versioning
- a centralized backend
- web collaboration

Far fewer start by asking for all of the following at once:

- verifiable history
- multiple valid branches
- default reading derived from fixed rules
- decentralized replication

That means the demand has often been absorbed piecemeal by existing categories instead of producing a dedicated protocol layer.

In practice, the problem has usually been split across four approaches:

- centralized platforms, which are convenient but weak on independent verification and portability
- Git-style workflows, which preserve history well but are not designed around governed default reading
- blockchains and smart contracts, which provide strong consensus at a cost and with assumptions that are often unnecessary for text-governance systems
- custom application backends, which are practical to ship but rarely interoperable and usually lack protocol-level verification

Part of the opportunity for Mycel is to turn that scattered set of needs into one layered, specifiable, and reusable public infrastructure surface.

## Proposed Approach

Mycel separates three layers:

1. **Verifiable history**  
   Revisions should be replayable, checkable, and rebuildable from canonical objects.

2. **Governed reading state**  
   The default accepted reading should be derived from fixed profile rules and verified governance signals, not from discretionary local preference or a claim of global truth.

3. **Decentralized replication**  
   Objects should replicate across peers without requiring a universal consensus result for every reader.

This architecture is intended to preserve both flexibility and auditability while avoiding unnecessary consensus overhead.

## Current Status

The project currently includes:

- a growing v0.1 protocol and wire-spec document set
- a Rust-based internal validation and simulator toolchain
- replay-based revision verification
- deterministic accepted-head inspection
- local object-store ingest and rebuild
- fixtures, simulator topologies, and negative validation coverage

The project does not yet provide:

- a finished interoperable public client
- end-to-end wire sync
- a production-ready node or end-user application

## What Grant Support Would Enable

Grant support would accelerate the most critical public-infrastructure work:

- shared protocol parsing and canonicalization closure
- replay and `state_hash` verification hardening
- rebuildable local storage and accepted-head selection
- deterministic negative testing and interop fixtures
- clearer profile, schema, and documentation boundaries
- a narrow first interoperable client

This is the layer with the highest leverage. Strengthening the shared core makes future profiles, applications, and deployment models safer and more reusable.

## Expected Outcomes

With support, Mycel aims to deliver:

- a stronger open protocol core for verifiable text systems and cultural reference corpora
- a more complete first-client implementation path
- reusable fixtures and negative tests for interoperability
- clearer public documentation around governed reading and accepted-state derivation
- a reference base for future text-governance, commentary, and cultural-preservation applications

## Why This Project Matters

Mycel explores an underbuilt part of digital infrastructure: systems that must preserve history, governance, and interpretation without collapsing them into either centralized platform control or mandatory global consensus.

This work has public value because it supports:

- durable and auditable knowledge systems
- text-governance and cultural-preservation infrastructure
- reproducible interpretation and commentary workflows
- open protocol alternatives to closed platforms

It also matters because this is exactly the sort of infrastructure gap that markets do not reliably fill on their own: the engineering burden is real, but the benefits are shared across many domains rather than concentrated in one obvious product category.

## Funding Fit

Mycel is a strong fit for grants that support:

- open digital infrastructure
- public-interest protocol development
- verifiable knowledge systems
- interoperable open-source foundations
- trustworthy data and governance tooling

## Closing

Mycel is still early, but it has already moved beyond pure concept into a concrete spec, implementation, and validation trajectory. Support at this stage would help turn a promising protocol direction into a usable public infrastructure foundation for governed text systems and cultural stewardship.
