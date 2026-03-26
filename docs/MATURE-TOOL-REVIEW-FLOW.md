# Mature Tool Review Flow

Status: active recurring `doc`-role review for mature tool and module adoption

This document defines the recurring review that asks whether Mycel should adopt
an existing mature tool, module, or framework instead of extending repeated
custom maintenance.

Use this together with:

- [`AGENTS.md`](../AGENTS.md)
- [`docs/GITHUB-ADOPTION-PLAN.md`](./GITHUB-ADOPTION-PLAN.md)
- [`.github/ISSUE_TEMPLATE/mature_tool_review.yml`](../.github/ISSUE_TEMPLATE/mature_tool_review.yml)

## Purpose

The goal is not to force new dependencies on a schedule.

The goal is to create one periodic checkpoint where `doc` reviews whether
recent Mycel work shows enough repeated friction that a mature tool, module, or
framework would now reduce maintenance cost or clarify the design.

## Trigger

Run this review every `400` commits.

At that point, `doc` should open one GitHub issue using the `Mature tool review`
template.

Use one issue per review window, not one issue per candidate.

The repo automation entrypoint for this checkpoint is:

```bash
python3 scripts/report_mature_tool_review_issue.py --threshold 400 --title "Mature Tool Review" --label periodic-review
```

The GitHub Actions workflow at [mature-tool-review.yml](/workspaces/Mycel/.github/workflows/mature-tool-review.yml)
runs the same command on pushes to `main`.

## What `doc` Should Collect

Before opening the issue, `doc` should gather signals from:

- recent repeated implementation patterns in code or repo scripts
- recent CI or workflow friction that suggests a mature process tool may help
- recent roadmap, checklist, or progress notes that point to scaling pressure
- recent mailbox handoffs from `coding` or `delivery` that mention repeated
  maintenance pain

The review should stay grounded in concrete repo evidence, not generic
"best practice" shopping.

## Good Fit Signals

Open a serious candidate only when one or more of these are true:

- the same custom pattern has been implemented or patched repeatedly
- a mature tool would replace boilerplate with a clearer standard structure
- maintenance cost is rising faster than the custom implementation's value
- the repo now has enough scope that the adoption cost is justified

## Required Candidate Framing

For every candidate listed in the issue, `doc` should record:

- what concrete problem it solves now
- why now is the right time to consider it
- the main tradeoff of adoption
- whether the next step belongs to `coding`, `delivery`, or remains `doc`

If a candidate cannot be framed that concretely, it should not be listed as a
real recommendation.

## Not-Now Candidates

The issue should also include mature tools that were considered but are not a
good fit yet.

This prevents the same vague idea from being re-litigated every review cycle
without context.

## Expected Outcomes

Each review issue should end in exactly one of these outcomes:

1. no action this cycle
2. open one narrower follow-up issue for `coding`
3. open one narrower follow-up issue for `delivery`
4. schedule one bounded spike if the value is plausible but still uncertain

The periodic review issue itself should not become an implementation task.

## Suggested Search Areas

The default scan order is:

1. CLI and command-surface growth
2. repeated test scaffolding or validation matrices
3. repo scripts and workflow/process tooling
4. docs-generation or planning-sync support tooling

This keeps the review focused on areas where mature tools most often reduce
maintenance drag.
