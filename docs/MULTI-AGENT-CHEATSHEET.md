# Multi-Agent Cheat Sheet

Status: draft

Use this as the short maintainer view of [MULTI-AGENT-COORDINATION.md](./MULTI-AGENT-COORDINATION.md).

Tracked registry spec: [AGENT-REGISTRY.md](./AGENT-REGISTRY.md)

Tracked mailbox spec: [AGENT-HANDOFF.md](./AGENT-HANDOFF.md)

Local registry file:

- `.agent-local/agents.json`

Local mailbox files:

- `.agent-local/mailboxes/<agent_uid>.md`
- archive: `.agent-local/mailboxes/archive/YYYY-MM/<agent_uid>.md`
- example template: `.agent-local/mailboxes/EXAMPLE-planning-sync-handoff.md`
- resolution template: `.agent-local/mailboxes/EXAMPLE-planning-sync-resolution.md`
- continuation template: `.agent-local/mailboxes/EXAMPLE-work-continuation-handoff.md`
- fallback: `.agent-local/coding-to-doc.md`
- fallback: `.agent-local/doc-to-coding.md`

Mailbox retention:

- registry cleanup does not delete mailbox files automatically
- active working-set uid-based mailboxes stay in `.agent-local/mailboxes/`
- orphaned uid-based mailboxes should move into `.agent-local/mailboxes/archive/YYYY-MM/`
- use `npm run handoffs:inactive-coding` after a new `coding` agent starts to check leftover open continuation handoffs from inactive coding agents
- use `scripts/mailbox_gc.py scan` to inspect referenced, missing, orphaned, and archived uid-based mailboxes
- use `scripts/mailbox_gc.py archive` to move orphaned uid-based mailboxes without deleting contents
- archived uid-based mailboxes older than 10 days may be deleted with `scripts/mailbox_gc.py prune` when they have no unresolved planning handoff
- shared fallback mailboxes outside `.agent-local/mailboxes/` are not touched by `scripts/mailbox_gc.py`

Doc cadence reminder:

- after each completed doc work item, while preparing next items, `doc` must run `scripts/check-plan-refresh.sh`
- if it reports `due`, add the due planning surfaces as next items and use `docs/PLANNING-SYNC-PLAN.md` as the entry point
- when `doc` mirrors a summary into a GitHub issue comment or closes an issue with a Markdown note, prefer `scripts/gh_issue_comment.py ... --body-file - <<'EOF'` over inline shell-quoted `gh issue ... --body/--comment "..."` text

## Agent Roles

- `coding`: owns issue resolution, feature work, local verification, commit/push flow, and CI checks after each push
- `doc`: owns `sync doc` / `sync plan` work, design notes, roadmap/checklist refresh, and planning-surface wording; this role does not check CI

Use `coding` when the main output is behavior, tests, fixtures, parser/verifier work, or CLI changes.

Use `doc` when the main output is syncing planning or explanatory docs after behavior is already settled.

Multiple agents may share the same role. Read `.agent-local/agents.json` first to see how many agents are active and which role each one owns.

No tracked work starts until the agent confirms its own entry in `.agent-local/agents.json`.

## Identity Model

- `agent_uid` is the stable identity for the chat and is never reused
- `display_id` is the short human-facing id such as `coding-1` and may be recycled
- write commands should prefer `agent_uid`
- the transitional CLI still accepts either `agent_uid` or the current `display_id` as `<agent-ref>`
- once a stale entry releases its `display_id`, only `agent_uid` can address that old entry

Startup command:

- `scripts/agent_registry.py claim <role|auto> [--scope <scope>]`
- `scripts/agent_registry.py start <agent-ref>`
- `scripts/agent_registry.py touch <agent-ref>`
- `scripts/agent_registry.py finish <agent-ref>`
- `scripts/agent_work_cycle.py begin <agent-ref> [--scope <scope-label>]`
- `scripts/agent_work_cycle.py end <agent-ref> [--scope <scope-label>]`
- `scripts/agent_registry.py status [<agent-ref>]`
- `scripts/agent_registry.py resume-check <agent-ref>`
- `scripts/agent_registry.py stop <agent-ref> [--status paused|done]`
- `scripts/agent_registry.py recover <agent-ref> [--scope <scope>]`
- `scripts/agent_registry.py takeover <stale-agent-ref> [--scope <scope>]`
- `scripts/agent_registry.py cleanup`

