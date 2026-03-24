# Doc Role Checklist

Status: canonical source for `doc` role work

Use this tracked file as the source for a per-agent checklist copy. Do not mark
progress in this file directly.

Suggested per-agent copy path:

- `.agent-local/agents/<agent_uid>/checklists/ROLE-doc-bootstrap-checklist.md`
- `.agent-local/agents/<agent_uid>/checklists/ROLE-doc-workcycle-checklist-<n>.md`

## New chat bootstrap

- Confirm the registry state and active peers before taking documentation or planning scope. <!-- item-id: doc.startup.registry-state -->
- Review the latest open same-role handoff when one exists and include it in the bootstrap next-work items. <!-- item-id: doc.startup.review-same-role-handoff -->
- Review open pull requests and distinguish Dependabot dependency-update PRs from human-authored product PRs when choosing bootstrap next-work items. <!-- item-id: doc.startup.review-open-prs -->

## Work Cycle Workflow

- Run `git status -sb` and avoid unrelated user changes already in the worktree. <!-- item-id: doc.cycle.git-status -->
- Treat `ROADMAP.*` and `IMPLEMENTATION-CHECKLIST.*` as the higher planning authority when surfaces disagree. <!-- item-id: doc.cycle.source-of-truth-order -->
- Before starting `sync doc`, `sync web`, or `sync plan`, scan the relevant registry mailboxes and use those notes as planning-sync input. <!-- item-id: doc.cycle.scan-mailboxes-before-sync -->
- Use `docs/PLANNING-SYNC-PLAN.md` as the entry point for `sync doc`, `sync web`, and `sync plan` batches. <!-- item-id: doc.cycle.planning-entry-point -->

## Planning Sync

- Run `scripts/check-plan-refresh.sh` after each completed doc work item while preparing next items. <!-- item-id: doc.plan.run-refresh-check -->

## Boundaries

- Do not check CI as part of normal `doc` work. <!-- item-id: doc.boundary.no-ci -->

## Handoff And Finish
