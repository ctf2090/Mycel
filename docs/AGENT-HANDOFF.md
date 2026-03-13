# Agent Handoff Protocol

Status: active local-mailbox protocol for multi-agent coordination

Use this file as the tracked specification for how agents communicate through local gitignored mailboxes.

For agent discovery and role lookup, read [AGENT-REGISTRY.md](./AGENT-REGISTRY.md) and the local `.agent-local/agents.json` file first. Do not use mailbox traffic as a substitute for registry assignment confirmation.

Live mailbox files are local and not committed. Each agent should use the mailbox path declared in `.agent-local/agents.json`.

Use `python3 scripts/mailbox_handoff.py create ...` when you want the tool to render a tracked mailbox template for you. For open current-state entries, the tool appends the new entry and automatically marks older `Status: open` entries in that mailbox as `superseded`.

The directory is ignored by git through `.gitignore`, except for tracked template examples such as `.agent-local/mailboxes/EXAMPLE-planning-sync-handoff.md`, `.agent-local/mailboxes/EXAMPLE-planning-sync-resolution.md`, `.agent-local/mailboxes/EXAMPLE-work-continuation-handoff.md`, and `.agent-local/mailboxes/EXAMPLE-doc-continuation-note.md`.

## Modes

Use role-based execution modes:

- `coding`
  resolves issues, implements features, runs local verification, commits and pushes, and checks CI after each push
- `doc`
  syncs design notes, roadmap/checklist surfaces, and explanatory docs; this mode does not check CI

Use `coding` when the main output is behavior, tests, fixtures, parser/verifier work, or CLI changes.

Use `doc` when the main output is `sync doc` or `sync plan` work after implementation or accepted design direction already exists.

Multiple agents may use the same role if their scopes and file ownership do not overlap.

## Mailbox Files

Default pattern:

- `.agent-local/mailboxes/<agent_uid>.md`
  one mailbox per agent, with peer-to-peer handoff entries addressed by scope and stable uid

Archive pattern:

- `.agent-local/mailboxes/archive/YYYY-MM/<agent_uid>.md`
  archive destination for orphaned uid-based mailboxes that are no longer referenced by `.agent-local/agents.json`

Shared fallback pattern:

- `.agent-local/coding-to-doc.md`
  implementation handoff, doc-impact notes, planning-surface follow-up, and unresolved doc requests
- `.agent-local/doc-to-coding.md`
  clarification requests, ambiguity reports, missing-source warnings, and doc-triggered follow-up requests

Append new entries to the end of each mailbox file so the mailbox reads in chronological order from top to bottom.

If the file does not exist yet, create it locally when the first message is needed.

If a chat is interrupted and another agent takes over, add one short takeover line near the top of the replacement mailbox, for example:

- `taking over from coding-2 after interrupted chat`

Mailbox retention and archive policy:

- registry cleanup does not delete mailbox files automatically
- active working-set uid-based mailboxes stay in `.agent-local/mailboxes/`
- `.agent-local/mailboxes/EXAMPLE-planning-sync-handoff.md`, `.agent-local/mailboxes/EXAMPLE-planning-sync-resolution.md`, and `.agent-local/mailboxes/EXAMPLE-work-continuation-handoff.md` stay in place and are never archived
- once an agent entry has been removed from `.agent-local/agents.json`, its uid-based mailbox becomes an orphaned mailbox candidate
- orphaned uid-based mailboxes should be moved into `.agent-local/mailboxes/archive/YYYY-MM/` instead of being deleted
- use `scripts/mailbox_gc.py scan` to inspect referenced, missing, orphaned, and archived uid-based mailboxes
- use `scripts/mailbox_gc.py archive` to move orphaned uid-based mailboxes into the archive tree without deleting contents
- archived uid-based mailboxes older than 10 days may be deleted with `scripts/mailbox_gc.py prune` only when they do not contain an unresolved planning handoff
- use `scripts/inactive_coding_handoffs.py` to collect the latest open `Work Continuation Handoff` left by each `inactive` `coding` agent
- use `npm run handoffs:inactive-coding` as the short startup command for a new `coding` agent that wants to scan those leftover handoffs first
- shared fallback mailbox files outside `.agent-local/mailboxes/` are not touched by `scripts/mailbox_gc.py`; retire or archive those only by explicit team decision

## Workflow

