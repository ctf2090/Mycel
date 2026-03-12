# Agent Registry Protocol

Status: active local-registry protocol for multi-agent coordination

Use this file as the tracked specification for the local registry that tells agents how many agents are active, what role each one has, and whether each agent has confirmed that assignment before starting tracked work.

The live registry file is local and gitignored:

- `.agent-local/agents.json`

Recommended startup gate:

- `scripts/agent_registry.py claim <role|auto> [--scope <scope>]`
- `scripts/agent_registry.py start <agent-id>`
- `scripts/agent_registry.py touch <agent-id>`
- `scripts/agent_registry.py finish <agent-id>`
- `scripts/agent_registry.py stop <agent-id> [--status paused|done]`
- `scripts/agent_registry.py cleanup`
- `scripts/agent_registry.py recover <stale-agent-id> [--scope <scope>]`

Recommended status command:

- `scripts/agent_registry.py status [<agent-id>]`
- `scripts/agent_registry.py resume-check <agent-id>`

Recommended startup self-label:

- `<agent-id> | <scope-label>`

Agents should read `.agent-local/agents.json` at the start of work to discover:

- how many agents are currently active
- each agent's `id`
- each agent's `role`
- who assigned that role
- whether the agent has already confirmed that assignment
- each agent's current scope
- whether a peer agent is active, paused, or done

If a new chat receives only a role declaration such as `you are coding` or `you are doc`, the agent should claim a fresh id with `scripts/agent_registry.py claim <role>` before running `scripts/agent_registry.py start <agent-id>`.

If the user does not assign any role in a new chat, the agent should use `scripts/agent_registry.py claim auto` to choose the default role from `.agent-local/agents.json` before starting work:

- if there is no active `coding` agent, take `coding` first
- if active `coding >= 1` and active `doc == 0`, take `doc`
- if active `coding >= 1` and active `doc >= 1`, take `coding`

This default-role rule is only for chats without a user-assigned role. An explicit user role selection still wins.

After claim/start, the agent should begin the chat with one fixed self-label line using the registry id and current scope, for example:

- `coding-2 | forum-design-note-sync`
- `doc-1 | roadmap-sync-for-forum`

## Role Model

The system supports multiple concurrent agents, not just one `coding` and one `doc`.

Each agent entry must declare one role:

- `coding`
  owns issue resolution, feature work, local verification, commit/push flow, and CI checks after each push
- `doc`
  owns design-note sync, roadmap/checklist refresh, explanatory docs, and planning-surface wording; this role does not check CI by default

Any number of agents may share the same role, as long as they do not collide on the same issue or primary file set.

## Registry Shape

The local registry file must be valid JSON and use this top-level shape:

```json
{
  "version": 1,
  "updated_at": "2026-03-11T00:00:00Z",
  "agent_count": 2,
  "agents": [
    {
      "id": "agent-coding-1",
      "role": "coding",
      "assigned_by": "maintainer",
      "assigned_at": "2026-03-11T00:00:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-11T00:02:00+0800",
      "last_touched_at": "2026-03-11T00:10:00+0800",
      "inactive_at": null,
      "status": "active",
      "scope": "#42 accepted-head strictness",
      "files": [
        "crates/mycel-core/src/verify.rs",
        "apps/mycel-cli/tests/object_verify_smoke.rs"
      ],
      "mailbox": ".agent-local/agent-coding-1.md"
    },
    {
      "id": "agent-doc-1",
      "role": "doc",
      "assigned_by": "maintainer",
      "assigned_at": "2026-03-11T00:01:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-11T00:03:00+0800",
      "last_touched_at": "2026-03-11T00:11:00+0800",
      "inactive_at": null,
      "status": "active",
      "scope": "planning sync for #42",
      "files": [
        "ROADMAP.md",
        "IMPLEMENTATION-CHECKLIST.en.md"
      ],
      "mailbox": ".agent-local/agent-doc-1.md"
    }
  ]
}
```

## Required Fields

Top level:

- `version`
- `updated_at`
- `agent_count`
- `agents`

Per agent:

- `id`
- `role`
- `assigned_by`
- `assigned_at`
- `confirmed_by_agent`
- `confirmed_at`
- `last_touched_at`
- `inactive_at`
- `status`
- `scope`
- `files`
- `mailbox`

Allowed `role` values:

- `coding`
- `doc`

Allowed `status` values:

- `active`
- `inactive`
- `paused`
- `blocked`
- `done`

`agent_count` must equal the number of entries in `agents`.

`confirmed_by_agent` must be `true` before the agent starts tracked work.

`confirmed_at` may be `null` only while the entry is still waiting for agent confirmation.

`last_touched_at` may be `null` only before the entry has ever been activated or touched.

`inactive_at` should be a timestamp when `status` is `inactive`, and should be `null` otherwise.

`scripts/agent_registry.py` writes timestamps in `Asia/Taipei (UTC+8)` using the `+0800` offset form.

