# Mycel

Language: English | [繁體中文](./README.zh-TW.md)

Mycel is a Rust-based protocol stack for verifiable text history, governed reading state, and decentralized replication.

It is aimed at text-bearing systems that need:

- replay-verifiable history
- signed governance signals
- multiple valid branches without global mandatory consensus
- profile-governed accepted reading instead of ad hoc local preference

## Why Mycel

Most collaboration tools force one of two shapes:

- centralized platforms with mutable state
- distributed systems that optimize for code collaboration or global consensus

Mycel takes a different path. It treats text history, accepted reading, and replication as separate but interoperable concerns.

The result is a protocol stack for long-lived texts, commentary systems, governed reference corpora, and other text-first distributed workflows.

## What Makes It Different

- Verifiable history: revisions are meant to be replayed and checked, not merely trusted.
- Governed reading state: accepted heads come from fixed profile rules and verified View objects.
- Fork-tolerant model: multiple heads can coexist without pretending one global truth exists everywhere.
- Neutral protocol core: domain-specific meaning belongs in profiles and app layers, not in the core protocol.

In other words, an "accepted version" is not network-wide consensus. It is the version derived under one fixed profile from verified objects.

## What It Is Not

Mycel is not:

- a blockchain with mandatory global consensus
- a Git clone
- a generic file transfer layer

## Try It In 60 Seconds

The current Rust CLI is an internal validation and simulator toolchain, not yet a production Mycel client or node.

If you are starting from a fresh environment, read [`docs/DEV-SETUP.md`](./docs/DEV-SETUP.md) first.

From the repo root:

```bash
cargo run -p mycel-cli -- info
cargo run -p mycel-cli -- validate fixtures/object-sets/minimal-valid/fixture.json --json
cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json --json
```

What those commands show:

- `info`: repo-local workspace and scaffold paths
- `validate`: stable validation output over checked-in fixtures
- `sim run`: deterministic simulator-harness output plus a generated report path

## Current Status

- Protocol stage: `v0.1` conceptual specification with growing profile and design-note layers
- Current implementation focus: first-client scoping, replay/verification hardening, and deterministic simulator workflows
- Current CLI boundary: suitable for internal validation, object inspection, object verification, accepted-head inspection, report inspection, and simulator runs inside this repository
- Not yet delivered: production node behavior, public-network wire sync, or a finished end-user client

## Read By Goal

If you want the shortest useful path:

- Start here for the protocol core: [PROTOCOL.en.md](./PROTOCOL.en.md)
- Read transport rules next: [WIRE-PROTOCOL.en.md](./WIRE-PROTOCOL.en.md)
- See implementation order: [ROADMAP.md](./ROADMAP.md)
- See build checklist: [IMPLEMENTATION-CHECKLIST.en.md](./IMPLEMENTATION-CHECKLIST.en.md)

If you want to contribute from a fresh environment:

- Start with setup: [docs/DEV-SETUP.md](./docs/DEV-SETUP.md)
- Then read contribution expectations: [CONTRIBUTING.md](./CONTRIBUTING.md)
- If you are using an AI coding agent, continue with: [BOT-CONTRIBUTING.md](./BOT-CONTRIBUTING.md)

## Start Contributing Here

If you want a narrow first task, start with one of these open issues:

