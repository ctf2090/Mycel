# Mycel Code Quality Checklist

Status: working checklist

This document is a recurring review list for implementation quality across the
Mycel workspace.

Use it when we are:

- reviewing a pull request or landed diff
- planning a refactor
- deciding whether a file should be split
- deciding whether repeated literals or helpers should be extracted
- checking whether tests are independent enough from product logic

It is meant to keep the codebase reviewable, composable, and easier to change
without hidden regressions.

## 1. Fast Gate

Before spending much time polishing a change, ask:

1. Is the scope small enough to review confidently?
2. Is this file or function trying to solve more than one problem?
3. Is this change repeating logic that already exists elsewhere?
4. Is this change introducing literals or structure that will be hard to update later?
5. Will the next person understand where to modify this behavior?

If the answer is unclear, narrow the change first.

## 2. Core Review Concepts

Check these concepts every time.

### 2.1 Scope and File Size

- Is the file still easy to scan in one sitting?
- Does the file mix unrelated responsibilities?
- Would splitting by concern make future review easier?
- Is one test file quietly becoming a second implementation surface?

Default bias:

- prefer smaller, purpose-shaped modules over catch-all files

Suggested warning signs:

- files growing past roughly `300-500` lines
- long sections separated only by comments instead of module boundaries
- one file containing CLI parsing, domain logic, and output formatting together

Useful tools/modules:

- `wc -l`, `rg --files`, and editor outline/symbol view for size and scanability
- `ast-grep` for repeated structural sections that suggest a split by concern
- `cloc` or repository stats tools for a quick file-size hotspot pass

### 2.2 Function Size and Intent

- Does each function do one job?
- Is the function long because of necessary domain detail, or because it lacks helpers?
- Can repeated setup or branching be named and extracted?
- Does the function name describe intent instead of mechanics only?

Default bias:

- prefer short functions with explicit names over long procedural blocks

Suggested warning signs:

- functions growing past roughly `40-60` lines without strong reason
- deeply nested branching that hides the main path
- setup, execution, and rendering mixed in one function

Useful tools/modules:

- `rg` plus editor symbol outline for long-function triage
- `ast-grep` for repeated setup, branch, or method-chain shapes worth extracting
- `clippy` for complexity-adjacent warnings and suspicious control flow patterns

### 2.3 Hard-Coded Values and Repeated Literals

- Is this literal stable domain truth, test fixture data, or an avoidable magic value?
- If the same string, number, or JSON fragment appears several times, should it become a constant or helper?
- Does the literal encode policy that belongs in a profile, config surface, or shared builder?
- Would changing this value later require touching many files?

Default bias:

- keep fixture literals local when they improve readability
- extract repeated or policy-bearing literals when they become maintenance risks

Suggested warning signs:

- repeated IDs, prefixes, timestamps, or magic numbers across many tests
- duplicated protocol version strings or object-type strings
- policy defaults copied into multiple places by hand

Useful tools/modules:

- `rg` for repeated strings, prefixes, IDs, timestamps, and object-type literals
- `ast-grep` or `comby` for repeated JSON/object construction shapes with renamed locals
- shared constants, builders, or profile/config modules when a literal carries policy meaning

### 2.4 Shared Logic vs Local Reimplementation

- Is this code reimplementing canonicalization, hashing, signing, parsing, replay, or selection logic that already exists in shared code?
- Is a test independently verifying behavior, or silently rebuilding the same implementation rules?
- Could a shared helper express the same setup with less drift risk?
- Is this "small helper" actually a second copy of product logic?

Default bias:

- prefer shared production or test-support helpers over local reimplementation

Suggested warning signs:

- local canonical JSON or signature code in tests
- copy-pasted derived ID or hash computation
- multiple modules building the same object shape by hand

Useful tools/modules:

- `ast-grep` for local copies of canonicalization, hashing, replay, or selector-like code shapes
- `rg` for calls or literals that should flow through shared helpers instead of local logic
- shared modules such as `canonical`, `signature`, `replay`, `verify`, and test-support helpers as the first places to reuse

### 2.5 Layer Boundaries

- Does CLI code stay thin while core logic stays reusable?
- Is formatting logic leaking into protocol or storage code?
- Are tests using the right layer for setup?
- Are profile or app-layer semantics leaking into shared core without need?

Default bias:

- keep boundaries explicit
- keep the core reusable and the CLI thin

Useful tools/modules:

- module layout review with `rg --files` and editor symbol/navigation tools
- `ast-grep` for CLI code that directly performs core-domain work instead of delegating
- existing crate/module boundaries such as `mycel-core`, CLI entrypoints, and store/protocol modules as the boundary map

### 2.6 Error Surfaces and Debuggability

- Do errors say what failed and where?
- Will CLI-visible failures help a user recover?
- Are assertions and `expect(...)` messages specific enough to debug quickly?
- Is failure behavior covered where the user can actually see it?

Default bias:

- prefer clear failures over vague success/failure states

Useful tools/modules:

- `clippy` for weak error-handling patterns and suspicious `unwrap`/`expect` usage
- `rg 'expect\\(|unwrap\\(|map_err\\('` for quick failure-surface review
- CLI-visible smoke tests and focused unit tests to verify the actual user-facing failure path

### 2.7 Test Quality

- Does the test describe behavior instead of implementation trivia?
- Does the test keep fixtures readable?
- Is the test overfitted to exact formatting that is not contractually important?
- Does the test duplicate production logic enough to risk false confidence?

Default bias:

- prefer behavior-focused tests with small, named builders

Suggested warning signs:

- large inline JSON blobs repeated across tests
- helper functions that reconstruct product algorithms
- assertions on incidental output instead of stable behavior

Useful tools/modules:

- `rg` for repeated fixture blobs, repeated assertions, and duplicated helper names across tests
- `ast-grep` for tests that structurally mirror production algorithms too closely
- shared test-support helpers/builders when fixture setup starts repeating across files

### 2.8 Changeability

- If we need to adjust this behavior next week, where would we edit it?
- Would the change require touching one place or many?
- Is the code organized around likely future changes?
- Are names and module boundaries helping or blocking that change?

Default bias:

- organize around expected change points, not only current convenience

Useful tools/modules:

- `git grep`/`rg` to estimate how many edit points a future change would touch
- `ast-grep` for repeated policy or construction patterns that imply future multi-file edits
- `git log -p` or blame/history review to see where changes repeatedly cluster

## 3. Repeated Review Questions

When we revisit a module, ask these six questions again:

1. Is any file or function larger than it needs to be?
2. Which literals are real fixture data, and which are maintenance debt?
3. Are we reimplementing shared logic locally?
4. Are module boundaries still clear?
5. Will failures be understandable at the user-facing surface?
6. If behavior changes, will we know the single right place to edit?

## 4. Decision Heuristics

Use these default heuristics unless there is a strong reason not to.

- Prefer one-purpose files over broad utility dumping grounds.
- Prefer named helpers over repeated setup blocks.
- Prefer shared protocol helpers over local copies of canonical rules.
- Prefer readable fixture literals over premature abstraction.
- Prefer extracting a constant only when it carries shared meaning or repeated maintenance cost.
- Prefer CLI-visible tests for user-facing behavior, and core tests for algorithmic behavior.
- Prefer refactors that reduce future edit count, not only current line count.

## 5. Minimum Review Write-Up

When we call out a code quality issue, the write-up should usually answer:

- Surface:
- Why it is hard to maintain:
- Whether it is a readability issue, drift risk, or boundary issue:
- Whether the problem is local or repeated elsewhere:
- Proposed smallest safe improvement:
- Verification plan:

## 6. Suggested Starter Checks

These are not hard rules, but they are useful default prompts:

- File size warning: over `800` lines
- Function size warning: over `100` lines
- Repeated literal warning: same non-trivial literal appears `3+` times
- Drift warning: tests or CLI helpers reimplement canonicalization, signatures, hashing, replay, or selector logic
- Boundary warning: one module mixes parsing, domain decisions, and rendering

Starter check tools/modules:

- size and hotspot scan: `wc -l`, `rg --files`, editor outline
- repeated literals: `rg`
- structural repetition or local reimplementation: `ast-grep`
- broad structural search-and-rewrite experiments: `comby`
- complexity and lint signals: `clippy`

Current CI-backed warning scan:

- warning-only: `python3 scripts/check_code_quality_hotspots.py --github-warning`
- current thresholds:
  - file size: over `800` lines
  - function size: over `100` lines
  - same non-trivial literal: `3+` repeats
- recurring GitHub issue refresh: `python3 scripts/report_code_quality_hotspots_issue.py --threshold 20`
  - intended use: on `main`, refresh the dedicated hotspot report issue whenever at least `20` commits have landed since the last reported head commit

Current CI-backed `ast-grep` gates:

- blocking: no local `canonical_json` helper definitions in `apps/mycel-cli/tests`
- blocking: no local `recompute_id` helper definitions in `apps/mycel-cli/tests`
- blocking: no local `sign_value` helper definitions in `apps/mycel-cli/tests`
- intent: keep CLI smoke tests on shared canonicalization, ID recomputation, and signing helpers so protocol drift is caught early

## 7. Relation to Other Surfaces

Use this checklist together with:

- [ROADMAP.md](../ROADMAP.md)
- [RUST-WORKSPACE.md](../RUST-WORKSPACE.md)
- [IMPLEMENTATION-CHECKLIST.en.md](../IMPLEMENTATION-CHECKLIST.en.md)
- [docs/FEATURE-REVIEW-CHECKLIST.md](./FEATURE-REVIEW-CHECKLIST.md)
- [AI-CO-WORKING-MODEL.md](./AI-CO-WORKING-MODEL.md)

If those surfaces disagree, follow the current planning-sync process in
[PLANNING-SYNC-PLAN.md](./PLANNING-SYNC-PLAN.md).
