# Agent Registry Protocol

Status: active local-registry protocol for multi-agent coordination

This file is the active specification for the local registry in:

- `.agent-local/agents.json`

The current protocol uses a split identity model:

- `agent_uid`: the true agent identity, never reused
- `display_id`: the human-readable short id such as `coding-1`, which may be recycled

## Command Surface

Recommended startup and lifecycle tools:

- `scripts/agent_bootstrap.py` for the repo-standard bootstrap flow
- `scripts/agent_registry.py` for role claim, startup confirmation, lifecycle state, recovery, takeover, cleanup, and checklist management
- `scripts/agent_work_cycle.py` for tracked per-command activity transitions
- `scripts/agent_timestamp.py` for canonical timestamp lines when no registry transition is needed

Transition note:

- write commands are now uid-first internally
- the current CLI still accepts either `agent_uid` or the current `display_id` as `<agent-ref>` during the transition
- once a stale agent has released its `display_id`, that old chat must use `agent_uid` to `resume-check` or `recover`

Recommended startup self-label:

- `<display-id> | <scope-label>`

Fast path:

- `scripts/agent_bootstrap.py` is the preferred thin wrapper when a new chat wants the repo-standard claim/start/work-cycle bootstrap in one call.
- The wrapper does not replace reading [`AGENTS.md`](../AGENTS.md) or local overlays first; it only reduces command round-trips after those inputs are loaded.
- Use this default 5-step startup sequence for a fresh chat unless recovery, takeover, or a direct user request needs more upfront context:
  1. scan the repo root with `ls`
  2. read `AGENTS-LOCAL.md` if it exists, then read `.agent-local/dev-setup-status.md`
  3. read [`docs/ROLE-CHECKLISTS/README.md`](./ROLE-CHECKLISTS/README.md), then inspect [`docs/AGENT-REGISTRY.md`](./AGENT-REGISTRY.md) and `.agent-local/agents.json`
  4. run `scripts/agent_bootstrap.py <role>` or `scripts/agent_bootstrap.py auto`
  5. if the claimed role is `coding`, check the latest completed CI result for the previous push before starting implementation
- Defer broader reading until task work begins:
  - `coding`: postpone `ROADMAP.md`, wide mailbox scans, and broad repo markdown sweeps until the actual implementation slice needs them
  - `doc`: postpone planning-sync mailbox scans, `scripts/check-plan-refresh.sh`, and broad roadmap/checklist refresh reading until the doc work item actually starts

Role checklist sources:

- before starting role-specific checklist work, read [`docs/ROLE-CHECKLISTS/README.md`](./ROLE-CHECKLISTS/README.md)
- canonical role checklist sources live in [`docs/ROLE-CHECKLISTS/coding.md`](./ROLE-CHECKLISTS/coding.md) and [`docs/ROLE-CHECKLISTS/doc.md`](./ROLE-CHECKLISTS/doc.md)
- per-agent checklist copies should live under `.agent-local/agents/<agent_uid>/checklists/`
- current role checklist section names should align with `New chat bootstrap` and `Work Cycle Workflow`

## Role Model

The system supports multiple concurrent agents, not just one `coding` and one `doc`.

Allowed `role` values:

- `coding`
- `doc`

Role responsibilities:

- `coding`
  owns issue resolution, feature work, local verification, commit/push flow, and CI checks after each push; when work may affect planning surfaces, this role hands the relevant material to `doc` through the registry mailbox and does not run `scripts/check-plan-refresh.sh`
- `doc`
  owns design-note sync, roadmap/checklist refresh, explanatory docs, planning-surface wording, and the `scripts/check-plan-refresh.sh` cadence check; this role must run that script after each completed doc work item while preparing next items, scans registry mailboxes to collect sync-relevant handoff material, and does not check CI

If the user does not assign any role in a new chat, `claim auto` should choose:

- `coding` if there is no active `coding`
- `doc` if active `coding >= 1` and active `doc == 0`
- `coding` if active `coding >= 1` and active `doc >= 1`

## Registry Shape

The local registry must be valid JSON and use this top-level shape:

