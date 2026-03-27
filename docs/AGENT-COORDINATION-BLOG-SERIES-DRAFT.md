# Agent Coordination Blog Series Draft

Status: draft writing plan for a public blog series about the Mycel
agent-coordination stack

This note proposes a first blog series for explaining the ideas behind the
Mycel agent-coordination stack.

The goal is not to present Mycel as "another agent framework."

The goal is to explain a practical operating model for making multiple AI
coding agents work reliably in one repository without relying only on prompt
memory, luck, or a single supervisor process.

## Series Positioning

Working positioning:

- Multi-agent coding is not mainly an orchestration problem.
- For real teams, it is a coordination problem.

The series should focus on:

- concrete failure modes
- design choices
- tradeoffs
- repo-local coordination patterns
- lessons from running the stack in a real repository

The series should avoid sounding like:

- generic "future of AI" commentary
- a product announcement for a framework we have not extracted yet
- a claim that process replaces model quality

Instead, the voice should be:

- field notes from practice
- design patterns for serious teams
- honest about overhead, not hype-driven

## Target Reader

Primary readers:

- maintainers already using AI coding agents in real repos
- teams experimenting with multiple parallel AI chats
- people who have already seen context loss, duplicate work, or mysterious "who changed what?" failures

Secondary readers:

- agent-framework builders
- engineering managers evaluating how to operationalize coding agents
- advanced solo developers who want durable workflows, not one-off demos

## Recommended Publishing Shape

Recommended first format:

- a 10-post blog series

Why this shape:

- easier to publish than a book
- lets us test which concepts resonate
- gives us reusable raw material for a future handbook or OSS README

Recommended release order:

1. start with the problem framing
2. then introduce the core patterns
3. then show the harder operational problems
4. end with product and ecosystem implications

## Series Outline

### 1. Multi-Agent Coding Is a Coordination Problem, Not Just an Orchestration Problem

Core thesis:

- Most public work on multi-agent systems focuses on delegation, routing, and tool use, but day-to-day coding failures usually come from poor coordination: unclear ownership, context drift, implicit handoffs, and stale state.

What this article should do:

- establish the main argument for the whole series
- distinguish orchestration from coordination
- introduce the idea that repo-local process matters

Why publish first:

- this is the clearest framing hook for the entire series

### 2. What Actually Manages Agent Workflow: The Repo-Native Scripts We Use

Core thesis:

- The practical coordination layer is often a small set of repo-native scripts and files, not one hidden supervisor runtime.

What this article should do:

- explain which concrete scripts and files currently manage bootstrap, registry, workcycle, handoff, Git boundaries, and planning refresh
- show why explicit executable guardrails beat invisible convention
- frame the coordination layer as inspectable repository state, not only chat behavior

### 3. Why Chat Memory Is Not a Coordination System

Core thesis:

- A long chat can remember intent for a while, but chat memory alone cannot serve as a durable team coordination surface once multiple agents or resumed sessions are involved.

What this article should do:

- explain why chat history is fragile
- show how compaction, interruption, and parallel work break assumptions
- motivate external coordination artifacts

### 4. The Smallest Useful Primitive Is a Registry, Not a Swarm

Core thesis:

- Before teams need a sophisticated swarm runtime, they usually need a reliable registry of who exists, what role they hold, and whether they are actually still active.

What this article should do:

- explain why identity and lifecycle tracking come before fancy agent topologies
- introduce stable agent UID vs human-friendly display ID
- argue for explicit state over implicit assumptions

### 5. Handoffs Should Be First-Class Artifacts

Core thesis:

- Handoffs are not just summaries; they are the operational bridge that lets another agent or a future session continue safely without rereading everything or guessing.

What this article should do:

- define what a good handoff contains
- explain why handoff quality matters more than verbose chat logs
- show the difference between a status note and a continuation artifact

### 6. Workcycles Beat Vague “In Progress” State

