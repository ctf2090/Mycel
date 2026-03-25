# Agent OSS Boundary Draft

Status: draft boundary proposal for separating reusable agent-coordination tooling from Mycel-specific workflow policy

This note proposes a first-pass split between:

- a possible standalone OSS project for multi-agent coding coordination
- Mycel-specific process, policy, and project-state material that should stay in this repo

It is intentionally practical rather than idealized. The goal is to identify a
small reusable core that could plausibly help other teams without forcing them
to adopt Mycel's whole operating model.

## Product Boundary

Working product hypothesis for a future OSS spinout:

- Git-native multi-agent coordination toolkit for coding teams

The likely reusable core is:

- agent identity and lifecycle tracking
- per-command work-cycle begin/end handling
- mailbox-style handoff records
- checklist copy and state tracking
- stale-active reconciliation from persisted local evidence
- optional bootstrap helper around those primitives

The following are **not** part of that reusable core by default:

- Mycel milestone or roadmap policy
- Mycel role policy (`coding` / `doc` / `delivery`)
- Mycel planning-sync cadence
- Mycel issue or Pages progress workflow
- Mycel commit-message and push discipline specifics
- Mycel language, timezone, or user-local overlays

## Keep In OSS Core

These files look like reusable coordination primitives and should be considered
strong OSS-core candidates.

### Core coordination scripts

- `scripts/agent_registry.py`
  Owns agent identity, role assignment, lifecycle state, recovery, takeover, and cleanup.
- `scripts/agent_work_cycle.py`
  Wraps per-command begin/end transitions and centralizes work-cycle bookkeeping.
- `scripts/agent_timestamp.py`
  Provides canonical timestamp lines without hand-written formatting drift.
- `scripts/agent_bootstrap.py`
  Likely reusable if it becomes config-driven instead of Mycel-role-driven.
- `scripts/mailbox_handoff.py`
  Encodes portable handoff templates and supersede-and-append behavior.
- `scripts/mailbox_gc.py`
  Reusable mailbox retention and orphan cleanup.
- `scripts/agent_checklist_gc.py`
  Reusable checklist retention cleanup.
- `scripts/item_id_checklist.py`
  Portable checklist materialization from tracked docs.
- `scripts/item_id_checklist_mark.py`
  Portable checklist state mutation for automation and terminal use.
- `scripts/agent_registry_reconcile.py`
  Strong OSS candidate because stale-active recovery is a general multi-agent problem.

### Supporting scripts that may belong in OSS core

- `scripts/check-runtime-preflight.py`
  Generic enough if kept small and tool-agnostic.
- `scripts/codex_thread_metadata.py`
  Reusable only if the OSS project explicitly targets Codex-hosted agent sessions.
- `scripts/codex_token_usage_summary.py`
  Reusable only if Codex token snapshots remain part of the chosen product shape.
- `scripts/codex_compaction_detector.py`
  Reusable as an optional adapter for hosts with compaction-like behavior.

### Candidate OSS-core documentation

- `docs/AGENT-REGISTRY.md`
  Good candidate after removing Mycel-specific role and planning wording.
- `docs/AGENT-HANDOFF.md`
  Good candidate after trimming Mycel-only examples and planning-sync references.
- `docs/ROLE-CHECKLISTS/README.md`
  Good candidate as a generic checklist-system guide.
- `docs/MULTI-AGENT-CHEATSHEET.md`
  Good candidate as a short operator guide after making it repo-neutral.

## Keep In Mycel

These files are primarily project policy, domain workflow, or Mycel operating
model. They should stay in Mycel even if a reusable coordination toolkit is
spun out.

### Canonical repo policy

- `AGENTS.md`
  This is Mycel's shared working agreement, not a generic product guide.
- `AGENTS-LOCAL.md`
  Explicitly local and user-specific by design.
- `docs/ROLE-CHECKLISTS/coding.md`
- `docs/ROLE-CHECKLISTS/doc.md`
- `docs/ROLE-CHECKLISTS/delivery.md`
  These capture Mycel role policy rather than reusable coordination primitives.

### Mycel operating-model and team-process docs

- `docs/AI-CO-WORKING-MODEL.md`
  Explains how Mycel wants humans and agents to collaborate around planning and protocol work.
- `docs/MULTI-AGENT-COORDINATION.md`
  Mostly Mycel team process, issue mode, file ownership expectations, and push discipline.
- `docs/BOOTSTRAP-TOKEN-ANALYSIS.md`
  Mycel-specific analysis and diagnostics.
