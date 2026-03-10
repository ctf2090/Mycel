# Planning Sync Plan

Status: active working agreement for keeping planning surfaces aligned

This document defines how Mycel keeps these surfaces in sync:

- repo-level planning Markdown, especially `ROADMAP.md`, `ROADMAP.zh-TW.md`, and `IMPLEMENTATION-CHECKLIST.*`
- GitHub Issues
- GitHub Pages planning summaries

It exists to prevent drift between the authoritative build plan, the open task queue, and the public-facing progress view.

## 1. Scope

This plan applies to:

- [`ROADMAP.md`](../ROADMAP.md)
- [`ROADMAP.zh-TW.md`](../ROADMAP.zh-TW.md)
- [`IMPLEMENTATION-CHECKLIST.en.md`](../IMPLEMENTATION-CHECKLIST.en.md)
- [`IMPLEMENTATION-CHECKLIST.zh-TW.md`](../IMPLEMENTATION-CHECKLIST.zh-TW.md)
- [`docs/PROGRESS.md`](./PROGRESS.md)
- [`docs/progress.html`](./progress.html)
- curated contributor-entry issue links in [`README.md`](../README.md) and [`README.zh-TW.md`](../README.zh-TW.md)
- GitHub Issues, especially `ai-ready` task issues

It does not apply to:

- protocol wording that does not change implementation order or closure status
- purely visual homepage or support-page changes
- issue triage that does not change project planning state

## 2. Source-of-Truth Order

Use this source-of-truth order whenever surfaces disagree:

1. `ROADMAP.md` and `ROADMAP.zh-TW.md`
2. `IMPLEMENTATION-CHECKLIST.*`
3. GitHub Issues
4. `docs/PROGRESS.md`
5. `docs/progress.html`
6. landing-page summaries or support-page references
7. curated README contributor-entry issue links

Interpretation:

- `ROADMAP.md` and `ROADMAP.zh-TW.md` jointly own milestone order, phase boundaries, and build sequence.
- `IMPLEMENTATION-CHECKLIST.*` owns section-level closure state and concrete implementation gates.
- GitHub Issues represent executable slices of the remaining gaps.
- `docs/PROGRESS.md` and `docs/progress.html` are derived summaries and must not invent project state.
- curated issue links in `README.*` are contributor-facing derived summaries and must point at currently valid `ai-ready` work.

## 3. Surface Roles

### 3.1 `ROADMAP.*`

Use `ROADMAP.md` and `ROADMAP.zh-TW.md` to answer:

- what phase we are in now
- what comes next
- what the milestone sequence is
- what the current lane excludes on purpose

`ROADMAP.*` should change when:

- milestone emphasis changes
- the repo moves from one phase boundary to another
- the main missing items in the current lane have materially changed

### 3.2 `IMPLEMENTATION-CHECKLIST.*`

Use the checklist files to answer:

- what is implemented
- what is still open
- what is partial versus closeable
- which readiness gates are blocked

The checklist should change when:

- a concrete implementation item is completed
- a previously open item becomes partial or closeable
- a readiness gate changes status

### 3.3 GitHub Issues

Use GitHub Issues to answer:

- what narrow work can be executed next
- which checklist gaps have actionable slices
- what can be delegated to bots or parallel contributors

Issues should not replace roadmap or checklist state. They should reflect it.

### 3.4 Pages Progress Surfaces

Use `docs/PROGRESS.md` and `docs/progress.html` to answer:

- what a reader should understand quickly
- which milestone lane is active
- which checklist sections are mostly done, partial, or not started

Pages must stay summary-first. They should compress planning state, not define it.

## 4. Sync Rules

### 4.1 If milestone status changes

Update in this order:

1. `ROADMAP.md`
2. `IMPLEMENTATION-CHECKLIST.*`
3. `docs/PROGRESS.md`
4. `docs/progress.html`
5. open or close related GitHub Issues

Example:

- `M1` moves from “late partial” to “complete enough to start closing”
- `M2` becomes the clear active lane

### 4.2 If a checklist item closes without changing the phase

Update in this order:

1. `IMPLEMENTATION-CHECKLIST.*`
2. related GitHub Issue status
3. `docs/PROGRESS.md` if section-level status changed
4. `docs/progress.html` if public summary wording changed

Example:

- `Implement snapshot parsing` becomes complete
- but the active roadmap lane remains the same

### 4.3 If a new actionable gap is discovered

Update in this order:

1. `IMPLEMENTATION-CHECKLIST.*` if the gap is real and durable
2. open a GitHub Issue if the gap is narrow enough to execute
3. do not update `ROADMAP.md` unless the milestone emphasis changed
4. update progress summaries only if section status materially changed

### 4.4 If issue triage changes only execution shape

Update:

1. GitHub Issues only

Do not update roadmap, checklist, or pages if:

