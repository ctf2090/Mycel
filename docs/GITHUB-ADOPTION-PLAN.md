# GitHub Adoption Plan

Status: proposed short rollout for repository governance and collaboration

This plan focuses on GitHub-native features that can improve Mycel's safety,
review flow, and planning visibility without changing product behavior.

## Current Snapshot

As of 2026-03-24, the repository already uses:

- GitHub Actions workflows for CI and Pages
- issue forms and a pull request template
- GitHub Discussions for open-ended conversation
- GitHub Projects as an available planning surface
- secret scanning and secret scanning push protection
- private vulnerability reporting
- classic branch protection on `main`
- three required status checks on `main`:
  - `rust-and-validation`
  - `ast-grep-quality`
  - `code-quality-hotspots-warning`
- one required pull-request approval on `main`
- `.github/CODEOWNERS`
- `.github/dependabot.yml`
- vulnerability alerts
- automated security fixes
- GitHub code scanning default setup
- the `MycelLayer/Mycel` organization-owned repository

The main gaps worth addressing first are:

- `main` still uses classic branch protection rather than `rulesets`
- code owner reviews are not required yet
- admin enforcement is still off for the branch protection
- delete-branch-on-merge is still off
- auto-merge remains disabled

## 1. Strengthen Branch Governance

Keep improving the `main` merge gate before adding more automation.

Target settings:

- keep pull requests and the existing required CI checks in place
- decide whether to move from classic branch protection to GitHub `rulesets`
- decide whether admins should also be fully enforced by the merge gate
- decide whether code owner reviews should become required now that
  `CODEOWNERS` exists

Why first:

- this tightens the real merge gate that already exists around the workflows we
  maintain
- later features such as auto-merge become more useful only after merge
  requirements exist
- this is the highest-leverage safety improvement with the lowest ongoing
  maintenance cost

Main tradeoff:

- maintainers lose some direct-to-main speed in exchange for a safer default

## 2. Refine Ownership Routing

Refine `.github/CODEOWNERS` now that the first-pass file exists.

Suggested first-pass ownership split:

- `apps/` -> application owners
- `crates/` -> core Rust owners
- `scripts/` and `.github/workflows/` -> workflow/process owners
- `docs/` and `pages/` -> docs/public-surface owners

Why second:

- review routing matters more now that pull-request review is already part of
  the merge path
- Mycel already has clear directory boundaries that map well to ownership

Main tradeoff:

- ownership needs periodic maintenance as responsibilities evolve

## 3. Mature Dependabot And Security Adoption

Keep the existing Dependabot and vulnerability-alert setup small and
intentional.

Recommended scope:

- keep the current `dependabot.yml` focused on the ecosystems we already use
  (`cargo`, GitHub Actions, and the root `npm` workspace)
- keep private vulnerability reporting enabled for the public repository
- keep secret scanning and secret scanning push protection enabled unless
  signal quality becomes a real burden
- decide whether host-side Dependabot behavior should stay on grouped low-churn
  updates or narrow further
- keep version updates intentional until the team decides how much update churn
  it wants
- review grouped update settings if alert volume becomes noisy

Why third:

- the repository now has secret protection, Dependabot config, and vulnerability
  alerts, so the next value is tuning signal quality rather than merely turning
  the features on
- security-update pull requests fit well now that branch protections and review
  routing already exist

Main tradeoff:

- maintainers will need to triage additional automated pull requests

## 4. Evaluate Code Scanning After Enabling Default Setup

Code scanning is now enabled through GitHub's default CodeQL setup, so the next
question is how much follow-up tuning it actually needs.

Current repo fit on 2026-03-24:

- GitHub's repository default-setup API now reports `state: configured`
- enabling default setup triggered the first `CodeQL Setup` Actions run on
  commit `665bf09`
- the first `CodeQL Setup` run completed successfully across `actions`,
  `python`, and `rust`
- the first alert batch stayed small and focused: three open
  `actions/missing-workflow-permissions` warnings in `.github/workflows/ci.yml`
- the repository does not currently carry a dedicated CodeQL workflow under
  `.github/workflows/`
- existing CI already covers formatting, Clippy, compile, tests, ast-grep, and
  hotspot warnings, so CodeQL would be an additive security-analysis layer
  rather than a replacement for current checks
- the current Pages and docs-tooling surface is not the main reason to enable
  code scanning here; the best immediate fit is the Rust codebase, GitHub
  Actions workflows, and Python-based repo scripts

Recommended scope:

- keep default setup as the first pass until the initial alerts and run cost are
  visible
