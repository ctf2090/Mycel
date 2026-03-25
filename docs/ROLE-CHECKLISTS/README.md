# Role Checklists

Status: active checklist sources for role-specific work

This directory holds the tracked checklist sources for each agent role.

Use these files as the canonical source only:

- `coding.md`
- `delivery.md`
- `doc.md`

Do not mark progress in these tracked files directly.

Instead, each agent should materialize its own checklist copy under its own
agent-local checklist directory using this standard path pattern:

- `.agent-local/agents/<agent_uid>/checklists/ROLE-<role>-bootstrap-checklist.md`
- `.agent-local/agents/<agent_uid>/checklists/ROLE-<role>-workcycle-checklist-<n>.md`

Recommended workflow:

1. Keep the role checklist source in this directory.
2. Create a per-agent copy with `scripts/item_id_checklist.py`.
3. Update only the per-agent copy while working.

Current section naming:

- role checklist sources should use `New chat bootstrap` for startup items
- role checklist sources should use `Work Cycle Workflow` for per-command cycle items
- this keeps the role-specific checklist structure aligned with the main `AGENTS.md` flow

The standard `AGENTS.md` bootstrap and work-cycle checklists are still generated
automatically by the registry and work-cycle tools. Role checklist copies now
follow the same bootstrap/work-cycle split under each agent-local checklist
directory. These role checklists are an additional role-focused layer, not a
replacement for the generated `AGENTS.md` checklists.

Layering rule:

- keep shared agent-process rules in [`AGENTS.md`](../../AGENTS.md)
- keep registry-specific coordination rules in [`docs/AGENT-REGISTRY.md`](../AGENT-REGISTRY.md)
- keep only role-specific deltas in `coding.md`, `delivery.md`, and `doc.md`
