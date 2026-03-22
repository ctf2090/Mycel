# Planning Sync Plan

Status: active working agreement for keeping planning surfaces aligned

This document defines how Mycel keeps these surfaces in sync:

- repo-level planning Markdown, especially `ROADMAP.md`, `ROADMAP.zh-TW.md`, and `IMPLEMENTATION-CHECKLIST.*`
- GitHub Issues
- GitHub Pages planning summaries

It exists to prevent drift between the authoritative build plan, the open task queue, and the public-facing progress view.

## 0. Sync Terms

Use these terms consistently:

- `sync doc`: Markdown-only sync. This covers planning/public-summary `.md` files such as `ROADMAP.*`, `IMPLEMENTATION-CHECKLIST.*`, `docs/PROGRESS.md`, and any related README wording.
- `sync web`: GitHub Pages-only sync. This covers Pages HTML summary surfaces such as `pages/progress.html`, localized progress pages, and non-issue landing-page wording. A `sync web` batch is only complete when every maintained language variant of the touched Pages surface is updated together.
- `sync issue`: GitHub Issues only.
- `sync plan`: the full sync. This means `sync doc` + `sync web` + `sync issue`. Any multilingual completeness rule that applies to a component sync target also applies to `sync plan`.

## 1. Scope

This plan applies to:

- [`ROADMAP.md`](../ROADMAP.md)
- [`ROADMAP.zh-TW.md`](../ROADMAP.zh-TW.md)
- [`IMPLEMENTATION-CHECKLIST.en.md`](../IMPLEMENTATION-CHECKLIST.en.md)
- [`IMPLEMENTATION-CHECKLIST.zh-TW.md`](../IMPLEMENTATION-CHECKLIST.zh-TW.md)
- [`docs/PROGRESS.md`](./PROGRESS.md)
- [`pages/progress.html`](../pages/progress.html)
- [`pages/zh-TW/progress.html`](../pages/zh-TW/progress.html)
- [`pages/zh-CN/progress.html`](../pages/zh-CN/progress.html)
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
5. `pages/progress.html`
6. landing-page summaries or support-page references
Interpretation:

- `ROADMAP.md` and `ROADMAP.zh-TW.md` jointly own milestone order, phase boundaries, and build sequence.
- `IMPLEMENTATION-CHECKLIST.*` owns section-level closure state and concrete implementation gates.
- GitHub Issues represent executable slices of the remaining gaps.
- `docs/PROGRESS.md`, `pages/progress.html`, and localized `pages/*/progress.html` summaries are derived surfaces and must not invent project state.

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

Use `docs/PROGRESS.md`, `pages/progress.html`, and localized `pages/*/progress.html` surfaces to answer:

- what a reader should understand quickly
- which milestone lane is active
- which checklist sections are mostly done, partial, or not started

Pages must stay summary-first. They should compress planning state, not define it.
When a Pages summary exists in multiple maintained languages, those localized variants should stay semantically aligned even if the phrasing is not word-for-word identical.
For localized Pages, a `sync web` review should also check visible UI labels such as section headings, stage labels, and status pills for stray source-language text.

## 4. Sync Rules

### 4.1 If milestone status changes

Update in this order:

1. `ROADMAP.md`
2. `IMPLEMENTATION-CHECKLIST.*`
3. `docs/PROGRESS.md`
4. `pages/progress.html` and localized `pages/*/progress.html`
5. open or close related GitHub Issues

Example:

- `M1` moves from “late partial” to “complete enough to start closing”
- `M2` becomes the clear active lane

### 4.2 If a checklist item closes without changing the phase

Update in this order:

1. `IMPLEMENTATION-CHECKLIST.*`
2. related GitHub Issue status
3. `docs/PROGRESS.md` if section-level status changed
4. `pages/progress.html` and localized `pages/*/progress.html` if public summary wording changed

Example:

- `Implement snapshot parsing` becomes complete
- but the active roadmap lane remains the same

### 4.3 If a new actionable gap is discovered

Update in this order:

1. `IMPLEMENTATION-CHECKLIST.*` if the gap is real and durable
2. open a GitHub Issue if the gap is narrow enough to execute
3. do not update `ROADMAP.md` unless the milestone emphasis changed
4. update progress summaries only if section status materially changed, and keep all maintained Pages language variants aligned when those summaries change

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
2. `pages/progress.html` and any maintained localized `pages/*/progress.html` counterparts

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

### 6.2 `pages/progress.html` and localized progress pages

These files should be treated as presentation layers over `docs/PROGRESS.md`.

When the planning state changes:

- update `docs/PROGRESS.md` first
- update `pages/progress.html` second
- update maintained localized `pages/*/progress.html` variants in the same batch

If there is disagreement, fix the HTML to match the Markdown summary, not the other way around. A `sync web` or `sync plan` pass that touches one maintained progress page should leave the sibling language variants semantically aligned before the batch is considered complete.
That review should include obvious localized-UI checks, especially headings, milestone card stage labels, and status-chip text, so content sync does not leave behind visible untranslated fragments.

## 7. Cadence

### 7.1 Event-driven updates

Update planning surfaces immediately when:

- a milestone meaningfully advances
- a checklist section changes state
- the active implementation lane changes

### 7.2 Commit-count refresh

Use `scripts/check-plan-refresh.sh` as the planning-refresh cadence checker.

Ownership:

- the active `doc` agent owns this check and must run it
- `coding` agents do not run this script
- instead, `coding` agents hand off sync-relevant implementation and issue-triage material through their registry mailbox so `doc` can collect it before the next planning-sync batch
- `sync doc` is due at 10 commits
- `sync issue` is due at 10 commits
- `sync web` is due at 20 commits

If it reports `due`, refresh the reported surfaces:

- `ROADMAP.md`
- `ROADMAP.zh-TW.md`
- `IMPLEMENTATION-CHECKLIST.en.md`
- `IMPLEMENTATION-CHECKLIST.zh-TW.md`
- aligned GitHub Issues
- Markdown planning surfaces such as `docs/PROGRESS.md` when `sync doc` is due
- aligned GitHub Issues when `sync issue` is due
- GitHub Pages HTML summary surfaces such as `pages/progress.html` and non-issue landing-page wording when `sync web` is due
- maintained localized Pages variants should be checked for both semantic alignment and obvious untranslated UI text when `sync web` is due

in the next planning-sync batch, even if no single change forced an update.

## 8. Recommended Sync Workflow

For a meaningful implementation batch:

1. land the code and tests
2. if the work may affect planning surfaces, leave a mailbox handoff note for `doc`
3. decide whether checklist status changed
4. decide whether roadmap emphasis changed
5. update GitHub Issues
6. update `docs/PROGRESS.md`
7. update `pages/progress.html`
8. run the relevant verification checks; the plan-refresh cadence check remains doc-owned

For a `sync plan` batch:

1. scan handoff mailboxes in this order:
   first, mailbox paths declared in the registry for active agents
   second, mailbox paths declared in the registry for paused agents
   third, mailbox paths declared in the registry for recently inactive agents that may still have unresolved planning notes
   fourth, fallback shared mailboxes such as `.agent-local/coding-to-doc.md` and `.agent-local/doc-to-coding.md` when they exist
2. ignore archived mailboxes unless a current mailbox explicitly points to an unresolved entry there
3. run the planning-refresh cadence checker
4. refresh Markdown planning surfaces such as `ROADMAP.md`, `IMPLEMENTATION-CHECKLIST.*`, `docs/PROGRESS.md`, and related README wording when `sync doc` is due
5. realign GitHub Issues when `sync issue` is due
6. update GitHub Pages HTML summary surfaces such as `pages/progress.html` and non-issue landing-page wording when `sync web` is due
7. ensure the GitHub Pages planning summary matches the refreshed roadmap/checklist/issues state

For a `sync doc` batch:

1. scan handoff mailboxes in this order:
   first, mailbox paths declared in the registry for active agents
   second, mailbox paths declared in the registry for paused agents
   third, mailbox paths declared in the registry for recently inactive agents that may still have unresolved planning notes
   fourth, fallback shared mailboxes such as `.agent-local/coding-to-doc.md` and `.agent-local/doc-to-coding.md` when they exist
2. ignore archived mailboxes unless a current mailbox explicitly points to an unresolved entry there
3. run the planning-refresh cadence checker
4. refresh Markdown planning surfaces such as `ROADMAP.md`, `IMPLEMENTATION-CHECKLIST.*`, `docs/PROGRESS.md`, and related README wording

For a `sync web` batch:

1. scan handoff mailboxes in this order:
   first, mailbox paths declared in the registry for active agents
   second, mailbox paths declared in the registry for paused agents
   third, mailbox paths declared in the registry for recently inactive agents that may still have unresolved planning notes
   fourth, fallback shared mailboxes such as `.agent-local/coding-to-doc.md` and `.agent-local/doc-to-coding.md` when they exist
2. ignore archived mailboxes unless a current mailbox explicitly points to an unresolved entry there
3. run the planning-refresh cadence checker
4. update GitHub Pages HTML summary surfaces such as `pages/progress.html` and non-issue landing-page wording

## 9. Anti-Drift Rules

Do not let these situations persist:

1. roadmap says the lane changed, but progress page still shows the old lane
2. checklist marks an item done, but the related issue remains open without a follow-up split
3. progress page claims a section is mostly done while the checklist is still mostly unchecked
4. issue titles drift into speculative work that the roadmap does not yet support
5. Pages introduce project status language not present in roadmap or checklist
6. landing-page contributor-entry links point at stale, closed, or no-longer-representative issues

## 10. Minimal Done Condition

Planning surfaces are considered in sync when all of the following are true:

- roadmap milestone wording matches the actual active lane
- checklist boxes reflect current implementation closure
- open issues correspond to real remaining gaps
- `docs/PROGRESS.md` matches roadmap and checklist summaries
- `pages/progress.html` matches `docs/PROGRESS.md`
- landing-page contributor-entry links still point at representative open starter issues

## 11. Current Practical Guidance for Mycel

Right now, use this concrete rule:

1. treat `ROADMAP.md` as the milestone and lane authority
2. treat `IMPLEMENTATION-CHECKLIST.*` as the closure authority
3. treat open `ai-ready` issues as narrow execution slices of checklist gaps
4. treat `docs/PROGRESS.md` and `pages/progress.html` as derived public summaries
5. treat README contributor guidance as Markdown-only doc copy; it should point readers at the GitHub issue list rather than curate starter issues there
6. treat contributor issue links in `pages/index.html` and localized landing pages as the public curated issue-entry surface during planning sync

This keeps roadmap, implementation closure, task queue, and public progress aligned without turning any one surface into an overloaded catch-all.