## Startup Gate

No agent may start tracked work until all of the following are true in `.agent-local/agents.json`:

1. the agent has a matching `id`
2. the entry has a non-empty `role`
3. the entry has `assigned_by` and `assigned_at`
4. the agent has set `confirmed_by_agent` to `true`
5. the entry has a non-null `confirmed_at`
6. the intended scope is present

If any of those checks fail, the agent must stop before editing tracked files and request a corrected assignment.

Recommended enforcement:

1. either a maintainer writes the assignment entry or the agent claims a new entry with `scripts/agent_registry.py claim <role|auto>`
2. the agent runs `scripts/agent_registry.py start <agent-id>`
3. the start script confirms the role, sets `confirmed_by_agent: true`, stamps `confirmed_at`, and creates the mailbox if needed
4. only then may tracked work begin

## Workflow

1. Before starting work, an agent reads `.agent-local/agents.json`.
2. The agent confirms the current agent count and scans the existing scopes and file sets.
3. If no entry exists yet but the role is known, the agent may claim a new id with `scripts/agent_registry.py claim <role>`; if the role is not user-assigned, the agent may use `scripts/agent_registry.py claim auto`.
4. Otherwise, a maintainer or coordinator writes the agent entry with `role`, `assigned_by`, `assigned_at`, `scope`, and `mailbox`.
5. The agent confirms its own assignment by running `scripts/agent_registry.py start <agent-id>`.
6. Only after confirmation may the agent start tracked work.
7. The agent uses its own `mailbox` file for peer coordination and handoff traffic.
8. Before doing work for each user command, the agent should run `scripts/agent_registry.py touch <agent-id>` so the registry marks that role active for the current command cycle.
9. After finishing work for that user command, the agent should run `scripts/agent_registry.py finish <agent-id>` so the registry marks that role inactive.
10. When scope changes, the agent updates its registry entry.
11. When work is finished or paused for longer-lived coordination reasons, the agent updates `status`, preferably with `scripts/agent_registry.py stop <agent-id> [--status paused|done]`.

If two `coding` agents would touch the same primary file or issue, one must pause or choose a narrower scope before proceeding.

## Activity Lease

The registry uses a per-command activity lease, not just a startup confirmation.

Rules:

1. on each new user command, the active agent should `touch` its own entry before starting work
2. when that command's work is complete, the agent should `finish` its own entry so the role becomes `inactive`
3. an entry that remains `inactive` for at least one hour becomes stale, but it is retained in the registry so its id is never reused automatically
4. `scripts/agent_registry.py cleanup` reports stale `inactive` entries for review instead of deleting them
5. a new chat should claim the next unused numeric suffix, not recycle the stale inactive id
6. a previously inactive confirmed agent may resume by re-checking or touching its own retained entry

## Standard New Chat Startup

Use this sequence in order. Do not run the registry commands in parallel.

1. read `AGENTS.md`, `AGENTS-LOCAL.md`, and `docs/AGENT-REGISTRY.md`
2. run `git status -sb`
3. check `rg` and `gh`
4. check the latest CI status from the previous push
5. determine the role for this chat:
   - if the user explicitly assigned a role, use that role and run `scripts/agent_registry.py claim <role> [--scope <scope>]`
   - otherwise run `scripts/agent_registry.py claim auto [--scope <scope>]`
6. run `scripts/agent_registry.py start <agent-id>`
7. run `scripts/agent_registry.py status <agent-id>`
8. when the first concrete user task arrives, run `scripts/agent_registry.py touch <agent-id>` before starting work
9. begin the chat with the startup self-label: `<agent-id> | <scope-label>`
10. only after that, report repo status and wait for the concrete task

Recommended startup output:

```text
coding-1 | pending-user-task

Please read AGENTS.md and operate as the coding agent.

已完成 coding agent 啟動流程，接下來我會照這套規則執行。

目前狀態：
- repo 乾淨：## main...origin/main
- 已讀取並套用 AGENTS.md、AGENTS-LOCAL.md、docs/AGENT-REGISTRY.md
- 已確認本地 agent registry：這個 chat 是 coding-1，狀態 active，scope 是 pending-user-task
- 前一次已完成的 CI 正常：latest completed workflow success
- 後續 commit 會用 `gpt-5:coding-1` 作為 agent identity

把具體任務丟給我，我就直接開始做。
```

Keep this startup output narrow:

- do not claim file-specific context before the user gives a concrete task
- do not run `claim`, `start`, and `status` in parallel
- do not omit the startup self-label line
- keep the CI line about the latest completed workflow, not a possibly in-progress run
- mark the agent `inactive` with `scripts/agent_registry.py finish <agent-id>` after the command-level work is done

## Interrupted Chat Recovery

Treat the local registry and mailbox files as the source of truth if a chat stops unexpectedly because of an OpenAI or Codespaces issue.

