# Coding Role Checklist

Status: canonical source for `coding` role work

Use this tracked file as the source for a per-agent checklist copy. Do not mark
progress in this file directly.

Suggested per-agent copy path:

- `.agent-local/agents/<agent_uid>/checklists/ROLE-coding-checklist.md`

## New chat bootstrap

- Confirm the registry state and active peers before taking implementation scope. <!-- item-id: coding.startup.registry-state -->
- Check the latest completed CI result for the previous push before starting the next coding slice. <!-- item-id: coding.startup.check-latest-ci -->

## Work Cycle Workflow

- Run `git status -sb` and avoid unrelated user changes already in the worktree. <!-- item-id: coding.cycle.git-status -->
- When touching a large module or repeated-helper-heavy area, consult the current code-quality hotspot scan (`python3 scripts/check_code_quality_hotspots.py --github-warning`) so the coding slice stays aligned with the repo's warning-only CI surface. <!-- item-id: coding.cycle.consult-hotspot-scan -->
- Review the roadmap and identify the highest-value next coding work as one default next-item recommendation at the end of the work cycle. <!-- item-id: coding.cycle.review-roadmap-priority -->
- Review the latest CQH issue and identify high-value work items as another default next-item recommendation at the end of bootstrap or a work cycle when the user has not already assigned the next concrete task. <!-- item-id: coding.cycle.review-cqh-priority -->

## Verification

## Commit And Push

## Handoff And Finish
