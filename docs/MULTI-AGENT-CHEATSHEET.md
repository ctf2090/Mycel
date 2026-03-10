# Multi-Agent Cheat Sheet

Status: draft

Use this as the short maintainer view of [MULTI-AGENT-COORDINATION.md](./MULTI-AGENT-COORDINATION.md).

## 10-Line Rule Set

1. Default to hybrid mode, not issue-for-everything.
2. Use one agent per issue when the work needs claims, handoff, or more than one commit.
3. One active issue should map to one chat and one worktree or isolated session.
4. Small local fixes can stay chat-first, but do not let them widen silently.
5. Claim the issue before editing, or leave a short local-scope note for chat-first work.
6. Do not run two agents on the same primary file at the same time.
7. Split work by file boundary, not by vague subtopic.
8. Verify with the commands named in the issue or local scope before handoff.
9. Push serially, never in parallel.
10. If `origin/main` moved, fetch and rebase before retrying. If the spec is unclear, stop and mark the task `blocked-by-spec`.

## Hybrid Rule

Use issue-first for:

- multi-commit work
- multi-file work
- bot-ready tasks
- anything that needs acceptance criteria or handoff

Use chat-first for:

- formatting-only follow-up
- tiny assertion alignment
- one-file typo or wording cleanup

If a chat-first fix expands, convert it into issue-first mode.

## Milestone Batch Done

A milestone batch is done only when:

1. batch scope is explicit
2. acceptance criteria are satisfied
3. named verify commands passed
4. latest relevant CI stayed green
5. a short handoff exists

Use this mini-template:

- Scope:
- Acceptance criteria:
- Verify commands:
- CI status:
- Remaining follow-up:

## Fast Triage

Good parallel split:

- one agent on `protocol.rs`
- one agent on `verify.rs`
- one agent on fixture-backed or simulator-backed tests
- one agent on docs / issue shaping / workflow maintenance

Bad parallel split:

- two agents both changing `protocol.rs`
- two agents both changing `verify.rs`
- one agent changing core behavior while another edits the same tests for a different reason

## Required Handoff

Every handoff should say:

- which issue was worked
- which files changed
- which verify commands passed
- what remains open

Recommended format:

- `Finished #4. Touched protocol.rs and object_verify_smoke.rs. Ran cargo test -p mycel-core and cargo test -p mycel-cli. Remaining follow-up: malformed snapshot fixtures.`
- `Finished local CI-fix follow-up. Touched protocol.rs. Ran cargo fmt --all and cargo test --workspace. Remaining follow-up: none.`