- [#1 Reject duplicate JSON object keys in shared object parsing](https://github.com/ctf2090/Mycel/issues/1)
- [#3 Add malformed logical-ID coverage for document and block objects](https://github.com/ctf2090/Mycel/issues/3)
- [#4 Add snapshot derived-ID verification smoke coverage](https://github.com/ctf2090/Mycel/issues/4)

For more structured task intake, browse issues labeled `ai-ready` and `well-scoped`.

If you want to understand the current Rust implementation:

- Workspace map: [RUST-WORKSPACE.md](./RUST-WORKSPACE.md)
- Simulator scaffold: [sim/README.md](./sim/README.md)
- Fixture layout: [fixtures/README.md](./fixtures/README.md)

If you want the design layer behind current decisions:

- First-client boundary: [docs/design-notes/DESIGN-NOTES.first-client-scope-v0.1.en.md](./docs/design-notes/DESIGN-NOTES.first-client-scope-v0.1.en.md)
- Full-stack map: [docs/design-notes/DESIGN-NOTES.mycel-full-stack-map.en.md](./docs/design-notes/DESIGN-NOTES.mycel-full-stack-map.en.md)
- Protocol upgrade philosophy: [docs/design-notes/DESIGN-NOTES.mycel-protocol-upgrade-philosophy.en.md](./docs/design-notes/DESIGN-NOTES.mycel-protocol-upgrade-philosophy.en.md)

## Key Documents

### Specs

- [PROTOCOL.en.md](./PROTOCOL.en.md): core protocol specification
- [WIRE-PROTOCOL.en.md](./WIRE-PROTOCOL.en.md): transport message format and sync flow draft
- [ROADMAP.md](./ROADMAP.md): phased build order from first client to later expansion
- [IMPLEMENTATION-CHECKLIST.en.md](./IMPLEMENTATION-CHECKLIST.en.md): implementation checklist for a narrow interoperable client
- [PROFILE.fund-auto-disbursement-v0.1.en.md](./PROFILE.fund-auto-disbursement-v0.1.en.md): narrow app-layer custody profile draft
- [PROFILE.mycel-over-tor-v0.1.en.md](./PROFILE.mycel-over-tor-v0.1.en.md): narrow Tor-oriented deployment profile draft

### Design Notes

- [docs/design-notes/DESIGN-NOTES.first-client-scope-v0.1.en.md](./docs/design-notes/DESIGN-NOTES.first-client-scope-v0.1.en.md): what the first client should do now, and what it should defer
- [docs/design-notes/DESIGN-NOTES.client-non-discretionary-multi-view.en.md](./docs/design-notes/DESIGN-NOTES.client-non-discretionary-multi-view.en.md): protocol-governed reader model
- [docs/design-notes/DESIGN-NOTES.two-maintainer-role.en.md](./docs/design-notes/DESIGN-NOTES.two-maintainer-role.en.md): split between editor and view maintainer authority
- [docs/design-notes/DESIGN-NOTES.mycel-full-stack-map.en.md](./docs/design-notes/DESIGN-NOTES.mycel-full-stack-map.en.md): layered map of the current document set
- [docs/design-notes/DESIGN-NOTES.peer-simulator-v0.en.md](./docs/design-notes/DESIGN-NOTES.peer-simulator-v0.en.md): early simulator and harness direction

### Meta

- [PROJECT-INTENT.md](./PROJECT-INTENT.md): project-intent boundary notes
- [CONTRIBUTING.md](./CONTRIBUTING.md): contribution expectations
- [docs/DEV-SETUP.md](./docs/DEV-SETUP.md): shortest path from fresh checkout to a usable dev environment
- [BOT-CONTRIBUTING.md](./BOT-CONTRIBUTING.md): bot-oriented contribution guide for narrow, verifiable work
- [docs/LABELS.md](./docs/LABELS.md): meanings and recommended combinations for tracked bot/task labels
- [AGENTS.md](./AGENTS.md): repository collaboration rules
- [docs/OUTWARD-RELEASE-CHECKLIST.md](./docs/OUTWARD-RELEASE-CHECKLIST.md): outward-facing publish checklist for repo, homepage, and share previews

## Near-Term Priorities

1. Finish the narrow first-client core around verification, replay, storage, and accepted-head inspection.
2. Move mature ideas into explicit profiles, schemas, fixtures, and tests before widening protocol-core scope.
3. Expand upward one layer at a time: canonical-text reading first, then selective app-layer support.

## License

This repository is licensed under the [MIT License](./LICENSE), unless a future file or directory states otherwise.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for contribution and license expectations.