- `docs/OUTWARD-RELEASE-CHECKLIST.md`
  Release policy is repo-local, not coordination-core.
- `docs/PLANNING-SYNC-PLAN.md`
  Planning cadence is Mycel policy.

### Mycel-specific helper scripts

- `scripts/check-plan-refresh.py`
- `scripts/check-plan-refresh.sh`
  These are planning-sync policy tools, not generic coordination primitives.
- `scripts/inactive_coding_handoffs.py`
  Too tightly coupled to a specific Mycel role and handoff interpretation.
- `scripts/gh_issue_comment.py`
  Useful, but it belongs to Mycel's GitHub workflow layer, not the coordination core.
- `scripts/agent_push.py`
  The explicit direct-to-`main` push helper reflects Mycel's Git identity and branch policy.
- `scripts/agent_safe_commit.py`
  Potentially reusable in part, but currently tied to Mycel commit-trailer and push-discipline conventions.
- `scripts/render_files_changed_table.py`
- `scripts/render_files_changed_from_json.py`
  More general engineering-report utilities than agent-coordination core.
- `scripts/check_code_quality_hotspots.py`
  Repo-specific engineering quality workflow.

## Gray Zone: Needs Abstraction First

These files or concepts could move into OSS later, but only after we separate
generic mechanics from Mycel policy.

### `scripts/agent_bootstrap.py`

Keep if it becomes:

- config-driven role bootstrap
- host-adapter-aware
- optional CI baseline hooks
- optional handoff-awareness hooks

Keep in Mycel if it remains:

- tightly bound to Mycel role names
- tightly bound to Mycel CI expectations
- tightly bound to Mycel startup transcript style

### `scripts/agent_safe_commit.py`

Reusable core idea:

- allowlist staging
- fail-closed commit creation
- agent-identity trailers

Mycel-specific layer:

- exact trailer schema
- exact git identity format
- exact push and branch policy

### `scripts/codex_thread_metadata.py` and Codex-specific evidence helpers

These belong in OSS only if the project explicitly chooses to support:

- Codex-hosted agents as a first-class runtime
- host adapters for other agent shells later

If the future OSS project wants to be provider-neutral from day one, these
should move behind an adapter boundary instead of living in the generic core.

## Recommended Split Strategy

Do not extract the whole agent process at once.

Use a three-layer model instead:

1. OSS coordination core
   Includes registry, work-cycle, handoff, checklist, cleanup, and liveness reconcile.
2. Host adapters
   Includes Codex-specific metadata, compaction detection, token snapshots, and any future Claude Code or other-host adapters.
3. Mycel policy pack
   Includes roles, planning-sync policy, GitHub workflow expectations, CI policy, roadmap and issue conventions, and local overlays.

## Candidate First OSS Repository Contents

A realistic minimal first version would include only:

- `scripts/agent_registry.py`
- `scripts/agent_work_cycle.py`
- `scripts/agent_timestamp.py`
- `scripts/mailbox_handoff.py`
- `scripts/mailbox_gc.py`
- `scripts/item_id_checklist.py`
- `scripts/item_id_checklist_mark.py`
- `scripts/agent_registry_reconcile.py`
- one generic bootstrap wrapper
- one generic README
- one demo config

Optional adapters:

- Codex metadata helpers
- compaction detection helpers

Excluded from first OSS cut:

- planning-sync scripts
- issue-comment scripts
- direct push helpers
- Mycel roadmap and doc-process rules
- Mycel role checklists

## Suggested Next Refactor Steps

Before creating a separate repository, we should do these in Mycel first:

1. Introduce an explicit config boundary for roles, templates, stale thresholds, and retention rules.
2. Move Codex-specific liveness evidence behind a host-adapter interface.
3. Separate generic handoff templates from Mycel planning-sync or role-specific examples.
4. Separate generic registry lifecycle docs from Mycel workflow policy.
5. Rename Mycel-specific scripts or docs where their names currently imply portability they do not yet have.

## Current Recommendation

Recommendation: pursue a spinout only for the coordination core, not for the
full Mycel agent process.

Why:

- the coordination core appears genuinely reusable
- the current Mycel process layer is valuable, but mostly as an opinionated example and policy pack
- separating those layers will make both projects clearer

Short version:

- OSS project candidate: registry + workcycle + handoff + checklist + reconcile
- Keep in Mycel: roles + planning + CI/push policy + roadmap/issue/docs workflow
