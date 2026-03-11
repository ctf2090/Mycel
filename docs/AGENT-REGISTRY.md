# Agent Registry Protocol

Status: active local-registry protocol for multi-agent coordination

Use this file as the tracked specification for the local registry that tells agents how many agents are active, what role each one has, and whether each agent has confirmed that assignment before starting tracked work.

The live registry file is local and gitignored:

- `.agent-local/agents.json`

Recommended startup gate:

- `scripts/agent-claim.sh <role> [--scope <scope>]`
- `scripts/agent-start.sh <agent-id>`

Recommended status command:

- `scripts/agent-status.sh [<agent-id>]`

Agents should read `.agent-local/agents.json` at the start of work to discover:

- how many agents are currently active
- each agent's `id`
- each agent's `role`
- who assigned that role
- whether the agent has already confirmed that assignment
- each agent's current scope
- whether a peer agent is active, paused, or done

If a new chat receives only a role declaration such as `you are coding` or `you are doc`, the agent should claim a fresh id with `scripts/agent-claim.sh <role>` before running `scripts/agent-start.sh <agent-id>`.

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
9. When work is finished or paused, the agent updates `status`.

If two `coding` agents would touch the same primary file or issue, one must pause or choose a narrower scope before proceeding.

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