```json
{
  "version": 2,
  "updated_at": "2026-03-12T12:00:00+0800",
  "agent_count": 2,
  "agents": [
    {
      "agent_uid": "agt_a1b2c3d4",
      "role": "coding",
      "current_display_id": "coding-1",
      "display_history": [
        {
          "display_id": "coding-1",
          "assigned_at": "2026-03-12T11:00:00+0800",
          "released_at": null,
          "released_reason": null
        }
      ],
      "assigned_by": "user",
      "assigned_at": "2026-03-12T11:00:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-12T11:01:00+0800",
      "last_touched_at": "2026-03-12T11:10:00+0800",
      "inactive_at": null,
      "paused_at": null,
      "status": "active",
      "scope": "forum inbox sync",
      "files": [],
      "mailbox": ".agent-local/mailboxes/agt_a1b2c3d4.md",
      "recovery_of": null,
      "superseded_by": null
    },
    {
      "agent_uid": "agt_e5f6g7h8",
      "role": "doc",
      "current_display_id": "doc-1",
      "display_history": [
        {
          "display_id": "doc-1",
          "assigned_at": "2026-03-12T11:05:00+0800",
          "released_at": null,
          "released_reason": null
        }
      ],
      "assigned_by": "user",
      "assigned_at": "2026-03-12T11:05:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-12T11:06:00+0800",
      "last_touched_at": "2026-03-12T11:15:00+0800",
      "inactive_at": null,
      "paused_at": null,
      "status": "active",
      "scope": "registry design note",
      "files": [],
      "mailbox": ".agent-local/mailboxes/agt_e5f6g7h8.md",
      "recovery_of": null,
      "superseded_by": null
    }
  ]
}
```

### Required Fields

Top level:

- `version`
- `updated_at`
- `agent_count`
- `agents`

Per agent:

- `agent_uid`
- `role`
- `current_display_id`
- `display_history`
- `assigned_by`
- `assigned_at`
- `confirmed_by_agent`
- `confirmed_at`
- `last_touched_at`
- `inactive_at`
- `paused_at`
- `status`
- `scope`
- `files`
- `mailbox`
- `recovery_of`
- `superseded_by`

Allowed `status` values:

- `active`
- `inactive`
- `paused`
- `blocked`
- `done`

Field rules:

- `agent_uid` is the registry primary key
- `current_display_id` may be `null`
- if `current_display_id != null`, it must be unique across all agents
- `display_history` must be ordered by time, and its latest record represents the most recent slot assignment
- mailbox paths should be uid-based, not display-id-based
- `agent_count` must match the number of entries in `agents`
- `confirmed_by_agent` must be `true` before tracked work starts
- `inactive_at` should be non-null only when `status == "inactive"`
- `paused_at` should be non-null only when `status == "paused"`

`scripts/agent_registry.py` writes timestamps in `Asia/Taipei (UTC+8)` using the `+0800` offset form.

## Identity Model

`agent_uid` and `display_id` do different jobs:

- `agent_uid` is the stable identity for the chat
- `display_id` is the current short slot shown to humans

Practical implications:

- `display_id` may be recycled after stale slot release
- `agent_uid` must not be recycled
- a reopened old chat is the same agent only if it still has the same `agent_uid`
- a different chat taking over the work must get a fresh `agent_uid`

## Mailbox Rule

The registry tells agents who exists. Mailboxes carry the actual messages.

Recommended mailbox pattern:

- `.agent-local/mailboxes/<agent_uid>.md`
- copyable planning-sync example: `.agent-local/mailboxes/EXAMPLE-planning-sync-handoff.md`
- copyable planning-sync resolution example: `.agent-local/mailboxes/EXAMPLE-planning-sync-resolution.md`
- copyable work-continuation example: `.agent-local/mailboxes/EXAMPLE-work-continuation-handoff.md`
- copyable doc-continuation example: `.agent-local/mailboxes/EXAMPLE-doc-continuation-note.md`

Fallback shared mailboxes such as `.agent-local/coding-to-doc.md` and `.agent-local/doc-to-coding.md` may still be used if the team explicitly wants them, but the registry remains the source of truth for role assignment.

Mailbox usage for `sync doc` / `sync web` / `sync plan` work:

- every agent should leave one mailbox handoff entry in its own declared mailbox before ending each completed work cycle; this is the per-cycle state record for that agent
- `coding` agents should leave sync-relevant notes in their own registry mailbox when work changes planning-relevant implementation state, checklist closure, roadmap emphasis, public progress wording, or issue-triage inputs
- `coding` agents should satisfy that per-cycle requirement with one open `Work Continuation Handoff` in their own mailbox, even when no planning-sync follow-up is needed
- `doc` agents should satisfy that per-cycle requirement with one mailbox handoff entry that captures the latest doc state for the cycle; use a planning-sync handoff, resolution reply, blocking note, or doc continuation note as appropriate
- before leaving a new open current-state handoff in the same mailbox, the agent should mark any older open current-state handoff for that scope as `superseded`, so the mailbox ends with one latest open current-state handoff; `python3 scripts/mailbox_handoff.py create ...` automates that supersede-and-append step
- `doc` should scan active, paused, and recently inactive agent mailboxes before any `sync doc`, `sync web`, or `sync plan` batch and use those notes as collection input for roadmap/checklist/progress or Pages refresh work
- scan order should be: active mailbox paths first, paused mailbox paths second, recently inactive mailbox paths third, and fallback shared mailboxes last
- mailbox handoff is the default coordination path for planning-sync material; `coding` should not replace it by running `scripts/check-plan-refresh.sh`