1. an agent finishes one user-command work cycle.
2. before appending a new current-state handoff, the agent updates any older open current-state handoff in the same mailbox and same scope to `Status: superseded` when the new entry replaces it; `scripts/mailbox_handoff.py create` automates that step for new open entries.
3. every agent appends or updates one mailbox handoff entry in its own mailbox before ending the work cycle, so the mailbox records the latest state for that cycle.
4. `coding` normally satisfies that requirement with one open `Work Continuation Handoff`, even if no doc follow-up is needed.
5. if the landed work is planning-relevant, `coding` also appends a `Planning Sync Handoff` entry to its own mailbox or the intended peer mailbox named in `.agent-local/agents.json`.
6. `coding` commits and pushes the tracked code or doc changes.
7. the next `coding` agent that resumes or takes over the scope reads the newest open `Work Continuation Handoff` entry first.
8. `doc` reads the newest open planning-sync entry addressed to its scope or mailbox.
9. `doc` updates only the docs justified by that message.
10. `doc` still leaves one mailbox handoff entry for that completed work cycle, using a planning-sync resolution, blocking note, or doc continuation note as appropriate.
11. the agent that absorbs the prior handoff marks the original mailbox entry `resolved`, `blocked`, or `superseded`.

If the work is issue-first, the same summary can also be mirrored into the issue comment, but the local mailbox remains the default agent-to-agent transport.

## Required Fields

Every mailbox entry should include:

- date
- mode
- scope
- status
- files touched
- behavior change
- protocol/schema/CLI/fixture impact
- verify commands
- docs impacted
- planning impact
- remaining follow-up

Every `Work Continuation Handoff` should also include:

- current state
- next suggested step
- blockers
- last landed commit when one exists

Allowed `status` values:

- `open`
- `resolved`
- `blocked`
- `superseded`

Suggested `planning impact` values:

- `none`
- `design-note`
- `progress`
- `roadmap`
- `checklist`
- short combinations such as `design-note + checklist`

## Entry Template

Copy this block into the relevant mailbox file and keep the newest entries first.

```md
## 2026-03-11 - coding - <scope>

- Status: open
- Files touched: `<path>`, `<path>`
- Behavior change: <one short sentence>
- Protocol/schema/CLI/fixture impact: <none or one short sentence>
- Verify commands: `<command>`; `<command>`
- Docs impacted: `<path>` or `none`
- Planning impact: `none`
- Remaining follow-up: <one short sentence>
```

When `doc` resolves or responds, either update the original entry status or append a reply entry in the relevant mailbox file. Resolution and reply entries should include a `Date` line in `Asia/Taipei (UTC+8)` so humans can see when the docs work landed:

```md
## 2026-03-11 - doc - <scope>

- Status: resolved
- Date: 2026-03-11 15:20 UTC+8
- Files touched: `<path>`, `<path>`
- Behavior change: none
- Protocol/schema/CLI/fixture impact: none
- Verify commands: `not run`
- Docs impacted: `<path>`, `<path>`
- Planning impact: `checklist`
- Remaining follow-up: none
```

If `doc` wants a ready-made starting point, copy from `.agent-local/mailboxes/EXAMPLE-planning-sync-resolution.md`, or use `python3 scripts/mailbox_handoff.py create <agent-ref> planning-resolution ...`.

## Doc Continuation Note

When `doc` needs to leave the latest state for an unfinished sync batch or a completed bootstrap/read-only cycle, use a `Doc Continuation Note`.

Copyable doc continuation template:

```md
## Doc Continuation Note

- Status: open
- Date: 2026-03-13 17:18 UTC+8
- Source agent: doc-3
- Scope: <scope>
- Current state:
  - <what was confirmed or refreshed in this cycle>
- Evidence:
  - <command or source consulted>
- Next suggested step:
  - <best next doc/planning action>
```

If `doc` wants a ready-made starting point, copy from `.agent-local/mailboxes/EXAMPLE-doc-continuation-note.md`, or use `python3 scripts/mailbox_handoff.py create <agent-ref> doc-continuation ...`.

## Work Continuation Handoff

At the end of every completed `coding` work item, leave one continuation entry in the active coding mailbox. This is how `coding` satisfies the per-work-cycle mailbox-handoff requirement, and it remains mandatory even when there is no planning-sync impact.

At any moment, each coding mailbox should have at most one open `Work Continuation Handoff`. Before adding a newer one, close older open continuation entries in that mailbox by marking them `superseded`.

Purpose:

- let the next `coding` chat resume the work without reconstructing state from git history alone
- preserve the latest known state when the user stops assigning follow-up work
- reduce duplicate investigation during takeovers or resumed chats

