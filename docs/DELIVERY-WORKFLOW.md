# Delivery Workflow

Status: active first-pass runbook for the `delivery` role

Use this document when a chat claims the `delivery` role and needs a practical
day-to-day operating loop.

Read this together with:

- [`AGENTS.md`](../AGENTS.md)
- [`AGENT-REGISTRY.md`](./AGENT-REGISTRY.md)
- [`AGENT-HANDOFF.md`](./AGENT-HANDOFF.md)
- [`ROLE-CHECKLISTS/delivery.md`](./ROLE-CHECKLISTS/delivery.md)

## Purpose

`delivery` exists to keep the path from landed change to healthy integration
moving.

This role is the default owner for:

- latest completed CI checks before delivery-focused work
- CI failure triage
- flaky-test follow-up
- workflow and process tooling updates
- merge or release readiness coordination
- blocker routing when the problem is not purely product-code implementation

This role is not the default owner for:

- product behavior changes whose primary fix is in application or library code
- roadmap, checklist, or progress wording
- broad design-note or planning-sync refresh work

## When To Use Delivery

Use `delivery` when the current question is closest to one of these:

- "Why did the latest completed CI fail?"
- "Is this failure a product bug, a test bug, or workflow/infrastructure?"
- "What is blocking merge or release readiness?"
- "Which process/tooling fix would remove repeated CI friction?"
- "Which role should take the next follow-up?"

Prefer `coding` when the next best action is primarily a product-code fix.

Prefer `doc` when the next best action is primarily planning-surface wording,
roadmap/checklist refresh, or explanatory docs.

## Default Loop

Use this delivery loop unless the user gives a narrower instruction:

1. confirm active peers in `.agent-local/agents.json`
2. check the latest completed CI result for the previous push
3. choose one narrow scope such as one failing workflow, one flaky test family,
   or one merge-readiness blocker
4. classify the problem
5. either fix the delivery-owned problem directly or route it to the owning role
6. verify the narrow fix or evidence
7. leave a `Delivery Continuation Note`, and add a `Planning Sync Handoff` only
   when the process change affects planning-visible status

## Default CI Triage Commands

Use this command ladder by default when a `delivery` chat starts CI triage.

### 1. Check the latest completed workflow run

Start with the latest completed run on `main`:

```bash
gh run list --branch main --limit 5 --json databaseId,status,conclusion,workflowName,displayTitle,headSha,updatedAt
```

Use this to answer:

- which workflow failed
- whether the run is completed rather than still in progress
- which commit and title the failure belongs to

For this repo, the most common workflow names are:

- `CI`
- `Pages`

### 2. Inspect one failing run in detail

After choosing one failing run id, inspect the run summary:

```bash
gh run view <run-id>
```

Then inspect the failing logs:

```bash
gh run view <run-id> --log-failed
```

Use this step to classify the failure before touching code.

### 3. Map the failure to the checked-in workflow

If the failing workflow is `CI`, read:

```bash
sed -n '1,220p' .github/workflows/ci.yml
```

If the failing workflow is `Pages`, read:

```bash
sed -n '1,220p' .github/workflows/pages.yml
```

This tells `delivery` whether the failure came from:

- formatting, compile, workspace tests, or simulator smoke inside `CI`
- Pages artifact or deployment flow inside `Pages`

### 4. Reproduce only the relevant step locally

For the `CI` workflow, the default local repro ladder is:

```bash
cargo fmt --all --check
cargo check
cargo nextest run --workspace
cargo test --workspace --doc
./sim/negative-validation/smoke.py --summary-only
```

For the `Pages` workflow, the default local repro is:

```bash
npm run lint:pages
```

Then, if the question is deployment visibility rather than static-page content,
check the live Pages surface:

```bash
curl -I -L https://mycellayer.github.io/Mycel/
```

Use only the narrowest command set needed to confirm the failing step.

### 5. Check release-facing public surfaces when relevant