Mailbox usage for resumed or takeover coding work:

- a new or resumed `coding` agent should read the newest open `Work Continuation Handoff` entry for the overlapping scope before starting implementation
- continuation handoffs are not doc-only; they are the default recovery surface for another coding agent that must continue the last landed or partially-finished slice

Recommended mailbox handoff template:

```md
## Planning Sync Handoff

- Status: open
- Date: 2026-03-12 11:30 UTC+8
- Source agent: coding-2
- Scope: accepted-head render editor admission
- Files changed:
  - apps/mycel-cli/src/head.rs
  - apps/mycel-cli/tests/head_inspect_smoke.rs
- Planning impact:
  - roadmap wording update needed
  - progress summary update needed
- Checklist impact:
  - no checkbox change
  - narrow status wording should mention editor-admission-aware inspect/render flows
- Issue impact:
  - no issue change
- Verification:
  - cargo test -p mycel-cli head_inspect
- Notes:
  - named-profile and store-backed render paths now apply editor admission consistently
```

Minimum handoff quality:

- each completed work cycle should leave one new or updated mailbox handoff entry in the agent's declared mailbox
- planning-sync handoff entries should explicitly include `Status: open` when `coding` creates them
- include enough detail for `doc` to identify the affected files, the likely planning surfaces, whether checklist closure changed, and what verification or evidence supports the claim
- after `doc` completes the related docs work, it should either update that handoff to `Status: resolved` or append a `doc` reply entry with a `Date` line that makes the resolution explicit
- if an agent wants a ready-made starting point instead of copying the Markdown block manually, use `.agent-local/mailboxes/EXAMPLE-planning-sync-handoff.md` for the open handoff and `.agent-local/mailboxes/EXAMPLE-planning-sync-resolution.md` for the resolved reply
- continuation handoffs should explicitly include `Status: open`, `Current state`, and `Next suggested step`, because they are written under the assumption that the user may not assign another follow-up before pause or takeover
- a mailbox should not accumulate multiple open current-state handoffs for the same scope; older ones should be closed as `superseded` before a newer open current-state handoff is added
- if an agent wants a ready-made starting point for continuation instead of copying the Markdown block manually, use `.agent-local/mailboxes/EXAMPLE-work-continuation-handoff.md`
- if an agent wants to render the tracked mailbox shapes directly, use `scripts/mailbox_handoff.py`

Mailbox retention and cleanup policy:

- active working-set mailboxes stay in `.agent-local/mailboxes/`
- the tracked example mailbox stays in place and is never a cleanup candidate
- once an agent entry has been removed from `.agent-local/agents.json`, its uid-based mailbox becomes an orphaned mailbox candidate
- orphaned uid-based mailboxes older than 3 days should be deleted; there is no archive step
- use `scripts/mailbox_gc.py` to inspect mailbox references and delete orphaned uid-based mailboxes after the retention window
- shared fallback mailbox files should stay small; each shared fallback file is limited to `1024` bytes
- shared fallback mailbox files outside `.agent-local/mailboxes/` are not touched by `scripts/mailbox_gc.py`; remove those only with an explicit team decision

## Startup Gate

No agent may start tracked work until all of the following are true:

1. the entry exists
2. the entry has a valid `agent_uid`
3. the entry has a valid `role`
4. the entry has `assigned_by`, `assigned_at`, `scope`, and `mailbox`
5. the agent has set `confirmed_by_agent = true`
6. the entry has a non-null `confirmed_at`
7. the entry still has a `current_display_id`

Recommended startup sequence:

1. read `AGENTS.md`, `AGENTS-LOCAL.md` if it exists locally, and `.agent-local/dev-setup-status.md`
2. scan the repo root with `ls`
3. read `docs/ROLE-CHECKLISTS/README.md`, then read `docs/AGENT-REGISTRY.md` and `.agent-local/agents.json`
4. if the user assigned a role, prefer `scripts/agent_bootstrap.py <role>`; otherwise prefer `scripts/agent_bootstrap.py auto`
5. immediately tell the user which role was claimed for this chat
6. begin the chat with `<display-id> | <scope-label>`
7. when the first concrete task arrives, use the work-cycle tool to begin tracked work
8. before implementation work, if the role is `coding`, check the latest completed CI status from the previous push
9. only when the scope overlaps existing coding work, recovery, or takeover, run `npm run handoffs:inactive-coding` and inspect the relevant mailbox before continuing
10. if a personalized task list would help, use the registry tool to materialize one after the bootstrap flow is complete

Keep startup output narrow:

- do not claim file-specific context before the user gives a concrete task
- do not run `claim`, `start`, and `status` in parallel
- when the role is `coding`, keep the CI line about the latest completed workflow, not a possibly in-progress run
- when the role is `coding`, treat `check handoffs` as task-start context for overlapping or resumed work, not a default bootstrap-time read
- after `claim`, include a short user-facing role announcement before moving on to task work

## Workflow

1. before starting work, read `.agent-local/agents.json`
2. confirm the current scopes and active peers
3. use the mailbox declared in the registry for coordination
4. before each user-command work cycle, prefer `scripts/agent_work_cycle.py`; it advances the work cycle together with the canonical timestamp line, and that exact line should be visible in user-facing commentary
5. before ending that completed work cycle, append or update one mailbox handoff entry in the agent's declared mailbox so the latest state for the cycle is captured
6. after that command's work is complete, prefer `scripts/agent_work_cycle.py`; it closes the work cycle together with the canonical timestamp line, and that exact line should be visible in user-facing commentary
7. do not immediately follow `scripts/agent_work_cycle.py` with a separate manual registry lifecycle step for the same work cycle
8. if you need only the timestamp line without the registry change, use `scripts/agent_timestamp.py` and paste the emitted line directly; do not hand-write, reformat, or replace it with dual-timezone text
9. normal progress updates should not add hand-written date or time prefixes; reserve timestamps for the canonical before/after lines
10. when longer-lived coordination changes are needed, use the registry tool to pause or stop the tracked agent state
11. when an agent wants a refreshable task list, use the registry tool to materialize one; by default it writes `.agent-local/agents/<agent_uid>/work-checklist.md` with Markdown `[X]` / `[ ]` items plus stable hidden `item-id` markers
12. to update one checklist line quickly from automation or a terminal command, use the registry tool's checklist-marking support
13. treat `paused` as a medium-term parking state, not an indefinite one; if the work should live longer than the paused lease, plan for a later `takeover` or close it as `done`

Planning-sync coordination:

- `coding` agents should append mailbox handoff notes when they land or discover planning-relevant changes
- every role should leave one mailbox handoff entry per completed work cycle so `doc` or a takeover agent can reconstruct the latest state without rereading the full diff first
- `doc` owns `scripts/check-plan-refresh.sh` and the decision to start a planning-sync batch
- after each completed doc work item, while preparing next items, `doc` must run `scripts/check-plan-refresh.sh`; if it reports `due`, add the due sync surfaces to the next items and then scan the declared mailboxes for recent handoff material before any `sync doc`, `sync web`, or `sync plan` batch
- if a mailbox note follows the recommended template, `doc` may treat it as ready-to-triage input instead of re-deriving the whole change from git history first

If two `coding` agents would touch the same primary file or issue, one must narrow scope or pause before proceeding.

## Activity Lease

The registry uses a per-command activity lease.

Rules:

1. `touch` marks the current command cycle as active
2. `finish` marks the role inactive for that command cycle
3. once an entry stays `inactive` for at least one hour, it becomes stale
4. when an entry becomes stale, its `display_id` is released and `current_display_id` becomes `null`
5. while the stale entry is still retained, the old chat must use `resume-check` and then `recover` by `agent_uid`
6. once an entry has remained `inactive` for 3 days, `scripts/agent_registry.py` removes it from `.agent-local/agents.json` and deletes that agent's local mailbox and agent directory
7. once an entry stays `paused` for at least 1 hour, it becomes stale-paused and releases its `display_id`
8. once an entry has remained `paused` for 3 days, `scripts/agent_registry.py` removes it from `.agent-local/agents.json` and deletes that agent's local mailbox and agent directory
9. `cleanup` reports both retained stale agents and removed agents

## Recovery Model

There are now two distinct flows:

### Self-Recovery

Use `recover` when the original chat returns and needs a new short slot.

Example:

- old chat A is still the same `agent_uid`
- A's `display_id` was already released after staleness
- A uses the registry tool to recover the stale identity
- A keeps the same `agent_uid` but receives a new `display_id`

