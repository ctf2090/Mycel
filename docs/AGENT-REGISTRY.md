# Agent Registry Protocol

Status: active local-registry protocol for multi-agent coordination

Use this file as the tracked specification for the local registry that tells agents how many agents are active, what role each one has, and whether each agent has confirmed that assignment before starting tracked work.

The live registry file is local and gitignored:

- `.agent-local/agents.json`

Recommended startup gate:

- `scripts/agent-claim.sh <role> [--scope <scope>]`
- `scripts/agent-start.sh <agent-id>`
- `scripts/agent-stop.sh <agent-id> [--status paused|done]`
- `scripts/agent-recover.sh <stale-agent-id> [--scope <scope>]`

Recommended status command:

- `scripts/agent-status.sh [<agent-id>]`

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

If a new chat receives only a role declaration such as `you are coding` or `you are doc`, the agent should claim a fresh id with `scripts/agent-claim.sh <role>` before running `scripts/agent-start.sh <agent-id>`.

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
      "assigned_at": "2026-03-11T00:00:00Z",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-11T00:02:00Z",
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
      "assigned_at": "2026-03-11T00:01:00Z",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-11T00:03:00Z",
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
- `status`
- `scope`
- `files`
- `mailbox`

Allowed `role` values:

- `coding`
- `doc`

Allowed `status` values:

- `active`
- `paused`
- `blocked`
- `done`

`agent_count` must equal the number of entries in `agents`.

`confirmed_by_agent` must be `true` before the agent starts tracked work.

`confirmed_at` may be `null` only while the entry is still waiting for agent confirmation.

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

1. either a maintainer writes the assignment entry or the agent claims a new entry with `scripts/agent-claim.sh <role>`
2. the agent runs `scripts/agent-start.sh <agent-id>`
3. the start script confirms the role, sets `confirmed_by_agent: true`, stamps `confirmed_at`, and creates the mailbox if needed
4. only then may tracked work begin

## Workflow

1. Before starting work, an agent reads `.agent-local/agents.json`.
2. The agent confirms the current agent count and scans the existing scopes and file sets.
3. If no entry exists yet but the role is known, the agent may claim a new id with `scripts/agent-claim.sh <role>`.
4. Otherwise, a maintainer or coordinator writes the agent entry with `role`, `assigned_by`, `assigned_at`, `scope`, and `mailbox`.
5. The agent confirms its own assignment by running `scripts/agent-start.sh <agent-id>`.
6. Only after confirmation may the agent start tracked work.
7. The agent uses its own `mailbox` file for peer coordination and handoff traffic.
8. When scope changes, the agent updates its registry entry.
9. When work is finished or paused, the agent updates `status`, preferably with `scripts/agent-stop.sh <agent-id> [--status paused|done]`.

If two `coding` agents would touch the same primary file or issue, one must pause or choose a narrower scope before proceeding.

## Standard New Chat Startup

Use this sequence in order. Do not run the registry commands in parallel.

1. read `AGENTS.md`, `AGENTS-LOCAL.md`, and `docs/AGENT-REGISTRY.md`
2. run `git status -sb`
3. check `rg` and `gh`
4. check the latest CI status from the previous push
5. if the user declared only a role, run `scripts/agent-claim.sh <role> [--scope <scope>]`
6. run `scripts/agent-start.sh <agent-id>`
7. run `scripts/agent-status.sh <agent-id>`
8. begin the chat with the startup self-label: `<agent-id> | <scope-label>`
9. only after that, report repo status and wait for the concrete task

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
- do not run `agent-claim`, `agent-start`, and `agent-status` in parallel
- do not omit the startup self-label line
- keep the CI line about the latest completed workflow, not a possibly in-progress run

## Interrupted Chat Recovery

Treat the local registry and mailbox files as the source of truth if a chat stops unexpectedly because of an OpenAI or Codespaces issue.

Recovery rules:

1. do not assume an `active` agent is still reachable just because the registry says `active`
2. read `.agent-local/agents.json` and the relevant mailbox file first
3. preserve the old agent entry for auditability; do not overwrite its `id`
4. if the old chat is clearly gone, mark that agent `paused` with `scripts/agent-stop.sh <agent-id>`
5. claim a new id for the replacement chat and continue from the mailbox handoff
6. if a previously forgotten chat is reopened later, that chat must run `scripts/agent-status.sh <its-agent-id>` before doing tracked work again
7. if the reopened chat is no longer `active`, it must stop and must not resume tracked work under the old id

Recommended recovery sequence:

1. run `scripts/agent-status.sh`
2. identify the stale `active` agent
3. read `.agent-local/<agent-id>.md`
4. either run `scripts/agent-stop.sh <old-agent-id>` then `scripts/agent-claim.sh <role>` plus `scripts/agent-start.sh <new-agent-id>`, or use `scripts/agent-recover.sh <old-agent-id>`
5. read the stale mailbox before resuming tracked work

Recommended scripted shortcut:

- `scripts/agent-recover.sh <old-agent-id>`

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
- 已執行 `scripts/agent-status.sh` 並確認舊 agent `<old-agent-id>` 需要接手
- 已執行 `scripts/agent-recover.sh <old-agent-id>`，目前這個 chat 是 `<new-agent-id>`，狀態 active
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
- it must re-check its own registry status before resuming work
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
      "assigned_at": "2026-03-11T00:00:00Z",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-11T00:02:00Z",
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
      "assigned_at": "2026-03-11T00:01:00Z",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-11T00:03:00Z",
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
