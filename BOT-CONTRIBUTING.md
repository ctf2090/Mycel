# Bot Contributing Guide

This guide is for AI coding bots and other automation-oriented contributors working in this repository.

For the repo-wide model of how humans, issues, agents, verification, and planning sync fit together, start with [docs/AI-CO-WORKING-MODEL.md](./docs/AI-CO-WORKING-MODEL.md). This guide stays narrower and more execution-oriented.

Use it together with:

- [README.md](./README.md)
- [docs/DEV-SETUP.md](./docs/DEV-SETUP.md)
- [CONTRIBUTING.md](./CONTRIBUTING.md)
- [AGENTS.md](./AGENTS.md)
- [docs/AI-CO-WORKING-MODEL.md](./docs/AI-CO-WORKING-MODEL.md) for the higher-level human-plus-agent operating model
- [`.github/labels.yml`](./.github/labels.yml) for the repo-tracked bot/task label set
- [docs/LABELS.md](./docs/LABELS.md) for label meanings and recommended combinations
- [docs/MULTI-AGENT-COORDINATION.md](./docs/MULTI-AGENT-COORDINATION.md) for parallel issue ownership, file boundaries, and push discipline
- [docs/PLANNING-SYNC-PLAN.md](./docs/PLANNING-SYNC-PLAN.md) for keeping roadmap, checklist, issues, and Pages summaries aligned
- [`scripts/sync-labels.sh`](./scripts/sync-labels.sh) to apply tracked labels back to GitHub
- [`scripts/check-labels.sh`](./scripts/check-labels.sh) to verify GitHub labels still match the tracked set
- [`scripts/check-doc-refresh.sh`](./scripts/check-doc-refresh.sh) to check whether planning-doc refresh is due

## Agent Onboarding

If you are starting fresh in this repo, use this order:

1. Read [README.md](./README.md) for scope and current positioning.
2. Read [ROADMAP.md](./ROADMAP.md) and [IMPLEMENTATION-CHECKLIST.en.md](./IMPLEMENTATION-CHECKLIST.en.md) for build order and closure targets.
3. Read [PROTOCOL.en.md](./PROTOCOL.en.md) and [WIRE-PROTOCOL.en.md](./WIRE-PROTOCOL.en.md) before changing protocol-facing behavior.
4. Read [`.github/labels.yml`](./.github/labels.yml) and [docs/LABELS.md](./docs/LABELS.md) before creating or triaging bot-ready issues.
5. Run [`scripts/sync-labels.sh`](./scripts/sync-labels.sh) only if GitHub labels need to be applied or refreshed.
6. Run [`scripts/check-labels.sh`](./scripts/check-labels.sh) if you need to verify that the tracked labels still match GitHub.
7. Run [`scripts/check-doc-refresh.sh`](./scripts/check-doc-refresh.sh) before or after a work batch if planning-doc cadence may be due.
8. Use [`.github/ISSUE_TEMPLATE/ai_ready_task.yml`](./.github/ISSUE_TEMPLATE/ai_ready_task.yml) and [docs/PROGRESS.md](./docs/PROGRESS.md) when shaping or selecting work.

This keeps scope, labels, and task shape aligned before implementation work starts.

If you need a machine-readable environment gate before starting work, use:

```bash
scripts/check-dev-env.sh --json
```

The JSON output includes:

- `status`
- `mode`
- `repo_root`
- `required_toolchain_channel`
- `minimum_rust`
- `checks[]`
- `error` when the check fails

If you need a machine-readable planning-sync gate, use:

```bash
scripts/check-doc-refresh.sh --json
```

The JSON output includes:

- `status` (`ok`, `due`, or `failed`)
- `threshold`
- `repo_root`
- `highest_commit_distance`
- `remaining_commits`
- `checks[]`
- `error` when the script itself cannot complete

## What Kind of Work Fits This Repo Best

Mycel is easiest to contribute to when the work is:

- narrow in scope
- spec-aligned
- deterministic to verify
- tied to fixtures, tests, or explicit acceptance criteria

The best contribution shapes are usually:

1. close one protocol-to-code gap
2. add one verification rule with tests
3. extend one typed parsing surface
4. add one fixture-backed negative case
5. strengthen one replay, store, or selector invariant

