# Mycel AI Co-Working Operating Model

Status: draft operating model

This document describes how Mycel should use human maintainers and AI coding agents together.

It is not a replacement for:

- [AGENTS.md](../AGENTS.md)
- [BOT-CONTRIBUTING.md](../BOT-CONTRIBUTING.md)
- [MULTI-AGENT-COORDINATION.md](./MULTI-AGENT-COORDINATION.md)
- [PLANNING-SYNC-PLAN.md](./PLANNING-SYNC-PLAN.md)

Instead, it gives the higher-level model that connects them.

## 1. Goal

The goal is not to maximize autonomous activity.

The goal is to let Mycel accept useful AI help while keeping:

- scope narrow
- protocol behavior conservative
- planning surfaces aligned
- local verification deterministic
- pushes serial and reviewable

## 2. Core Principle

Mycel works best when humans define direction and acceptance boundaries, while agents execute narrow, verifiable slices inside those boundaries.

In practice:

- humans define milestone order and roadmap meaning
- issues turn open gaps into executable slices
- agents implement one narrow slice at a time
- local verification and CI act as gates
- humans keep final authority over ambiguity, planning state, and merge judgment

## 3. Operating Layers

### 3.1 Planning Layer

This layer decides:

- what milestone is active
- what the current lane excludes on purpose
- which checklist gaps matter now

Primary surfaces:

- [ROADMAP.md](../ROADMAP.md)
- [ROADMAP.zh-TW.md](../ROADMAP.zh-TW.md)
- [IMPLEMENTATION-CHECKLIST.en.md](../IMPLEMENTATION-CHECKLIST.en.md)
- [IMPLEMENTATION-CHECKLIST.zh-TW.md](../IMPLEMENTATION-CHECKLIST.zh-TW.md)

Humans should lead this layer.

Agents may help summarize, reshape, or sync it, but should not silently redefine milestone meaning.

### 3.2 Task Intake Layer

This layer converts planning gaps into executable work.

Primary surfaces:

- GitHub Issues
- [`.github/ISSUE_TEMPLATE/ai_ready_task.yml`](../.github/ISSUE_TEMPLATE/ai_ready_task.yml)
- [docs/LABELS.md](./LABELS.md)

The preferred issue shape is:

- one problem
- one narrow scope
- one or two primary files
- one short verification set
- explicit non-goals

This is the main handoff surface between humans and agents.

### 3.3 Execution Layer

This layer performs implementation work.

Primary surfaces:

- local chats or agent sessions
- isolated worktrees where useful
- direct code and test edits

Execution should follow:

- one agent per issue
- one active issue per agent
- one narrow scope per batch

### 3.3.1 Long-Chat Context Risk

Long-running agent chats can accumulate enough context that the host may compact earlier turns into a shorter summary. In some chat surfaces this appears as `Automatically compacting context`.

Mycel should treat that event as a risk signal, not as a harmless UI detail. After compaction, the agent may retain the main direction while losing lower-level constraints, file targets, acceptance boundaries, or earlier wording decisions.

For long-running work, the execution layer should prefer these mitigations:

- keep scope narrow enough that a chat does not need to carry too many parallel decisions
- ask the agent for a short checkpoint summary after each major step
- if context may have been compacted, restate the goal, constraints, decisions, touched files, and next step before continuing
- if any requirement is no longer certain after compaction, the agent should say so explicitly and ask for clarification instead of guessing

### 3.4 Verification Layer

This layer decides whether a task is actually complete.

Primary surfaces:

- local test commands
- fixture-backed validation
- simulator smoke checks
- CI workflows

The important rule is that an agent should not declare a task complete just because code changed. It should be complete because the named acceptance criteria and verification commands are satisfied.

### 3.5 Public Summary Layer

This layer compresses project state for readers and contributors.

Primary surfaces:

- [docs/PROGRESS.md](./PROGRESS.md)
- [docs/progress.html](./progress.html)
- README contributor guidance and landing-page contributor-entry issue links

This layer is derived. It must not invent new project state.

## 4. Recommended Agent Roles

Mycel does not need magical built-in subagents to benefit from specialization.

It can use explicit role lanes instead.

### 4.1 Planning and Docs Agent

Best for:

- roadmap/checklist sync
- README and public copy
- issue shaping
- terminology cleanup
- Pages progress summaries

Typical files:

- `README*`
- `ROADMAP*`
- `IMPLEMENTATION-CHECKLIST*`
- `docs/`

Avoid:

- protocol semantics changes hidden inside wording edits

### 4.2 Parser and Strictness Agent

Best for:

- typed parsing
- field-shape validation
- logical-ID and derived-ID strictness

Typical files:

- `crates/mycel-core/src/protocol.rs`
- direct parser tests

Avoid:

- replay or store behavior unless the issue explicitly requires it