If the work touches outward-facing surfaces such as `README` or `pages/`, use:

```bash
gh run list -R MycelLayer/Mycel --limit 5
gh api repos/MycelLayer/Mycel/community/profile
curl -I -L https://mycellayer.github.io/Mycel/
curl -I https://mycellayer.github.io/Mycel/social-preview.png
```

Use [`docs/OUTWARD-RELEASE-CHECKLIST.md`](./OUTWARD-RELEASE-CHECKLIST.md) as
the narrow companion checklist for this case.

## Command Interpretation

Use the first matching rule:

1. `gh run list` shows the latest completed run is green
   outcome: do not invent a CI problem; move to merge-readiness or blocker
   triage instead
2. `gh run view --log-failed` points to `cargo fmt`, `cargo check`, `cargo nextest`,
   `cargo test --doc`,
   or simulator smoke
   outcome: likely `coding`, unless the root cause is obviously CI-only wiring
3. `gh run view --log-failed` points to workflow config, runner setup, caching,
   artifact upload, or deployment wiring
   outcome: likely `delivery`
4. public-surface checks fail but the underlying content is already correct
   outcome: likely `delivery`
5. public-surface checks imply wording/status drift rather than tooling failure
   outcome: hand off to `doc`

## Classification Rules

Use these buckets first:

1. product bug
   the failure is caused by real application or library behavior
   owner: `coding`
2. test bug
   the failure is caused by stale expectations, nondeterministic tests, or bad
   fixtures
   owner: usually `coding`; `delivery` may still isolate and document the cause
3. workflow or infrastructure bug
   the failure is caused by CI config, caching, runner assumptions, script
   wiring, missing setup, or release/process tooling
   owner: `delivery`
4. planning-visible process state
   the main change is status wording, readiness communication, or a planning
   surface that should reflect new delivery reality
   owner: `doc`, usually through a mailbox handoff from `delivery`

If the classification is unclear, `delivery` should narrow the uncertainty
before reassigning the work. A good first pass is often worth more than an
immediate handoff.

## Delivery-Owned Work

`delivery` should usually fix the problem directly when the work is mainly:

- GitHub Actions or workflow file updates
- CI script wiring
- cache key or setup corrections
- retry or timeout tuning with clear evidence
- release or merge gate process updates
- flaky-test isolation work that does not require a real product behavior change

Keep the scope narrow. One delivery slice should usually address one of:

- one failing workflow
- one flaky test family
- one process bottleneck
- one release-readiness blocker

## Handoff Rules

Route the work to `coding` when:

- the fix requires changing product behavior
- the failing assertion is correct and the code is wrong
- the scope turns into feature or bug implementation rather than delivery health

Route the work to `doc` when:

- roadmap or checklist wording must change
- public progress wording should reflect CI or release-readiness state
- the new delivery result changes planning-visible status for the team

When handing off:

- leave one open `Delivery Continuation Note` in the delivery mailbox
- use a `Planning Sync Handoff` for `doc` when wording or status surfaces need
  updates
- keep the handoff specific: failure surface, likely owner, evidence, and best
  next step

## Verification Expectations

Prefer evidence that matches the scope:

- latest completed CI metadata when triaging
- failing-step logs when classifying
- targeted local repro commands when confirming a workflow or script fix
- focused automated tests when a delivery-owned code path changed

When possible, record the exact `gh` or `curl` commands used in the mailbox
handoff so the next agent can reproduce the same evidence quickly.

Do not inflate the scope just to run a full repo validation pass unless the
change really needs it.

## End-Of-Cycle Output

At the end of a completed `delivery` work cycle, the mailbox should answer:

- what CI/process state was confirmed
- whether the latest blocker is delivery-owned or not
- what evidence supports that conclusion
- what the best next narrow step is

If the work landed a planning-visible process change, include a separate
cross-role note for `doc`. Otherwise, keep the delivery mailbox focused on the
current operational state.