### Takeover

Use `takeover` when a different chat needs to continue a stale agent's work.

Example:

- old chat A is gone or should not resume directly
- a new chat B uses the registry tool to take over the stale scope
- B gets a fresh `agent_uid`
- B gets a fresh `display_id`
- old A records `superseded_by = <new-agent-uid>`
- new B records `recovery_of = <old-agent-uid>`

Copyable takeover transcript:

```text
Please take over the existing handoff.

Repo status: `## main...origin/main`

Short plan:
1. Check the latest completed CI result for `main`.
2. Scan leftover inactive-coding continuation handoffs and choose the takeover target.
3. Run `takeover`, read the source mailbox, and begin the work cycle for the resumed scope.

Background terminal finished with the registry tool to inspect current state.
Background terminal finished with gh run list --branch main --limit 1 --json databaseId,status,conclusion,workflowName,displayTitle,headSha,updatedAt
Background terminal finished with npm run handoffs:inactive-coding
Background terminal finished with the registry tool to take over the stale scope and confirm the replacement agent.
Background terminal finished with the work-cycle tool for the resumed command.

<paste the exact before-work line emitted by `scripts/agent_work_cycle.py` here>

Please take over the existing handoff.

我已檢查 `main` 的最新 completed CI，並用 `npm run handoffs:inactive-coding` 掃描遺留 handoff。這個 chat 已透過 `takeover` 接手 `coding-4`（`agt_example5678`）留下的 `m4-snapshot-offer-sync` scope，新的 agent 是 `coding-3`（`agt_newagent1234`）。接下來我會先讀來源 mailbox 的最新 open `Work Continuation Handoff`，再從那個切片繼續實作。
```

## Resume Rules

`resume-check <agent-ref>` should be interpreted like this:

1. if the agent is `active` or `inactive`, still confirmed, and still has `current_display_id`, it may directly resume
2. if the agent is confirmed but `current_display_id == null`, it must recover before doing tracked work
3. if the agent is `paused`, `blocked`, or `done`, it must stop instead of resuming
4. a stale-paused entry should normally be continued through `takeover`, not direct resume
5. if the stale entry has already been removed after the retention window, the old chat must not continue under that old identity

## Transitional CLI Policy

The repo is in the transition from display-id-first commands to uid-first commands.

Current CLI policy:

- write commands should prefer `agent_uid`
- read commands may use either `agent_uid` or the current `display_id`
- display-id references only work while that slot is still current
- once a slot has been released, only `agent_uid` can address that stale agent

## Minimal Example

```json
{
  "version": 2,
  "updated_at": "2026-03-12T12:00:00+0800",
  "agent_count": 2,
  "agents": [
    {
      "agent_uid": "agt_a1b2c3d4",
      "role": "coding",
      "current_display_id": "coding-1",
      "display_history": [
        {
          "display_id": "coding-1",
          "assigned_at": "2026-03-12T11:00:00+0800",
          "released_at": null,
          "released_reason": null
        }
      ],
      "assigned_by": "maintainer",
      "assigned_at": "2026-03-12T11:00:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-12T11:02:00+0800",
      "last_touched_at": "2026-03-12T11:10:00+0800",
      "inactive_at": null,
      "paused_at": null,
      "status": "active",
      "scope": "#17 store refactor",
      "files": [
        "apps/mycel-cli/src/store.rs",
        "apps/mycel-cli/src/store/index.rs"
      ],
      "mailbox": ".agent-local/mailboxes/agt_a1b2c3d4.md",
      "recovery_of": null,
      "superseded_by": null
    },
    {
      "agent_uid": "agt_e5f6g7h8",
      "role": "doc",
      "current_display_id": "doc-1",
      "display_history": [
        {
          "display_id": "doc-1",
          "assigned_at": "2026-03-12T11:05:00+0800",
          "released_at": null,
          "released_reason": null
        }
      ],
      "assigned_by": "maintainer",
      "assigned_at": "2026-03-12T11:05:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-12T11:06:00+0800",
      "last_touched_at": "2026-03-12T11:15:00+0800",
      "inactive_at": null,
      "paused_at": null,
      "status": "active",
      "scope": "planning sync for #17",
      "files": [
        "ROADMAP.md",
        "IMPLEMENTATION-CHECKLIST.en.md"
      ],
      "mailbox": ".agent-local/mailboxes/agt_e5f6g7h8.md",
      "recovery_of": null,
      "superseded_by": null
    }
  ]
}
```
