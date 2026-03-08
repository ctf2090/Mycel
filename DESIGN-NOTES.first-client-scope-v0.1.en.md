# First-client Scope v0.1

Status: design draft

This note defines the narrowest practical first-client scope for Mycel.

The main design principle is:

- the first client should prove protocol viability, not full platform ambition
- reader behavior should come before rich authoring or app execution
- explicit profiles should be chosen up front
- everything outside the chosen scope should remain deferred on purpose

## 0. Goal

Set a realistic, buildable first-client target that:

- validates the core protocol
- validates the wire sync path
- validates accepted-head selection
- presents accepted text to a reader

This note is intentionally narrower than the full document set.

## 1. Build Shape

The recommended first client is:

- a reader-first client
- one local node process
- one local object store
- one constrained wire implementation
- one narrow profile set

It is not:

- a full editor
- a general-purpose app runtime
- a fund-execution node
- a signer node
- a broad public mesh node

## 2. In-Scope Layers

The first client should include only these layers:

1. protocol core
2. object verification and local state
3. minimal synchronization and transport
4. governance and accepted-state selection
5. reader-facing client surface

It may optionally include:

- a narrow anonymous transport profile
- a narrow canonical-text reading profile

## 3. Required Protocol Features

The first client should implement:

- `document`
- `block`
- `patch`
- `revision`
- `view`
- `snapshot`
- canonical serialization
- derived-ID verification
- signature verification
- replay-based `state_hash` verification

Required wire messages:

- `HELLO`
- `MANIFEST`
- `HEADS`
- `WANT`
- `OBJECT`
- `BYE`
- `ERROR`

Optional for first build:

- `SNAPSHOT_OFFER`
- `VIEW_ANNOUNCE`

## 4. Required Local Capabilities

The first client should support:

- one persistent local object store
- rebuildable local indexes
- accepted-head computation for one fixed reader profile
- verified-object-only indexing
- local state rebuild from canonical objects alone

The client should not depend on:

- server-side database infrastructure
- external search services
- background execution runtimes

## 5. Required Reader Behavior

The first client should be able to:

- open a document by logical ID
- compute and display the accepted head
- render accepted text
- show basic history context
- show alternative heads
- show which profile selected the accepted head

Recommended but still minimal:

- show citations or source references if present
- show a compact `Why this text` panel

## 6. Chosen Profiles

The first client should explicitly choose its supported profiles.

Recommended first set:

- one fixed reader profile for accepted-head selection
- optional `mycel-over-tor-v0.1`

Deferred profiles:

- `fund-auto-disbursement-v0.1`
- signer-oriented custody profiles
- runtime-heavy app profiles

The first client should not claim support for profiles it does not implement end to end.

## 7. Explicit Non-goals

The first client should defer all of the following:

- rich editing UX
- editor-maintainer authoring workflows
- view-maintainer publication workflows
- donation execution
- threshold custody
- automatic effect execution
- sensor-triggered flows
- broad Q&A authoring workflows
- public anonymous mesh discovery beyond bounded configuration

Deferral is a feature, not a failure.

## 8. Networking Posture

The first client should use a bounded network posture.

Recommended modes:

- restricted peer list
- explicit bootstrap peers
- optional Tor-routed transport

The first client should avoid:

- uncontrolled public discovery
- transport fallback ambiguity
- role mixing between reader and signer/runtime behavior

## 9. UI Surface

The first client UI should stay reader-first.

Primary views:

- library or document list
- accepted text reader
- history and branch inspection
- profile or selection status

Deferred UI surfaces:

- full curator console
- fund operations console
- signer controls
- runtime control panels

## 10. Testing Gate

The first client should not be considered complete unless it passes:

- canonical object parsing tests
- derived-ID verification tests
- wire sync end-to-end tests
- replay-based state reconstruction tests
- deterministic accepted-head tests
- rebuild-from-store tests

## 11. Minimal Success Criteria

The first client is successful if a new user can:

1. bootstrap a local node
2. connect to a bounded peer set
3. sync objects
4. verify them locally
5. compute the accepted head for a document
6. read the accepted text with basic trace context

If those six things work, Mycel has a real first client.

## 12. Recommended Next Step After This Scope

Once the first client works, the next expansion should be one of:

- richer canonical-text reading
- basic authoring workflows
- profile-specific app support

The first expansion should still add only one major layer at a time.