- the underlying project status did not change
- the work was only split into smaller issues
- labels or ownership changed without affecting closure state

### 4.5 If Pages wording changes only for readability

Update:

1. `docs/PROGRESS.md`
2. `docs/progress.html`

Do not change roadmap or checklist unless the underlying status changed.

## 5. GitHub Issue Mapping Rules

### 5.1 Issue source

Every planning-oriented implementation issue should map to one of:

- one checklist item
- one checklist sub-gap
- one milestone-close proof point

Avoid issues that span multiple unrelated checklist sections.

### 5.2 Recommended issue metadata

Each issue should include:

- the exact checklist section or roadmap milestone it supports
- the start files
- acceptance criteria
- verification commands
- non-goals

For bot-friendly issues, use:

- `ai-ready`
- `well-scoped`
- `tests-needed`
- `fixture-backed`
- `spec-follow-up` when applicable

### 5.3 Issue lifecycle

Use this lifecycle:

1. open when the gap is real and actionable
2. keep open while the checklist item is still materially unclosed
3. close when the narrow acceptance criteria are satisfied
4. if broader checklist closure still remains, open follow-up issues instead of leaving the original issue vague

## 6. Pages Derivation Rules

### 6.1 `docs/PROGRESS.md`

This file is the Markdown summary source for the public progress view.

It should:

- restate the active lane
- compress milestone status
- compress checklist section status
- link back to roadmap and checklist authority

It should not:

- introduce milestone names that do not exist in `ROADMAP.md`
- mark a checklist area complete if the checklist does not
- speculate about future phases beyond what roadmap already says

### 6.2 `docs/progress.html`

This file should be treated as a presentation layer over `docs/PROGRESS.md`.

When the planning state changes:

- update `docs/PROGRESS.md` first
- update `docs/progress.html` second

If there is disagreement, fix the HTML to match the Markdown summary, not the other way around.

## 7. Cadence

### 7.1 Event-driven updates

Update planning surfaces immediately when:

- a milestone meaningfully advances
- a checklist section changes state
- the active implementation lane changes

### 7.2 Commit-count refresh

Use:

```bash
scripts/check-doc-refresh.sh
```

If it reports `due`, refresh:

- `ROADMAP.md`
- `ROADMAP.zh-TW.md`
- `IMPLEMENTATION-CHECKLIST.en.md`
- `IMPLEMENTATION-CHECKLIST.zh-TW.md`
- aligned GitHub Issues
- GitHub Pages planning summary surfaces such as `docs/PROGRESS.md` and `docs/progress.html`

in the next docs-sync batch, even if no single change forced an update.

## 8. Recommended Sync Workflow

For a meaningful implementation batch:

1. land the code and tests
2. decide whether checklist status changed
3. decide whether roadmap emphasis changed
4. update GitHub Issues
5. update `docs/PROGRESS.md`
6. update `docs/progress.html`
7. run the relevant verification and doc-refresh checks

For a docs-only planning refresh:

1. refresh `ROADMAP.md`
2. refresh `IMPLEMENTATION-CHECKLIST.*`
3. realign issues
4. regenerate or manually update `docs/PROGRESS.md`
5. update `docs/progress.html`
6. ensure the GitHub Pages planning summary matches the refreshed roadmap/checklist/issues state
7. refresh curated contributor-entry issue links in `README.*` if the current starter issues changed

## 9. Anti-Drift Rules

Do not let these situations persist:

1. roadmap says the lane changed, but progress page still shows the old lane
2. checklist marks an item done, but the related issue remains open without a follow-up split
3. progress page claims a section is mostly done while the checklist is still mostly unchecked
4. issue titles drift into speculative work that the roadmap does not yet support
5. Pages introduce project status language not present in roadmap or checklist
6. README contributor-entry links point at stale, closed, or no-longer-representative issues

## 10. Minimal Done Condition

Planning surfaces are considered in sync when all of the following are true:

- roadmap milestone wording matches the actual active lane
- checklist boxes reflect current implementation closure
- open issues correspond to real remaining gaps
- `docs/PROGRESS.md` matches roadmap and checklist summaries
- `docs/progress.html` matches `docs/PROGRESS.md`
- curated README contributor-entry links still point at representative open starter issues

## 11. Current Practical Guidance for Mycel

Right now, use this concrete rule:

1. treat `ROADMAP.md` as the milestone and lane authority
2. treat `IMPLEMENTATION-CHECKLIST.*` as the closure authority
3. treat open `ai-ready` issues as narrow execution slices of checklist gaps
4. treat `docs/PROGRESS.md` and `docs/progress.html` as derived public summaries
5. treat curated `README.*` contributor issue links as narrow public entry points that should be refreshed during planning sync

This keeps roadmap, implementation closure, task queue, and public progress aligned without turning any one surface into an overloaded catch-all.
