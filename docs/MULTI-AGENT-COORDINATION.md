# Multi-Agent Coordination Note

Status: draft

This note describes how multiple AI coding agents should work in parallel in the Mycel repository without colliding on scope, files, or push order.

For the short maintainer version, see [MULTI-AGENT-CHEATSHEET.md](./MULTI-AGENT-CHEATSHEET.md).

Use it together with:

- [BOT-CONTRIBUTING.md](../BOT-CONTRIBUTING.md)
- [ROADMAP.md](../ROADMAP.md)
- [IMPLEMENTATION-CHECKLIST.en.md](../IMPLEMENTATION-CHECKLIST.en.md)
- [docs/PROGRESS.md](./PROGRESS.md)
- [docs/LABELS.md](./LABELS.md)

## Goal

The goal is not to maximize the number of active chats. The goal is to let multiple agents work at once while keeping:

- scope narrow
- file ownership clear
- verification deterministic
- pushes serial
- reviewable diffs small

## Core Rule

Use one agent per issue, and one active issue per agent.

If a task cannot stay mostly inside one issue boundary, split the task instead of expanding the agent scope.

## Claiming Work

Before an agent starts:

1. choose one open issue
2. check whether another agent or human is already working on it
3. leave a short claim note in the issue or team channel
4. confirm the likely file set before editing

Recommended claim format:

- `Claiming #5 for protocol/parser work in protocol.rs plus direct tests.`

Do not let two agents actively write the same issue unless the work is explicitly split into separate file regions.

## File-Boundary Ownership

Preferred ownership split:

- `crates/mycel-core/src/protocol.rs`
  one agent at a time
- `crates/mycel-core/src/verify.rs`
  one agent at a time
- `crates/mycel-core/src/head.rs`
  one agent at a time
- `crates/mycel-core/src/store.rs`
  one agent at a time
- `apps/mycel-cli/tests/`
  can run in parallel only if tests are in clearly separate files
- `fixtures/`
  can run in parallel if fixture sets do not overlap
- `docs/`
  can usually run in parallel with code work, but avoid editing the same file at the same time

If two issues touch the same primary file, do not run them in parallel unless one is paused or explicitly rebased after the other lands.

## Recommended Parallel Split

Good parallel combinations:

- one agent on `protocol.rs` parsing rules
- one agent on `verify.rs` canonical or signature behavior
- one agent on fixture-backed negative tests
- one agent on docs / issue shaping / repo workflow

Bad parallel combinations:

- two agents both changing `protocol.rs`
- two agents both changing `verify.rs`
- one agent changing core behavior while another changes the same tests for a different reason

## Worktree and Session Model

Prefer one worktree or isolated session per active issue.

This keeps:

- local diffs smaller
- rebase simpler
- accidental cross-issue edits lower

Recommended mapping:

- one chat
- one issue
- one worktree or isolated branch state

## Commit and Push Discipline

For this repo, agents push directly to `origin/main`, so push order matters.

Rules:

1. commit only issue-local changes
2. push serially, not in parallel
3. re-check `origin/main` before pushing
4. if another commit landed first, fetch and rebase before retrying
5. do not mix another issue's files into the push

If a rebase reveals real overlap, stop and coordinate instead of guessing.

## Verification Rule

Every issue should have one short verification set.

Prefer:

- `cargo test -p mycel-core`
- `cargo test -p mycel-cli`
- fixture-backed validation commands
- simulator smoke checks where relevant

Do not hand off a task as "done" if the acceptance criteria and verify commands in the issue have not been checked.

## Spec Ambiguity Rule

If an issue runs into unclear protocol or profile semantics:

1. stop widening implementation
2. mark the issue or handoff with the ambiguity
3. use `blocked-by-spec` if code work should pause

Do not let one agent silently invent behavior that another agent will later have to unwind.

## Handoff Rule

When an agent stops or finishes, leave a short handoff:

- what changed
- what files were touched
- what verify commands passed
- what remains open
- whether another issue is now unblocked

Recommended handoff format:

- `Finished #4. Touched protocol.rs and object_verify_smoke.rs. Ran cargo test -p mycel-core and cargo test -p mycel-cli. Remaining follow-up: fixture-backed malformed snapshot cases.`

## Maintainer View

Maintainers should prefer:

- assigning one issue owner at a time
- checking file overlap before approving new parallel work
- moving ambiguous tasks back into spec/design discussion quickly
- keeping heavy tasks split into reviewable issue slices

## Practical Rule

If there is any doubt:

1. reduce scope
2. isolate files
3. verify locally
4. push in order
5. hand off clearly
