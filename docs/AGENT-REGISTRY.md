# Agent Registry Protocol

Status: active local-registry protocol for multi-agent coordination

This file is the active specification for the local registry in:

- `.agent-local/agents.json`

The current protocol uses a split identity model:

- `agent_uid`: the true agent identity, never reused
- `display_id`: the human-readable short id such as `coding-1`, which may be recycled

## Command Surface

Recommended startup and lifecycle commands:

- `scripts/agent_registry.py claim <role|auto> [--scope <scope>]`
- `scripts/agent_registry.py start <agent-ref>`
- `scripts/agent_registry.py status [<agent-ref>] [--verbose]`
- `scripts/agent_registry.py touch <agent-ref>`
- `scripts/agent_registry.py finish <agent-ref>`
- `scripts/agent_registry.py stop <agent-ref> [--status paused|done]`
- `scripts/agent_registry.py resume-check <agent-ref>`
- `scripts/agent_registry.py recover <agent-ref> [--scope <scope>]`
- `scripts/agent_registry.py takeover <stale-agent-ref> [--scope <scope>]`
- `scripts/agent_registry.py work-checklist <agent-ref> [--output .agent-local/agents/<agent_uid>/...md]`
- `scripts/agent_registry.py work-checklist-mark <agent-ref> <item-id> [--state checked|unchecked|toggle]`
- `scripts/agent_registry.py cleanup`

Transition note:

- write commands are now uid-first internally
- the current CLI still accepts either `agent_uid` or the current `display_id` as `<agent-ref>` during the transition
- once a stale agent has released its `display_id`, that old chat must use `agent_uid` to `resume-check` or `recover`

Recommended startup self-label:

- `<display-id> | <scope-label>`

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
- mailbox archive root: `.agent-local/mailboxes/archive/YYYY-MM/`

Fallback shared mailboxes such as `.agent-local/coding-to-doc.md` and `.agent-local/doc-to-coding.md` may still be used if the team explicitly wants them, but the registry remains the source of truth for role assignment.

Mailbox usage for `sync doc` / `sync web` / `sync plan` work:

- every agent should leave one mailbox handoff entry in its own declared mailbox before ending each completed work cycle; this is the per-cycle state record for that agent
- `coding` agents should leave sync-relevant notes in their own registry mailbox when work changes planning-relevant implementation state, checklist closure, roadmap emphasis, public progress wording, or issue-triage inputs
- `coding` agents should satisfy that per-cycle requirement with one open `Work Continuation Handoff` in their own mailbox, even when no planning-sync follow-up is needed
- `doc` agents should satisfy that per-cycle requirement with one mailbox handoff entry that captures the latest doc state for the cycle; use a planning-sync handoff, resolution reply, blocking note, or doc continuation note as appropriate
- before leaving a new open current-state handoff in the same mailbox, the agent should mark any older open current-state handoff for that scope as `superseded`, so the mailbox ends with one latest open current-state handoff; `python3 scripts/mailbox_handoff.py create ...` automates that supersede-and-append step
- `doc` should scan active, paused, and recently inactive agent mailboxes before any `sync doc`, `sync web`, or `sync plan` batch and use those notes as collection input for roadmap/checklist/progress or Pages refresh work
- scan order should be: active mailbox paths first, paused mailbox paths second, recently inactive mailbox paths third, and fallback shared mailboxes last; archived mailboxes stay out of scope unless a current mailbox explicitly points to an unresolved archived entry
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
- if an agent wants to render the tracked mailbox shapes directly, use `python3 scripts/mailbox_handoff.py create <agent-ref> <template> ...`

Mailbox retention and archive policy:

- registry cleanup does not delete mailbox files; mailbox history is retained until a mailbox-specific archive step moves it out of the active working set
- active working-set mailboxes stay in `.agent-local/mailboxes/`
- the tracked example mailbox stays in place and is never an archive candidate
- once an agent entry has been removed from `.agent-local/agents.json`, its uid-based mailbox becomes an orphaned mailbox candidate
- orphaned uid-based mailboxes should be moved into `.agent-local/mailboxes/archive/YYYY-MM/` instead of being deleted
- use `scripts/mailbox_gc.py scan` to inspect referenced, missing, orphaned, and archived uid-based mailboxes
- use `scripts/mailbox_gc.py archive` to move orphaned uid-based mailboxes into the archive tree without deleting their contents
- archived uid-based mailboxes older than 10 days may be deleted with `scripts/mailbox_gc.py prune` only when they do not contain an unresolved planning handoff
- shared fallback mailbox files outside `.agent-local/mailboxes/` are not touched by `scripts/mailbox_gc.py`; retire or archive those only with an explicit team decision

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

