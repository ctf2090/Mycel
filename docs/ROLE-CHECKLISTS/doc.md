# Doc Role Checklist

Status: canonical source for `doc` role work

Use this tracked file as the source for a per-agent checklist copy. Do not mark
progress in this file directly.

Suggested per-agent copy path:

- `.agent-local/agents/<agent_uid>/checklists/ROLE-doc-checklist.md`

## Startup

- Confirm the registry state and active peers before taking documentation or planning scope. <!-- item-id: doc.startup.registry-state -->

## Work Cycle

- Run `git status -sb` and avoid unrelated user changes already in the worktree. <!-- item-id: doc.cycle.git-status -->
- Treat `ROADMAP.*` and `IMPLEMENTATION-CHECKLIST.*` as the higher planning authority when surfaces disagree. <!-- item-id: doc.cycle.source-of-truth-order -->
- Use `docs/PLANNING-SYNC-PLAN.md` as the entry point for `sync doc`, `sync web`, and `sync plan` batches. <!-- item-id: doc.cycle.planning-entry-point -->

## Planning Sync

- Run `scripts/check-plan-refresh.sh` after each completed doc work item while preparing next items. <!-- item-id: doc.plan.run-refresh-check -->

## Boundaries

- Do not check CI as part of normal `doc` work. <!-- item-id: doc.boundary.no-ci -->

## Handoff And Finish