### 4.3 Verification and Replay Agent

Best for:

- canonicalization
- verification rules
- replay invariants
- `state_hash`-adjacent closure

Typical files:

- `crates/mycel-core/src/verify.rs`
- `crates/mycel-core/src/replay.rs`

Avoid:

- broad CLI changes unless needed for verification surfaces

### 4.4 Fixture and Simulator Agent

Best for:

- negative cases
- deterministic regression inputs
- simulator validation/report coverage

Typical files:

- `fixtures/`
- `sim/`
- `apps/mycel-cli/tests/`

Avoid:

- widening protocol behavior without a matched core issue

### 4.5 Public Host and Contributor-Flow Agent

Best for:

- landing pages
- support pages
- contributor entry points
- bot-friendly workflow surfaces

Typical files:

- `docs/index.html`
- `docs/support.html`
- `BOT-CONTRIBUTING.md`
- issue templates and label docs

Avoid:

- changing planning truth instead of derived summaries

## 5. Work Intake Model

Mycel should use a hybrid model:

- issue-first for scoped feature work, bot-ready tasks, multi-commit work, handoff work, and planning-relevant work
- chat-first for tiny local corrections that are obviously narrow

Issue-first is preferred when:

1. the task will likely take more than one commit
2. the task touches more than one main file
3. the task may be handed to another agent
4. the task changes roadmap or checklist meaning
5. the task needs explicit acceptance criteria

This keeps the task queue legible for both humans and agents.

## 6. Task Routing Rules

Task routing should answer two questions:

1. should this work be done by an agent at all?
2. if yes, which role lane should own it?

### 6.1 Good Agent Tasks

Good agent tasks are:

- narrow
- deterministic
- fixture-backed or test-backed
- spec-aligned
- easy to verify locally

### 6.2 Human-Led Tasks

Human maintainers should lead:

- major roadmap shifts
- ambiguous protocol decisions
- scope redefinition
- merge judgment when multiple issue lanes collide
- high-context tradeoffs that are not yet encoded in docs

### 6.3 Shared Tasks

Some work is best done in sequence:

1. human frames the issue
2. agent implements the narrow slice
3. human reviews the interpretation and next-step impact

## 7. Parallel Execution Model

Mycel should assume multiple agents may work at once, but it should not optimize for maximum concurrency.

The preferred model is:

- one chat
- one issue
- one main file group
- one verification set

Parallel work is good when file boundaries are distinct.

Parallel work is bad when two agents both want the same primary file or the same semantic boundary.

Use [MULTI-AGENT-COORDINATION.md](./MULTI-AGENT-COORDINATION.md) for the detailed rules.

## 8. Verification Gates

Mycel should keep verification short, local, and explicit.

The ideal task ends with:

- one or two local commands
- fixture-backed proof where relevant
- no unrelated file churn

Verification should prefer:

- shared core tests before CLI-only confidence
- deterministic simulator checks where relevant
- acceptance criteria written in the issue before implementation starts

## 9. Planning Sync Model

Planning sync is part of the operating model, not a side task.

Mycel now has four planning-facing surfaces that must not drift apart:

- roadmap and checklist authority
- GitHub Issues
- public progress summaries
- public contributor-entry guidance and curated landing-page issue links

The authoritative sync entry point is:

- [PLANNING-SYNC-PLAN.md](./PLANNING-SYNC-PLAN.md)

Use:

```bash
scripts/check-plan-refresh.sh
scripts/check-plan-refresh.sh --json
```

to decide whether a planning-surface refresh is due.

## 10. Human Review and Control

Humans should remain the final control layer for:

- protocol ambiguity
- planning truth
- acceptance of milestone meaning
- whether a narrow task actually closed the intended gap

Agent output should be treated as structured implementation work, not as self-authenticating project truth.

## 11. Where Mycel Is Now

Today, Mycel is closest to this stage:

- multiple local chats may work in parallel
- issue-first work is preferred for real implementation slices
- planning, Pages, and contributor entry points are now partially synchronized
- agent specialization is explicit and social, not built into a single orchestration system

That is already useful.

It is enough to support real multi-agent progress without pretending the repo has a full autonomous orchestration layer.

## 12. Next Maturity Steps

The next useful maturity steps are:

1. keep the `ai-ready` issue pool current
2. keep planning sync cheap and regular
3. make role-lane ownership more explicit in issue claims and handoffs
4. tighten the distinction between human-only decisions and agent-friendly execution
5. later, if needed, add more formal issue-to-agent delegation on top of the current model

## 13. Short Version

Mycel's AI co-working model is:

- humans define direction
- issues encode narrow executable work
- agents implement one scoped slice at a time
- verification gates prove completion
- planning sync prevents drift
- humans keep final authority over ambiguity and project truth