Use `Status: open` when `coding` writes the entry.

That continuation entry stays open until one of the following happens:

- a later `coding` agent resumes the same scope and marks it `resolved`
- a newer continuation entry replaces it and marks the older one `superseded`
- the work is blocked and the absorbing agent updates the status to `blocked`

Copyable continuation template:

```md
## Work Continuation Handoff

- Status: open
- Date: 2026-03-12 22:30 UTC+8
- Source agent: coding-2
- Scope: <scope>
- Files changed:
  - `<path>`
  - `<path>`
- Behavior change:
  - <one short sentence>
- Verification:
  - `<command>`
  - `<command>`
- Last landed commit:
  - <short-sha subject> or `none`
- Current state:
  - <what is landed or locally understood now>
- Next suggested step:
  - <best next narrow slice if another coding agent resumes>
- Blockers:
  - `none` or <one short blocker sentence>
- Notes:
  - <optional short context>
```

If `coding` wants a ready-made starting point, copy from `.agent-local/mailboxes/EXAMPLE-work-continuation-handoff.md`, or use `python3 scripts/mailbox_handoff.py create <agent-ref> work-continuation ...`.

## Example

Example `coding` to `doc` message in `.agent-local/mailboxes/agt_example1234.md`:

```md
## 2026-03-11 - coding - #12 duplicate revision parent strictness

- Status: open
- Files touched: `crates/mycel-core/src/verify.rs`, `apps/mycel-cli/tests/object_verify_smoke.rs`
- Behavior change: reject duplicate revision parents earlier in verification
- Protocol/schema/CLI/fixture impact: none
- Verify commands: `cargo test -p mycel-core`; `cargo test -p mycel-cli`
- Docs impacted: `IMPLEMENTATION-CHECKLIST.en.md`
- Planning impact: `checklist`
- Remaining follow-up: update the checklist after the batch lands
```

## Due Planning Sync Example

Use this pattern when `coding` finishes implementation work and the landed change is likely to require planning-sync follow-up.

Sequence:

1. `coding` finishes the implementation slice in `file A`.
2. `coding` runs the relevant local verification.
3. `coding` appends an open entry to its mailbox or the relevant peer mailbox with `planning impact` set to the affected planning surfaces.
4. `coding` commits and pushes the tracked implementation change.
5. `coding` checks the latest CI status after the push.
6. when `doc` finishes its current work item and prepares next items, `doc` must run `scripts/check-plan-refresh.sh`.
7. if the script reports `due`, `doc` adds the due planning sync surfaces to the next items, scans the relevant handoff mailboxes, reads the mailbox entry, and follows [`PLANNING-SYNC-PLAN.md`](./PLANNING-SYNC-PLAN.md).
8. `doc` updates only the planning files justified by the landed change.
9. `doc` appends a reply or resolution entry with a `Date` line to its mailbox or the relevant peer mailbox, or updates the original planning handoff to `Status: resolved`.
10. `doc` commits and pushes the planning-sync change.

Example `coding` mailbox entry:

```md
## 2026-03-11 - coding - file A landed, planning sync due

- Status: open
- Files touched: `path/to/fileA`
- Behavior change: implemented the remaining accepted-head filter branch for file A
- Protocol/schema/CLI/fixture impact: CLI behavior changed
- Verify commands: `cargo test -p mycel-cli`
- Docs impacted: `ROADMAP.md`, `IMPLEMENTATION-CHECKLIST.en.md`, `IMPLEMENTATION-CHECKLIST.zh-TW.md`
- Planning impact: `roadmap + checklist`
- Remaining follow-up: likely `sync doc`, `sync issue`, or `sync web` work remains; `doc` should check cadence and scan handoff mailboxes when preparing next items
```

Example `doc` reply entry:

```md
## 2026-03-11 - doc - file A planning sync

- Status: resolved
- Date: 2026-03-11 15:20 UTC+8
- Files touched: `ROADMAP.md`, `IMPLEMENTATION-CHECKLIST.en.md`, `IMPLEMENTATION-CHECKLIST.zh-TW.md`
- Behavior change: none
- Protocol/schema/CLI/fixture impact: none
- Verify commands: `not run`
- Docs impacted: `ROADMAP.md`, `IMPLEMENTATION-CHECKLIST.en.md`, `IMPLEMENTATION-CHECKLIST.zh-TW.md`
- Planning impact: `roadmap + checklist`
- Remaining follow-up: none
```
