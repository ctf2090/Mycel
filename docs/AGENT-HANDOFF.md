# Agent Handoff Protocol

Status: active local-mailbox protocol for multi-agent coordination

Use this file as the tracked specification for how agents communicate through local gitignored mailboxes.

For agent discovery and role lookup, read [AGENT-REGISTRY.md](./AGENT-REGISTRY.md) and the local `.agent-local/agents.json` file first. Do not use mailbox traffic as a substitute for registry assignment confirmation.

Live mailbox files are local and not committed. Each agent should use the mailbox path declared in `.agent-local/agents.json`.

The directory is ignored by git through `.gitignore`, except for tracked template examples such as `.agent-local/mailboxes/EXAMPLE-planning-sync-handoff.md`.

## Modes

Use role-based execution modes:

- `coding`
  resolves issues, implements features, runs local verification, commits and pushes, and checks CI after each push
- `doc`
  syncs design notes, roadmap/checklist surfaces, and explanatory docs; this mode does not check CI

Use `coding` when the main output is behavior, tests, fixtures, parser/verifier work, or CLI changes.

Use `doc` when the main output is document sync after implementation or accepted design direction already exists.

Multiple agents may use the same role if their scopes and file ownership do not overlap.

## Mailbox Files

Default pattern:

- `.agent-local/<agent-id>.md`
  one mailbox per agent, with peer-to-peer handoff entries addressed by scope and agent id

Shared fallback pattern:

- `.agent-local/coding-to-doc.md`
  implementation handoff, doc-impact notes, planning-surface follow-up, and unresolved doc requests
- `.agent-local/doc-to-coding.md`
  clarification requests, ambiguity reports, missing-source warnings, and doc-triggered follow-up requests

Keep the newest entry at the top of each mailbox file.

If the file does not exist yet, create it locally when the first message is needed.

If a chat is interrupted and another agent takes over, add one short takeover line near the top of the replacement mailbox, for example:

- `taking over from coding-2 after interrupted chat`

## Workflow

1. `coding` finishes one issue slice or chat-first implementation slice.
2. `coding` appends a new entry to its mailbox or the intended peer mailbox named in `.agent-local/agents.json`.
3. `coding` commits and pushes the tracked code or doc changes.
4. `doc` reads the newest open entry addressed to its scope or mailbox.
5. `doc` updates only the docs justified by that message.
6. `doc` appends any follow-up or blocking question to its own mailbox or the relevant peer mailbox.
7. `doc` marks the original mailbox entry `resolved`, `blocked`, or `superseded`.

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

When `doc` resolves or responds, either update the original entry status or append a reply entry in the relevant mailbox file:

```md
## 2026-03-11 - doc - <scope>

- Status: resolved
- Files touched: `<path>`, `<path>`
- Behavior change: none
- Protocol/schema/CLI/fixture impact: none
- Verify commands: `not run`
- Docs impacted: `<path>`, `<path>`
- Planning impact: `checklist`
- Remaining follow-up: none
```

## Example

Example `coding` to `doc` message in `.agent-local/coding-1.md`:

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

Use this pattern when `coding` finishes implementation work and `scripts/check-doc-refresh.sh` reports that planning sync is due.

Sequence:

1. `coding` finishes the implementation slice in `file A`.
2. `coding` runs the relevant local verification.
3. `coding` appends an open entry to its mailbox or the relevant peer mailbox with `planning impact` set to the affected planning surfaces.
4. `coding` commits and pushes the tracked implementation change.
5. `coding` checks the latest CI status after the push.
6. `doc` reads the mailbox entry and follows [`PLANNING-SYNC-PLAN.md`](./PLANNING-SYNC-PLAN.md).
7. `doc` updates only the planning files justified by the landed change.
8. `doc` appends a reply or resolution entry to its mailbox or the relevant peer mailbox.
9. `doc` commits and pushes the planning-sync docs change.

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
- Remaining follow-up: `scripts/check-doc-refresh.sh` reported due; sync planning surfaces for the landed behavior
```

Example `doc` reply entry:

```md
## 2026-03-11 - doc - file A planning sync

- Status: resolved
- Files touched: `ROADMAP.md`, `IMPLEMENTATION-CHECKLIST.en.md`, `IMPLEMENTATION-CHECKLIST.zh-TW.md`
- Behavior change: none
- Protocol/schema/CLI/fixture impact: none
- Verify commands: `not run`
- Docs impacted: `ROADMAP.md`, `IMPLEMENTATION-CHECKLIST.en.md`, `IMPLEMENTATION-CHECKLIST.zh-TW.md`
- Planning impact: `roadmap + checklist`
- Remaining follow-up: none
```