Core thesis:

- Multi-agent coding becomes much easier to reason about when every user-command cycle has a clear begin/end boundary, instead of one endless notion of "the agent is working."

What this article should do:

- explain explicit workcycle semantics
- show why begin/end boundaries help with bookkeeping, closeout, and handoff freshness
- connect lifecycle boundaries to verification discipline

### 7. Git Ownership Is a Coordination Primitive

Core thesis:

- In coding workflows, coordination is incomplete until it includes file ownership, commit boundaries, and push discipline.

What this article should do:

- argue that Git process is not separate from agent coordination
- explain why shared repos need explicit anti-collision rules
- connect code ownership to handoff and issue boundaries

### 8. Stale-Active Agents Are a Real Operational Failure Mode

Core thesis:

- In real environments, agents do not always end cleanly; disconnects, compacted sessions, and abandoned chats leave "active" state behind, so teams need stale-active detection and reconciliation.

What this article should do:

- introduce the stale-active problem as a first-class operational issue
- explain why this is different from simple crash recovery
- show why lifecycle state needs external evidence

### 9. Human Control Matters More After the Agents Get Better

Core thesis:

- As coding agents improve, the risk shifts from basic inability to subtle overreach, which makes human control surfaces, planning authority, and review boundaries more important, not less.

What this article should do:

- reject the idea that stronger models remove the need for process
- explain human role in planning, acceptance criteria, and ambiguity handling
- position the system as human-led, agent-executed

### 10. What Existing OSS Multi-Agent Stacks Solve, and What They Mostly Do Not

Core thesis:

- Existing OSS multi-agent frameworks are strong at runtime orchestration, but most do not treat repo-local coordination, handoff governance, Git ownership, or stale-active recovery as first-class problems.

What this article should do:

- summarize the OSS landscape comparison
- explain where our stack overlaps with frameworks like AutoGen, CrewAI, LangGraph, Agent Squad, and AgentBase
- clarify the niche of a Git-native coordination layer

### 11. From Internal Process to Reusable Coordination Layer

Core thesis:

- The most reusable part of this work is not the full Mycel process bundle; it is the coordination core: registry, workcycle, handoff, checklist, and reconcile.

What this article should do:

- connect the series back to the possible OSS spinout
- explain the difference between a reusable coordination core and project-specific policy
- end with a credible future direction instead of hype

## Suggested Priority Order

If we only write the first three posts soon, use this order:

1. `Multi-Agent Coding Is a Coordination Problem, Not Just an Orchestration Problem`
2. `What Actually Manages Agent Workflow: The Repo-Native Scripts We Use`
3. `Why Chat Memory Is Not a Coordination System`

If we want a stronger operations-first arc, use this order:

1. `Multi-Agent Coding Is a Coordination Problem, Not Just an Orchestration Problem`
2. `What Actually Manages Agent Workflow: The Repo-Native Scripts We Use`
3. `Workcycles Beat Vague "In Progress" State`

## Suggested Writing Style

Recommended style:

- short and concrete
- technical but not academic
- examples before abstractions
- honest about overhead and tradeoffs

Recommended structure per article:

1. one concrete failure mode
2. one missing coordination primitive
3. one design response
4. one tradeoff
5. one takeaway for teams

## Suggested Reusable Themes

Themes worth repeating across the series:

- state should be explicit
- handoff should outlive chat memory
- coordination artifacts should be repo-local when possible
- lifecycle boundaries reduce confusion
- process is not anti-agent; it is what makes agents usable in teams

## Current Recommendation

Best next move:

- write post 1 first
- use [`docs/BLOG-PUBLISHING-PLAN.md`](./BLOG-PUBLISHING-PLAN.md) as the companion note for v1 hosting, URL, and publishing-structure decisions

Why:

- it gives the entire series its framing
- it can stand alone even before any tool is extracted
- it creates the conceptual hook for later posts, talks, or an OSS README
