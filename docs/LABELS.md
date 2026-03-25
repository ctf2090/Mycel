# GitHub Label Guide

Status: draft

This page explains the tracked bot/task labels defined in [`.github/labels.yml`](../.github/labels.yml).

Use these labels together with:

- [BOT-CONTRIBUTING.md](../BOT-CONTRIBUTING.md)
- [`.github/ISSUE_TEMPLATE/ai_ready_task.yml`](../.github/ISSUE_TEMPLATE/ai_ready_task.yml)
- [`scripts/sync-labels.py`](../scripts/sync-labels.py)
- [`scripts/check-labels.py`](../scripts/check-labels.py)

## Source of Truth

The repo-tracked source for custom bot/task labels is:

- [`.github/labels.yml`](../.github/labels.yml)

Sync and verify with:

- `scripts/sync-labels.py` for syncing tracked labels
- `scripts/check-labels.py` for verifying tracked labels against GitHub

## Label Meanings

### `ai-ready`

Use when an issue is ready for direct bot execution.

Expected shape:

- clear problem statement
- explicit scope
- verification commands
- concrete acceptance criteria

### `well-scoped`

Use when the task is tightly bounded.

Typical signs:

- one subsystem
- a small set of files
- limited side effects
- low ambiguity about what “done” means

### `heavy-lift`

Use when the task is substantial even if it is still well-scoped.

Typical signs:

- multiple related steps
- non-trivial refactor or closure work
- likely more than a quick patch

Do not use this for tiny cleanup or simple docs edits.

### `spec-follow-up`

Use when the task directly closes a gap from:

- `PROTOCOL`
- `WIRE-PROTOCOL`
- `ROADMAP`
- `IMPLEMENTATION-CHECKLIST`
- relevant design notes

This label is especially useful for protocol-to-code closure tasks.

### `tests-needed`

Use when the task should not be considered complete without new or updated:

- unit tests
- smoke tests
- negative tests
- fixture-backed regression coverage

This should often accompany code-path changes.

### `fixture-backed`

Use when checked-in fixtures should drive, reproduce, or verify the task.

This is especially useful for:

- validation rules
- parser strictness
- replay behavior
- negative cases

### `blocked-by-spec`

Use when implementation should pause until a clearer spec or design decision exists.

Typical signs:

- conflicting interpretations
- missing normative rule
- unclear profile or governance semantics

Prefer this over pushing speculative implementation into code.

## Recommended Combinations

Common issue label combinations:

- `ai-ready` + `well-scoped`
  For narrow tasks with a clear entry point.
- `ai-ready` + `well-scoped` + `tests-needed`
  For small implementation tasks that must land with coverage.
- `ai-ready` + `heavy-lift` + `spec-follow-up`
  For larger protocol-to-code closure work.
- `ai-ready` + `fixture-backed` + `tests-needed`
  For validation and regression tasks driven by checked-in examples.
- `blocked-by-spec`
  Use alone or with minimal context labels when code work should wait.

## Practical Rule

If you are unsure:

1. always keep `ai-ready` on the AI-ready task template
2. add `well-scoped` when boundaries are tight
3. add `tests-needed` when verification must expand
4. add `fixture-backed` when fixtures should carry the proof
5. add `spec-follow-up` when the issue exists to close documented planning or spec debt
6. switch to `blocked-by-spec` when implementation would otherwise guess
