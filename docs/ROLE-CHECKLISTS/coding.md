# Coding Role Checklist

Status: canonical source for `coding` role work

Use this tracked file as the source for a per-agent checklist copy. Do not mark
progress in this file directly.

Suggested per-agent copy path:

- `.agent-local/agents/<agent_uid>/checklists/ROLE-coding-bootstrap-checklist.md`
- `.agent-local/agents/<agent_uid>/checklists/ROLE-coding-workcycle-checklist-<n>.md`

## New chat bootstrap

- Confirm the registry state and active peers before taking implementation scope. <!-- item-id: coding.startup.registry-state -->
- Check the latest completed CI result for the previous push before starting the next coding slice. <!-- item-id: coding.startup.check-latest-ci -->
- Review the latest open same-role handoff when one exists and include it in the bootstrap next-work items. <!-- item-id: coding.startup.review-same-role-handoff -->

## Work Cycle Workflow

- Run `git status -sb` and avoid unrelated user changes already in the worktree. <!-- item-id: coding.cycle.git-status -->
- When touching a large module or repeated-helper-heavy area, consult the current code-quality hotspot scan (`python3 scripts/check_code_quality_hotspots.py --github-warning`) so the coding slice stays aligned with the repo's warning-only CI surface. <!-- item-id: coding.cycle.consult-hotspot-scan -->
- Hand planning-relevant implementation state to `doc` through the registry mailbox instead of running planning-refresh work directly. <!-- item-id: coding.cycle.handoff-planning-state -->
- Include the shared `coding` next-item defaults from `AGENTS.md`, especially reviewing `ROADMAP.md` for the highest-value next coding work and reviewing the latest CQH issue for high-value work items when the user has not already assigned the next concrete task. <!-- item-id: coding.cycle.follow-shared-next-item-guidance -->

## Verification

## Commit And Push

## Handoff And Finish
