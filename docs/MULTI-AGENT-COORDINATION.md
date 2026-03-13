# Multi-Agent Coordination Note

Status: draft

This note describes how multiple agents with `coding` or `doc` roles should work in parallel in the Mycel repository without colliding on scope, files, push order, or handoff flow.

For the higher-level operating model that connects planning, issue intake, execution, verification, and human control, see [AI-CO-WORKING-MODEL.md](./AI-CO-WORKING-MODEL.md).

For the short maintainer version, see [MULTI-AGENT-CHEATSHEET.md](./MULTI-AGENT-CHEATSHEET.md).

Use it together with:

- [AGENT-REGISTRY.md](./AGENT-REGISTRY.md)
- [AI-CO-WORKING-MODEL.md](./AI-CO-WORKING-MODEL.md)
- [AGENT-HANDOFF.md](./AGENT-HANDOFF.md)
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

## Agent Roles

Use role-based agent modes:

- `coding`
  owns issue resolution, feature work, local verification, commit and push flow, and CI checks after each push
- `doc`
  owns `sync doc` / `sync plan` work, design notes, roadmap or checklist updates, and planning-surface wording; this role does not check CI by default

Use `coding` when the main output is behavior, tests, fixtures, parser or verifier changes, CLI changes, or any landed feature slice.

Use `doc` when the main output is syncing planning surfaces or explanatory docs after code or design work has already clarified the intended behavior.

If a task starts as doc-only but requires new implementation decisions, stop and hand the open question back to `coding` or a maintainer instead of inventing behavior in docs.

Use `.agent-local/agents.json` as the local registry file for active agent count and role assignment. The tracked definition for that file lives in [AGENT-REGISTRY.md](./AGENT-REGISTRY.md).

## Identity Model

The active registry model is now:

- `agent_uid`: the stable identity for the chat, never reused
- `display_id`: the short human-facing id such as `coding-1`, recyclable after stale slot release

Operational consequences:

- write commands should prefer `agent_uid`
- the transitional CLI still accepts either `agent_uid` or the current `display_id` as `<agent-ref>`
- once a stale agent has released its `display_id`, reopened chats must use `agent_uid` for `resume-check` and `recover`
- a different chat taking over the stale work must not reuse the old `agent_uid`

No agent may start tracked work until it has confirmed its own assigned role in `.agent-local/agents.json`.

Recommended startup command:

- `scripts/agent_registry.py claim <role|auto> [--scope <scope>]`
- `scripts/agent_registry.py start <agent-ref>`
- `scripts/agent_registry.py touch <agent-ref>`
- `scripts/agent_registry.py finish <agent-ref>`
- `scripts/agent_registry.py status [<agent-ref>]`
- `scripts/agent_registry.py stop <agent-ref> [--status paused|done]`
- `scripts/agent_registry.py cleanup`
- `scripts/agent_registry.py resume-check <agent-ref>`
- `scripts/agent_registry.py recover <agent-ref> [--scope <scope>]`
- `scripts/agent_registry.py takeover <stale-agent-ref> [--scope <scope>]`
- `scripts/agent_registry.py work-checklist <agent-ref> [--output .agent-local/checklists/...md]`
- `scripts/agent_registry.py work-checklist-mark <agent-ref> <item-id> [--state checked|unchecked|toggle]`

Recommended startup self-label:

- first chat line: `<display-id> | <scope-label>`

## Hybrid Issue Mode

Do not force every coding action through a GitHub issue first.

Use a hybrid mode:

- issue-first for scoped feature work, bot-ready tasks, multi-commit work, or anything another agent may need to pick up
- chat-first for tiny fixes such as formatting-only changes, one-line assertion updates, or other obviously local cleanup

Recommended issue-first triggers:

1. the task will likely take more than one commit
2. the task touches more than one primary file
3. the task is intended for handoff to another agent
4. the task changes roadmap or checklist meaning
5. the task is large enough to deserve acceptance criteria and verify commands

Recommended chat-first exceptions:

1. formatting-only follow-up after a failed CI run
2. narrow test assertion alignment after a behavior-preserving refactor
3. trivial doc wording or typo cleanup in one file

If a chat-first fix grows beyond that boundary, convert it into issue-first mode before widening scope.

Practical default:

- if the work needs a claim, handoff, labels, or batching, use a GitHub issue
- if the work is obviously one short local correction, it can stay issue-free

## Claiming Work

Before an agent starts:

1. read `.agent-local/agents.json`
2. if the user already declared a role but no entry exists, run `scripts/agent_registry.py claim <role>`
3. confirm the entry has `agent_uid`, `role`, `assigned_by`, `assigned_at`, and `current_display_id`
4. run `scripts/agent_registry.py start <agent-ref>`
5. only then decide whether the task is issue-first or chat-first
6. if it is issue-first, choose one open issue
7. check whether another agent or human is already working on it
8. leave a short claim note in the issue or team channel
9. confirm the likely file set before editing
10. prefer `scripts/agent_work_cycle.py begin <agent-ref> [--scope <scope-label>]` before working the current user-command cycle; it wraps `touch` together with the canonical before-work timestamp line, and that exact line should be surfaced in user-visible commentary
11. update the local registry entry when scope or status changes
12. prefer `scripts/agent_work_cycle.py end <agent-ref> [--scope <scope-label>]` after the command-level work is complete; it wraps `finish` together with the canonical after-work timestamp line, and that exact line should be surfaced in user-visible commentary
13. do not immediately follow `scripts/agent_work_cycle.py begin|end` with a manual `scripts/agent_registry.py touch|finish` for the same work cycle
14. use `scripts/agent_timestamp.py before|after --agent <display-id> --scope <scope-label>` only when you need the timestamp line without the registry transition, and keep the same single-line `UTC+8` format
15. normal progress updates should not add hand-written date or time prefixes; reserve timestamps for the canonical before/after lines