Startup self-label:

- `<display-id> | <scope-label>`

Startup order:

1. `scripts/agent_registry.py claim <role|auto> [--scope <scope>]` if needed
2. `scripts/agent_registry.py start <agent-ref>`
3. `scripts/agent_registry.py status <agent-ref>`
4. if the new agent is `coding`, run `npm run handoffs:inactive-coding` to see leftover inactive-coding continuation handoffs before taking new implementation scope
5. `scripts/agent_work_cycle.py begin <agent-ref> [--scope <scope-label>]` before working the current command
6. first chat line: `<display-id> | <scope-label>`

Do not run `claim`, `start`, and `status` in parallel.

Per-command activity:

1. prefer `scripts/agent_work_cycle.py begin <agent-ref> [--scope <scope-label>]` before working; it wraps `touch` and prints the canonical before-work timestamp line, and that exact line should appear in user-visible commentary
2. prefer `scripts/agent_work_cycle.py end <agent-ref> [--scope <scope-label>]` after the command completes; it wraps `finish` and prints the canonical after-work timestamp line, and that exact line should appear in user-visible commentary
3. do not immediately follow `scripts/agent_work_cycle.py begin|end` with a manual `scripts/agent_registry.py touch|finish` for the same work cycle
4. use `scripts/agent_timestamp.py before|after --agent <display-id> --scope <scope-label>` only when you need the timestamp line without the registry change, and keep the same single-line `UTC+8` format
5. normal progress updates should not add hand-written date or time prefixes; reserve timestamps for the canonical before/after lines
6. inactive entries older than one hour become stale and release their `display_id`
7. once an inactive stale entry stays retained for 24 more hours, `cleanup` removes it from `.agent-local/agents.json`
8. paused entries older than 24 hours become stale-paused and release their `display_id`
9. paused entries older than 3 days are cleanup candidates and should be removed from `.agent-local/agents.json`

## Bootstrap Transcript

Use this copyable pattern for a fresh chat when the user explicitly assigns a role:

```text
Please read AGENTS.md and treat this chat as the coding role.

Repo status: `## main...origin/main`

Short plan:
1. Read `AGENTS.md`, `AGENTS-LOCAL.md` if it exists locally, `docs/AGENT-REGISTRY.md`, and `.agent-local/agents.json`.
2. Claim the `coding` role for this chat and start the registry entry.
3. Begin the current work cycle with the canonical timestamp line, then report the claimed role and repo status.

Background terminal finished with python scripts/agent_registry.py claim coding --scope read-agents-md
Background terminal finished with python scripts/agent_registry.py start agt_example1234
Background terminal finished with python scripts/agent_registry.py status agt_example1234
Background terminal finished with python scripts/agent_work_cycle.py begin agt_example1234 --scope read-agents-md

[2026-03-12 14:50:20 UTC+8] Before work | coding-1 | read-agents-md

... do the startup/read work for this command cycle ...

Background terminal finished with python scripts/agent_work_cycle.py end agt_example1234 --scope read-agents-md

[2026-03-12 14:51:05 UTC+8] After work | coding-1 | read-agents-md

Please read AGENTS.md and treat this chat as the coding role.

