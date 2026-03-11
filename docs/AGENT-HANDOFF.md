# Agent Handoff Protocol

Status: active local-mailbox protocol for `coding` and `doc` modes

Use this file as the tracked specification for how the two agent modes communicate through local gitignored files.

The active mailbox files are not committed:

- `.agent-local/coding-to-doc.md`
- `.agent-local/doc-to-coding.md`

The directory is ignored by git through `.gitignore`, so agents can exchange local state without polluting repo history.

## Modes

Use exactly two execution modes:

- `coding`
  resolves issues, implements features, runs local verification, commits and pushes, and checks CI after each push
- `doc`
  syncs design notes, roadmap/checklist surfaces, and explanatory docs; this mode does not check CI by default

Use `coding` when the main output is behavior, tests, fixtures, parser/verifier work, or CLI changes.

Use `doc` when the main output is document sync after implementation or accepted design direction already exists.

## Mailbox Files

Use one mailbox file in each direction:

- `.agent-local/coding-to-doc.md`
  implementation handoff, doc-impact notes, planning-surface follow-up, and unresolved doc requests
- `.agent-local/doc-to-coding.md`
  clarification requests, ambiguity reports, missing-source warnings, and doc-triggered follow-up requests

Keep the newest entry at the top of each file.

If the file does not exist yet, create it locally when the first message is needed.

## Workflow

1. `coding` finishes one issue slice or chat-first implementation slice.
2. `coding` appends a new entry to `.agent-local/coding-to-doc.md`.
3. `coding` commits and pushes the tracked code or doc changes.
4. `doc` reads the newest open entry in `.agent-local/coding-to-doc.md`.
5. `doc` updates only the docs justified by that message.
6. `doc` appends any follow-up or blocking question to `.agent-local/doc-to-coding.md`.
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

Copy this block into either mailbox file and keep the newest entries first.

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

When `doc` resolves or responds, either update the original entry status or append a reply entry in `.agent-local/doc-to-coding.md`:

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

Example `coding` to `doc` message in `.agent-local/coding-to-doc.md`:

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
