# Multi-Agent Cheat Sheet

Status: draft

Use this as the short maintainer view of [MULTI-AGENT-COORDINATION.md](./MULTI-AGENT-COORDINATION.md).

## 10-Line Rule Set

1. One agent owns one issue at a time.
2. One active issue should map to one chat and one worktree or isolated session.
3. Claim the issue before editing.
4. Do not run two agents on the same primary file at the same time.
5. Split work by file boundary, not by vague subtopic.
6. Keep each diff issue-local and reviewable.
7. Verify with the commands named in the issue before handoff.
8. Push serially, never in parallel.
9. If `origin/main` moved, fetch and rebase before retrying.
10. If the spec is unclear, stop and mark the task `blocked-by-spec`.

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