我已讀過 AGENTS.md、AGENTS-LOCAL.md（若本機存在）、docs/AGENT-REGISTRY.md 與 `.agent-local/agents.json`，並依流程認領 `coding` 角色為 `coding-1`（`agt_example1234`）。目前 repo 狀態是 `## main...origin/main`；若有既有未提交變更，我會避開它們。
```

Interrupted chat recovery:

1. `scripts/agent_registry.py status`
2. read the stale agent mailbox, starting from the newest open `Work Continuation Handoff`
3. if the original chat itself is returning, run `scripts/agent_registry.py resume-check <agent_uid>` and then `scripts/agent_registry.py recover <agent_uid>` if the display slot was released
4. if a different chat is taking over, run `scripts/agent_registry.py takeover <stale-agent-ref>`
5. read the stale mailbox before resuming tracked work

## Takeover Transcript

Use this copyable pattern when a fresh `coding` chat is explicitly taking over an inactive coding handoff:

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

Reopened chat startup:

1. `read AGENTS.md, you are <role>`
2. `scripts/agent_registry.py status`
3. `scripts/agent_registry.py resume-check <agent_uid>`
4. if `must_recover = true`, run `scripts/agent_registry.py recover <agent_uid>`
5. read `.agent-local/mailboxes/<agent_uid>.md`
6. first chat line: `<display-id> | <scope-label>`

Role note:

- `coding` usually reports the latest completed CI result after recovery
- `doc` usually skips CI unless explicitly asked
- if an old forgotten chat is reopened, run `scripts/agent_registry.py resume-check <agent_uid>` before doing any tracked work

## 10-Line Rule Set

1. Default to hybrid mode, not issue-for-everything.
2. Read `.agent-local/agents.json`; if the user declared only a role, claim an id first with `scripts/agent_registry.py claim <role>`.
3. Use one agent per issue when the work needs claims, handoff, or more than one commit.
4. One active issue should map to one chat and one worktree or isolated session.
5. Small local fixes can stay chat-first, but do not let them widen silently.
6. Claim the issue before editing, or leave a short local-scope note for chat-first work.
7. Do not run two agents on the same primary file at the same time.
8. Split work by file boundary, not by vague subtopic.
9. Verify with the commands named in the issue or local scope before handoff.
10. Push serially, never in parallel. If `origin/main` moved, fetch and rebase before retrying. If the spec is unclear, stop and mark the task `blocked-by-spec`.

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
- what behavior changed
- whether protocol, schema, CLI, or fixture meaning changed
- which verify commands passed
- which docs are impacted
- whether planning impact is `none`, `design-note`, `progress`, `roadmap`, `checklist`, or a short combination
- what remains open

For `coding`, always leave one open `Work Continuation Handoff` at the end of the work item, even if there is no planning-sync note. Assume the user may stop assigning work after the current task.

Before leaving that new open continuation handoff, close any older open `Work Continuation Handoff` entries in the same mailbox by marking them `superseded`.

That continuation handoff should also say:

- current state
- next suggested step
- blockers
- last landed commit when one exists

Recommended format:

- `Finished #4. Touched protocol.rs and object_verify_smoke.rs. Ran cargo test -p mycel-core and cargo test -p mycel-cli. Remaining follow-up: malformed snapshot fixtures.`
- `Finished local CI-fix follow-up. Touched protocol.rs. Ran cargo fmt --all and cargo test --workspace. Remaining follow-up: none.`

For `coding` to `doc` handoff, prefer:

- `Finished #12. Touched verify.rs and object_verify_smoke.rs. Behavior change: reject duplicate revision parents earlier. Protocol/schema impact: none. Verify: cargo test -p mycel-core and cargo test -p mycel-cli. Docs impacted: none. Planning impact: checklist. Remaining follow-up: update IMPLEMENTATION-CHECKLIST after the batch lands.`
- `Finished file A. Touched path/to/fileA. Behavior change: implemented the missing branch. Protocol/schema impact: CLI behavior changed. Verify: cargo test -p mycel-cli. Docs impacted: ROADMAP.md and IMPLEMENTATION-CHECKLIST.*. Planning impact: roadmap + checklist. Remaining follow-up: planning sync due.`
- planning-sync handoffs should always include `Status: open`; after `doc` finishes, mark them `resolved` or append a `doc` reply entry with a `Date` line in `UTC+8`
- work-continuation handoffs should always include `Status: open`; keep only one open continuation handoff per coding mailbox, and supersede older open continuation notes before adding a newer one
- before `doc` starts `sync doc` or `sync web`, scan the relevant handoff mailboxes and treat open planning-sync notes as the first collection input
- use `.agent-local/mailboxes/EXAMPLE-planning-sync-handoff.md` for open handoffs and `.agent-local/mailboxes/EXAMPLE-planning-sync-resolution.md` for resolved doc replies
- use `.agent-local/mailboxes/EXAMPLE-work-continuation-handoff.md` for coding continuation notes

If there is no active issue comment thread, append the same content to the mailbox path declared for that agent in `.agent-local/agents.json`.