Recovery rules:

1. do not assume an `active` agent is still reachable just because the registry says `active`
2. read `.agent-local/agents.json` and the relevant mailbox file first
3. preserve the old agent entry for auditability; do not overwrite its `id`
4. if the old chat is clearly gone, mark that agent `paused` with `scripts/agent_registry.py stop <agent-id>`
5. claim a new id for the replacement chat and continue from the mailbox handoff
6. if a previously forgotten chat is reopened later, that chat must run `scripts/agent_registry.py resume-check <its-agent-id>` before doing tracked work again
7. if the reopened chat is no longer `active`, it must stop and must not resume tracked work under the old id

Recommended recovery sequence:

1. run `scripts/agent_registry.py status`
2. identify the stale `active` agent
3. read `.agent-local/<agent-id>.md`
4. either run `scripts/agent_registry.py stop <old-agent-id>` then `scripts/agent_registry.py claim <role>` plus `scripts/agent_registry.py start <new-agent-id>`, or use `scripts/agent_registry.py recover <old-agent-id>`
5. read the stale mailbox before resuming tracked work

Recommended scripted shortcut:

- `scripts/agent_registry.py recover <old-agent-id>`

The recovery helper pauses the stale agent, creates a fresh id for the same role, starts the replacement entry immediately, and appends the default takeover note to the new mailbox.

Recommended takeover note:

- `taking over from coding-2 after interrupted chat`

Recommended reopened chat startup:

```text
<new-agent-id> | <scope-label>

Please read AGENTS.md and operate as the <role> agent.

已完成 interrupted-chat recovery，接下來我會接手前一個中斷 chat 的工作。

目前狀態：
- repo 乾淨：## main...origin/main
- 已讀取並套用 AGENTS.md、AGENTS-LOCAL.md、docs/AGENT-REGISTRY.md
- 已執行 `scripts/agent_registry.py status` 並確認舊 agent `<old-agent-id>` 需要接手
- 已執行 `scripts/agent_registry.py recover <old-agent-id>`，目前這個 chat 是 `<new-agent-id>`，狀態 active
- 已讀取舊 mailbox `.agent-local/<old-agent-id>.md` 與新 mailbox `.agent-local/<new-agent-id>.md`
- 前一次已完成的 CI 正常：latest completed workflow success
- 後續 commit 會用 `gpt-5:<new-agent-id>` 作為 agent identity

把接續的任務丟給我，我就直接開始做。
```

Keep this recovery startup output narrow:

- identify the stale agent id explicitly
- confirm that the old mailbox was read before resumed work
- use the new replacement id in the self-label and agent identity line
- do not claim new file-level context until the user gives the next concrete task

Forgotten-chat note:

- a reopened old chat is not trusted just because the window still exists
- it must re-check its own registry status before resuming work, preferably with `scripts/agent_registry.py resume-check <agent-id>`
- if the old id is merely `inactive`, it may resume its own retained entry and a new chat should continue under a newer id such as `coding-2`
- if another chat already recovered the scope and the old id is now `paused`, the reopened old chat must stop and yield to the replacement id

Role note:

- `coding` should keep the CI line because that role owns CI checks after pushes
- `doc` can omit the CI line unless the maintainer explicitly asked that chat to monitor CI

## Mailbox Rule

The registry tells agents who exists. Mailboxes carry the actual messages.

Recommended local mailbox pattern:

- `.agent-local/<agent-id>.md`

If a simpler shared mailbox flow is preferred, agents may still use:

- `.agent-local/coding-to-doc.md`
- `.agent-local/doc-to-coding.md`

The registry remains the source for current agent count and role assignment.

## Minimal Example

For one `coding` agent and one `doc` agent:

```json
{
  "version": 1,
  "updated_at": "2026-03-11T00:00:00Z",
  "agent_count": 2,
  "agents": [
    {
      "id": "coding-1",
      "role": "coding",
      "assigned_by": "maintainer",
      "assigned_at": "2026-03-11T00:00:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-11T00:02:00+0800",
      "last_touched_at": "2026-03-11T00:10:00+0800",
      "inactive_at": null,
      "status": "active",
      "scope": "#17 store refactor",
      "files": [
        "apps/mycel-cli/src/store.rs",
        "apps/mycel-cli/src/store/index.rs"
      ],
      "mailbox": ".agent-local/coding-1.md"
    },
    {
      "id": "doc-1",
      "role": "doc",
      "assigned_by": "maintainer",
      "assigned_at": "2026-03-11T00:01:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-11T00:03:00+0800",
      "last_touched_at": "2026-03-11T00:11:00+0800",
      "inactive_at": null,
      "status": "active",
      "scope": "planning sync for #17",
      "files": [
        "ROADMAP.md",
        "IMPLEMENTATION-CHECKLIST.en.md"
      ],
      "mailbox": ".agent-local/doc-1.md"
    }
  ]
}
```