1. read `AGENTS.md`, `AGENTS-LOCAL.md` if it exists locally, and `docs/AGENT-REGISTRY.md`
2. run `git status -sb`
3. check `rg` and `gh`
4. if the role is `coding`, check the latest completed CI status from the previous push
5. if the user assigned a role, run `scripts/agent_registry.py claim <role> [--scope <scope>]`
6. otherwise run `scripts/agent_registry.py claim auto [--scope <scope>]`
7. immediately tell the user which role was claimed for this chat
8. run `scripts/agent_registry.py start <agent-ref>`
9. run `scripts/agent_registry.py status <agent-ref>`
10. if a personalized task list would help, run `scripts/agent_registry.py work-checklist <agent-ref>`
11. if the role is `coding`, run `npm run handoffs:inactive-coding` and treat handoff scan as the next item before taking a new implementation scope
12. begin the chat with `<display-id> | <scope-label>`
13. when the first concrete task arrives, run `scripts/agent_registry.py touch <agent-ref>`
14. before doing the work, prefer `scripts/agent_work_cycle.py begin <agent-ref> [--scope <scope-label>]`; it runs `touch` and prints the canonical `Asia/Taipei (UTC+8)` timestamp line, and that exact line must be surfaced in user-visible commentary rather than only terminal output

Keep startup output narrow:

- do not claim file-specific context before the user gives a concrete task
- do not run `claim`, `start`, and `status` in parallel
- when the role is `coding`, keep the CI line about the latest completed workflow, not a possibly in-progress run
- when the role is `coding`, treat `check handoffs` as the default next item after startup rather than an optional follow-up
- after `claim`, include a short user-facing role announcement before moving on to task work

## Workflow

1. before starting work, read `.agent-local/agents.json`
2. confirm the current scopes and active peers
3. use the mailbox declared in the registry for coordination
4. before each user-command work cycle, prefer `scripts/agent_work_cycle.py begin <agent-ref> [--scope <scope-label>]`; it wraps `scripts/agent_registry.py touch <agent-ref>` together with the canonical timestamp line, and that exact line should be visible in user-facing commentary
5. before ending that completed work cycle, append or update one mailbox handoff entry in the agent's declared mailbox so the latest state for the cycle is captured
6. after that command's work is complete, prefer `scripts/agent_work_cycle.py end <agent-ref> [--scope <scope-label>]`; it wraps `scripts/agent_registry.py finish <agent-ref>` together with the canonical timestamp line, and that exact line should be visible in user-facing commentary
7. do not immediately follow `scripts/agent_work_cycle.py begin|end` with a manual `scripts/agent_registry.py touch|finish` for the same work cycle; `begin` already performs `touch`, and `end` already performs `finish`
8. if you need only the timestamp line without the registry change, use `scripts/agent_timestamp.py before|after --agent <display-id> --scope <scope-label>` and keep the same single-line `UTC+8` format; do not hand-write or replace it with dual-timezone text
9. normal progress updates should not add hand-written date or time prefixes; reserve timestamps for the canonical before/after lines
10. when longer-lived coordination changes are needed, use `scripts/agent_registry.py stop <agent-ref> [--status paused|done]`
11. when an agent wants a refreshable task list, use `scripts/agent_registry.py work-checklist <agent-ref>`; by default it writes `.agent-local/agents/<agent_uid>/work-checklist.md` with Markdown `[X]` / `[ ]` items plus stable hidden `item-id` markers
12. to update one checklist line quickly from automation or a terminal command, use `scripts/agent_registry.py work-checklist-mark <agent-ref> <item-id>` and optionally `--state checked|unchecked|toggle`
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
6. once an entry has remained stale for at least 1 more hour, `scripts/agent_registry.py` removes it from `.agent-local/agents.json`
7. once an entry stays `paused` for at least 1 hour, it becomes stale-paused and releases its `display_id`
8. once an entry has remained `paused` for at least 2 hours total, `scripts/agent_registry.py` removes it from `.agent-local/agents.json`
9. `cleanup` reports both retained stale agents and removed agents

## Recovery Model

There are now two distinct flows:

### Self-Recovery

Use `recover` when the original chat returns and needs a new short slot.

Example:

- old chat A is still the same `agent_uid`
- A's `display_id` was already released after staleness
- A runs `scripts/agent_registry.py recover <agent_uid>`
- A keeps the same `agent_uid` but receives a new `display_id`

### Takeover

Use `takeover` when a different chat needs to continue a stale agent's work.

Example:

- old chat A is gone or should not resume directly
- a new chat B runs `scripts/agent_registry.py takeover <stale-agent-ref>`
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

Background terminal finished with python scripts/agent_registry.py status
Background terminal finished with gh run list --branch main --limit 1 --json databaseId,status,conclusion,workflowName,displayTitle,headSha,updatedAt
Background terminal finished with npm run handoffs:inactive-coding
Background terminal finished with python scripts/agent_registry.py takeover agt_example5678 --scope m4-snapshot-offer-sync
Background terminal finished with python scripts/agent_registry.py status agt_newagent1234
Background terminal finished with python scripts/agent_work_cycle.py begin agt_newagent1234 --scope m4-snapshot-offer-sync

[2026-03-12 15:20:00 UTC+8] Before work | coding-3 | m4-snapshot-offer-sync

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
