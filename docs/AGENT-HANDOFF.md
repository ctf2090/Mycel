# Agent Handoff Log

Status: active shared handoff surface for `coding` and `doc` roles

Use this file when one agent needs to pass implementation state to another agent without requiring both chats to be active at the same time.

Primary use:

- `coding` to `doc` handoff after issue resolution, feature work, or behavior-changing maintenance
- `doc` follow-through notes after planning or explanatory docs are synced
- chat-first work that still needs a durable repo-local handoff record

Do not use this file for:

- long design discussion
- speculative roadmap ideas not grounded in landed code or accepted design notes
- replacing issue acceptance criteria or repo-level planning documents

## Workflow

1. `coding` finishes one issue slice or one chat-first local scope.
2. `coding` appends one new handoff entry at the top of this file.
3. `coding` commits and pushes the related code or doc change.
4. `doc` reads the newest unresolved handoff entry.
5. `doc` updates only the docs named in the handoff.
6. `doc` marks the handoff resolved, superseded, or blocked.

If the work is issue-first, prefer leaving the same summary in the issue comment as well. This file is the repo-local fallback and shared queue.

## Required Fields

Every handoff entry should include:

- date
- role
- scope
- status
- files touched
- behavior change
- protocol/schema/CLI/fixture impact
- verify commands
- docs impacted
- planning impact
- remaining follow-up

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

Copy this block for each new handoff and keep the newest entries first.

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

When `doc` resolves a handoff, update the same entry:

```md
- Status: resolved
- Docs updated: `<path>`, `<path>`
- Resolution note: <one short sentence>
```

## Example

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

## Current Queue

No open handoffs yet.