When the chat itself starts, use one short self-label line first, such as:

- `coding-2 | forum-design-note-sync`
- `doc-1 | planning-sync-for-m1`

Recommended claim format:

- `Claiming #5 for protocol/parser work in protocol.rs plus direct tests.`

Recommended chat-first start note:

- `Taking a local follow-up fix for the latest formatting failure in protocol.rs only.`

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

For role-specific responsibility:

- `coding` runs the relevant local verification and checks the latest CI result from the previous push before starting new work
- `doc` verifies document coherence locally as needed, but does not own CI monitoring unless a maintainer explicitly asks for it

## Milestone Batch Completion Gate

Do not treat a milestone batch as complete just because several related issues landed.

A milestone batch is complete only when all of the following are true:

1. the intended scope for the batch is explicit
2. the matching issue acceptance criteria are satisfied
3. the related roadmap or checklist items can be closed or narrowed clearly
4. the named verify commands for the batch have passed
5. no new CI failure was introduced by the batch
6. a short handoff exists for the next agent or maintainer

Recommended completion template:

- Scope:
  one short sentence describing what this batch was supposed to close
- Acceptance criteria:
  the issue or issue set used to define done
- Verify commands:
  the exact commands that were run
- CI status:
  whether the latest relevant workflow stayed green
- Remaining follow-up:
  what is still open after this batch

Recommended maintainer check:

1. compare the landed commits against the issue scope, not against intent alone
2. re-run the batch verification commands if the result is unclear
3. update `ROADMAP.md` and `IMPLEMENTATION-CHECKLIST.*` if the batch meaningfully changed milestone status
4. only then mark the batch complete in docs, issue tracking, or handoff notes

## Spec Ambiguity Rule

If an issue runs into unclear protocol or profile semantics:

1. stop widening implementation
2. mark the issue or handoff with the ambiguity
3. use `blocked-by-spec` if code work should pause

Do not let one agent silently invent behavior that another agent will later have to unwind.

## Interrupted Chat Recovery

If a chat stops unexpectedly, use the local registry and mailbox files as the recovery surface instead of relying on the lost chat state.

Recovery sequence:

1. inspect `.agent-local/agents.json`
2. run `scripts/agent_registry.py status`
3. read the stale agent's mailbox, starting from the newest open `Work Continuation Handoff`
4. if the original chat itself has returned, run `scripts/agent_registry.py resume-check <agent_uid>` and `scripts/agent_registry.py recover <agent_uid>` if needed
5. if a different chat must continue the work, run `scripts/agent_registry.py takeover <stale-agent-ref>`
6. continue work only under the currently assigned `display_id`

Inactive lease rule:

1. `touch` before each user-command work cycle
2. `finish` when that command completes
3. inactive entries older than one hour become stale and release their `display_id`
4. stale entries remain recoverable by `agent_uid` during the retention window
5. once an inactive stale entry remains retained for 24 hours, `scripts/agent_registry.py` should remove it from `.agent-local/agents.json`
6. paused entries older than 24 hours become stale-paused and release their `display_id`
7. once a paused entry remains older than 3 days, `scripts/agent_registry.py` should remove it from `.agent-local/agents.json`
8. `scripts/agent_registry.py cleanup` can be used to report both retained stale agents and removed agents

Do not silently reuse the old `agent_uid` for a new chat after an interruption.

## Handoff Rule

When an agent stops or finishes, leave a short handoff:

- what changed
- what files were touched
- what verify commands passed
- what remains open
- whether another issue is now unblocked

For `coding`, this is not optional. At the end of every completed coding work item, leave one open `Work Continuation Handoff` in the coding mailbox and assume the current user task may be the last one assigned before pause, interruption, or takeover.

Before adding that new open continuation entry, close any older open `Work Continuation Handoff` entries in the same mailbox by marking them `superseded`.

Recommended handoff format:

- `Finished #4. Touched protocol.rs and object_verify_smoke.rs. Ran cargo test -p mycel-core and cargo test -p mycel-cli. Remaining follow-up: fixture-backed malformed snapshot cases.`

For chat-first work with no issue, still leave the same handoff structure, but replace the issue reference with a short scope label.

The continuation handoff should always include:

- current state
- next suggested step
- blockers
- last landed commit when one exists

Example:

- `Finished local CI-fix follow-up. Touched protocol.rs. Ran cargo fmt --all and cargo test --workspace. Remaining follow-up: none.`

When `coding` hands work to `doc`, use a real-time handoff that is structured enough for `sync doc` / `sync plan` work without rereading the full diff.

Default repo-local handoff surface:

- use per-agent mailbox files declared in `.agent-local/agents.json`, preferably `.agent-local/mailboxes/<agent_uid>.md`
- shared directional files such as `.agent-local/coding-to-doc.md` and `.agent-local/doc-to-coding.md` are fallback paths only
- use [AGENT-HANDOFF.md](./AGENT-HANDOFF.md) only as the tracked protocol and template reference
- if the work is issue-first, mirror the same summary in the issue comment when useful