- use GitHub code scanning for persistent SARIF-backed findings in the Security
  tab while keeping existing CI and `ast-grep` as separate checks
- switch to advanced setup only if the first wave of results shows a real need
  for tighter event/path control, custom packs, or more explicit workflow
  ownership

Why now:

- the lower-friction security switches are already on, and the repository has
  now crossed the line from planning code scanning to operating it
- this is now an evaluation-and-tuning step rather than a feature-adoption
  placeholder

Main tradeoff:

- better static-analysis visibility in exchange for extra CI time and alert
  triage overhead

Practical decision options:

- keep default setup running and review the first alert batch before changing
  anything
- move to advanced setup only if default setup proves too noisy, misses needed
  customization, or needs tighter event/path control than the UI-managed
  defaults provide
- disable code scanning later only if the first runs show that the analysis cost
  or alert quality is materially worse than expected

Current recommendation:

- keep GitHub's default CodeQL setup in place through the first successful scan
  cycle
- treat the current three `actions/missing-workflow-permissions` alerts as a
  workflow-hardening follow-up, not as a signal that advanced setup is needed
- review the first alert batch and runtime overhead before deciding whether to
  keep default setup as-is or graduate to advanced setup
- treat advanced setup as a second-step escalation, not the default starting
  point, because this repository already has a stable CI baseline and still
  does not show a strong need for a hand-maintained CodeQL workflow

## 5. Revisit Auto-Merge After The Merge Gate Exists

Enable auto-merge only after step 1 is working well.

Why later:

- auto-merge is most useful when pull requests must wait on checks or reviews
- enabling it before the team is comfortable with the current merge gate still
  provides limited operational value

Main tradeoff:

- small convenience gain, but it can hide merge timing if the team is not yet
  comfortable with enforced review rules

## 6. Treat Projects As An Optional Planning Upgrade

GitHub Projects is worth adopting only if we want a GitHub-native planning view
for issue, PR, and role-based work tracking.

Good fit signals:

- we want one place to see coding, delivery, and doc work together
- we want custom fields such as role, scope, planning impact, or priority
- we want roadmap or status views tied directly to issues and pull requests

Why not earlier:

- Projects improves coordination clarity, but it does not reduce merge or
  security risk as directly as the first three steps
- Mycel already has local multi-agent coordination, so Projects would be an
  additive planning layer rather than a prerequisite

Main tradeoff:

- better visibility in exchange for setup and ongoing curation work

## Keep, But Do Not Expand Yet

GitHub Discussions should remain enabled for design questions, early
exploration, and public conversation, but it does not need immediate process
expansion.

Keep the current boundary:

- Discussions for open-ended design or community conversation
- Issues for tracked work
- pull requests for mergeable changes

## Not A Near-Term Candidate

Merge queue should stay deferred for now.

Reason:

- it is a strong fit for busy protected branches, but the current branch
  governance still looks light enough that merge queue would be extra process
  weight
- Mycel is organization-owned now, but current throughput still looks too small
  to justify merge-queue overhead yet

## Minimal Adoption Sequence

If we want the smallest practical rollout, use this sequence:

1. strengthen `main` governance, including the rulesets vs classic-branch-protection decision
2. refine `CODEOWNERS` and decide whether code owner reviews should be required
3. keep tuning Dependabot and the newly enabled security features
4. review the first CodeQL runs and decide whether default setup needs tuning
5. optionally enable auto-merge

This sequence keeps the change surface small while improving safety and review
discipline quickly.

## Follow-Up Work

Concrete next implementation tasks for a future work item:

- draft the exact required status checks for `main`
- keep a recurring `doc`-owned `Mature tool review` issue every `400` commits using [`.github/ISSUE_TEMPLATE/mature_tool_review.yml`](../.github/ISSUE_TEMPLATE/mature_tool_review.yml)
- use [`docs/MATURE-TOOL-REVIEW-FLOW.md`](./MATURE-TOOL-REVIEW-FLOW.md) as the runbook for collecting evidence, framing tradeoffs, and routing follow-up work
- decide whether classic branch protection should move to `rulesets`
- decide whether code owner reviews and admin enforcement should be required
- refine the first-pass `.github/CODEOWNERS`
- record which maintainers can bypass rulesets, if any
- decide whether Dependabot should stay on grouped low-churn updates or narrow
  further
- add explicit least-privilege `permissions` blocks to `.github/workflows/ci.yml`
  so the first CodeQL workflow findings close cleanly
- review the first GitHub code scanning runs and decide whether default CodeQL
  setup should stay as-is or move to advanced setup
- decide whether GitHub Projects should mirror the existing multi-agent workflow
  or stay out of scope