Avoid broad cleanup or style-only churn unless it directly supports active implementation work.

## Read Order

Before changing code, read in this order:

1. [README.md](./README.md) for project scope
2. [ROADMAP.md](./ROADMAP.md) for milestone order
3. [IMPLEMENTATION-CHECKLIST.en.md](./IMPLEMENTATION-CHECKLIST.en.md) for closure targets
4. [PROTOCOL.en.md](./PROTOCOL.en.md) and [WIRE-PROTOCOL.en.md](./WIRE-PROTOCOL.en.md) for normative behavior
5. [RUST-WORKSPACE.md](./RUST-WORKSPACE.md) for crate layout
6. [fixtures/README.md](./fixtures/README.md) and [sim/README.md](./sim/README.md) for verification surfaces

If a change touches accepted-head behavior or governance semantics, also read:

- [docs/design-notes/DESIGN-NOTES.client-non-discretionary-multi-view.en.md](./docs/design-notes/DESIGN-NOTES.client-non-discretionary-multi-view.en.md)
- [docs/design-notes/DESIGN-NOTES.two-maintainer-role.en.md](./docs/design-notes/DESIGN-NOTES.two-maintainer-role.en.md)

## Where To Start

If you need a starting point, prefer tasks in these areas:

1. `crates/mycel-core/src/protocol.rs`
   Typed parsing, field-shape checks, logical-ID and derived-ID handling.
2. `crates/mycel-core/src/verify.rs`
   Canonicalization, derived-ID recomputation, signature checks, replay validation.
3. `apps/mycel-cli/tests/`
   Narrow smoke coverage that proves current CLI behavior stays aligned with shared core behavior.
4. `fixtures/`
   Deterministic regression inputs and negative validation cases.
5. `sim/`
   Validation and reporting coverage around deterministic simulator workflows.

## What To Avoid

Do not assume these are welcome unless the task explicitly requires them:

- broad architectural rewrites
- speculative protocol expansion
- backward-compatibility shims
- large CLI UX redesigns
- mixing unrelated refactors with behavior changes
- introducing discretionary local policy into accepted-head behavior

## Preferred Task Format

This repo works best when work is framed with:

1. one problem statement
2. one narrow scope boundary
3. one or two primary files
4. one verification command set
5. explicit non-goals

Example:

- Problem: `snapshot` objects are not yet handled through the shared typed parser.
- Scope: update shared parsing and the direct tests around it only.
- Files: `crates/mycel-core/src/protocol.rs`, `crates/mycel-core/tests/...`
- Verify: `cargo test -p mycel-core`
- Non-goals: CLI UX changes, wire protocol work, store schema changes

## Verification Expectations

Contributions are much more likely to be useful if they end with deterministic verification.

Common commands:

```bash
cargo test -p mycel-core
cargo test -p mycel-cli
cargo run -p mycel-cli -- validate fixtures/object-sets/minimal-valid/fixture.json --json
cargo run -p mycel-cli -- object inspect <path> --json
cargo run -p mycel-cli -- object verify <path> --json
./sim/negative-validation/smoke.sh --summary-only
```

Prefer adding or updating tests close to the shared core instead of relying only on CLI-level coverage.

## Acceptance Style

Good contributions in this repo usually satisfy most of the following:

- one spec or checklist item is more clearly closed than before
- behavior is deterministic
- fixtures or tests prove the change
- unrelated files are left alone
- follow-up work is explicit instead of hidden

## If the Spec Is Ambiguous

Do not silently invent protocol behavior.

If a task depends on an ambiguous rule:

1. point to the exact spec or design-note section
2. describe the conflicting interpretations
3. choose the most conservative implementation only if the task explicitly requires progress now
4. otherwise leave the ambiguity visible for maintainers to resolve

## Best Next Tasks for Bots

The most bot-friendly heavy tasks in this repo are usually:

1. closing typed object-family coverage
2. extracting shared canonical helpers from verification code
3. tightening object-field strictness and negative tests
4. strengthening replay and store rebuild coverage
5. preparing narrow wire-validation primitives without widening full sync scope
