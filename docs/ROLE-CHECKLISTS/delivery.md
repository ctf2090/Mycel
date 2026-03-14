# Delivery Role Checklist

Status: canonical source for `delivery` role work

Use this tracked file as the source for a per-agent checklist copy. Do not mark
progress in this file directly.

Suggested per-agent copy path:

- `.agent-local/agents/<agent_uid>/checklists/ROLE-delivery-checklist.md`

## New chat bootstrap

- Confirm the registry state and active peers before taking CI or process scope. <!-- item-id: delivery.startup.registry-state -->
- Check the latest completed CI result for the previous push before starting the next delivery slice. <!-- item-id: delivery.startup.check-latest-ci -->

## Work Cycle Workflow

- Run `git status -sb` and avoid unrelated user changes already in the worktree. <!-- item-id: delivery.cycle.git-status -->
- Keep the scope focused on CI health, workflow/process tooling, merge readiness, or blocker triage. <!-- item-id: delivery.cycle.keep-scope -->
- Route product-code fixes to `coding` and planning-surface wording to `doc` through mailbox handoffs when the issue crosses role boundaries. <!-- item-id: delivery.cycle.route-cross-role -->

## Verification

## Commit And Push

## Handoff And Finish

- Leave one open `Delivery Continuation Note` in the mailbox at the end of the work cycle, plus a `Planning Sync Handoff` when delivery work changes planning-visible process state. <!-- item-id: delivery.finish.leave-handoff -->
