# Two-Maintainer-Role Model

Status: design draft

This note proposes splitting maintainer responsibility into two explicit roles:

- a role that can publish new candidate heads
- a role that can influence accepted-head selection

Related notes:

- `DESIGN-NOTES.maintainer-conflict-flow.*` for the end-to-end flow when rival maintainer ideas become competing heads
- `DESIGN-NOTES.interpretation-dispute-model.*` for cases where the disagreement is substantive enough to become a formal interpretation dispute
- `DESIGN-NOTES.client-non-discretionary-multi-view.*` for the client-side accepted-head rules that consume these governance signals

The goal is to avoid treating "can write content" and "can decide what readers see" as the same authority by default.

## 0. Goal

Separate:

- content-production authority
- acceptance-governance authority

Preserve:

- multi-head document history
- signed governance signals
- profile-governed accepted-head selection

Allow:

- one key to hold either role or both roles

## 1. Proposed Roles

### 1.1 Editor Maintainer

An `editor-maintainer` is allowed to create new candidate document heads.

Capabilities:

- publish `patch`
- publish `revision`
- create a new branch head
- create merge revisions if allowed by the profile

Non-capabilities by default:

- no automatic influence on accepted-head selection
- no automatic selector weight

### 1.2 View Maintainer

A `view-maintainer` is allowed to publish signed governance signals that influence accepted-head selection.

Capabilities:

- publish `view`
- contribute selector signals
- accumulate selector weight under profile rules

Non-capabilities by default:

- no automatic right to create revisions just because the key can publish views

### 1.3 Dual-Role Maintainer

One key may hold both roles:

- `editor-maintainer`
- `view-maintainer`

This allows one maintainer to create new heads and also contribute to accepted-head selection, but the two authorities remain conceptually distinct.

## 2. Why Split the Roles

Without this split, the protocol risks implying that:

- whoever writes more content should also decide what readers see
- curator influence and content authorship are the same kind of authority
- active writers can dominate selection simply by being prolific

The split keeps governance clearer:

- editors create candidates
- view maintainers choose among candidates

## 3. Protocol Interpretation

Under this model:

- `revision` publishing is an editor-side action
- `view` publishing is a governance-side action
- accepted-head selection remains driven by `view` signals, not by revision authorship alone

This means a newly created head is eligible to exist in the graph even if it has no immediate accepted-head support.

## 4. Admission and Weighting

### 4.1 Editor-Maintainer Admission

A network profile may define who is allowed to publish maintainer-grade revisions.

Possible policies:

- open publication by any valid author key
- only admitted editor-maintainer keys may publish official candidate heads
- mixed mode where all authors may publish revisions, but only editor-maintainers are highlighted as formal candidates

### 4.2 View-Maintainer Admission

View-maintainer admission should remain separate.

Selector weight should be derived only from:

- valid View publication history
- profile-defined admission rules
- profile-defined penalty rules

Revision output alone should not create selector weight.

## 5. Recommended Rule Boundary

The cleanest rule boundary is:

- `patch` and `revision` authority do not imply governance weight
- `view` authority does not imply content-publishing authority
- dual-role keys must satisfy both admission paths independently

This reduces accidental concentration of power.

## 6. Accepted-Head Selection

Accepted-head selection should continue to use:

- eligible revisions as candidates
- View objects as governance signals
- fixed profile rules for weights and tie-breaks

Editor-maintainers matter because they create candidate heads.
View-maintainers matter because they influence which candidate becomes the active accepted head.

## 7. Reader and Curator Behavior

### 7.1 Reader Client

A reader client should:

- display accepted heads derived from View-maintainer signals
- show editor-produced alternative heads as branch candidates
- avoid treating editor authority as selector authority unless the same key also has the view-maintainer role

### 7.2 Curator or Governance Client

A governance-capable client should:

- verify which keys have editor-maintainer status
- verify which keys have view-maintainer status
- keep the two role assignments auditable

## 8. Data Model Options

There are three viable representation choices.

### Option A: Two Explicit Role Types

Define two role classes directly:

- `editor-maintainer`
- `view-maintainer`

Tradeoff:

- clearest semantics
- larger protocol change

### Option B: One Maintainer Type, Two Capabilities

Keep one maintainer concept, but define two capabilities:

- `can_publish_revision`
- `can_publish_view`

Tradeoff:

- smaller spec change
- weaker conceptual clarity

### Option C: Authors + View Maintainers

Let all authors create revisions, and reserve only governance power for view maintainers.

Tradeoff:

- simplest governance model
- less distinction between ordinary authors and high-trust editors

## 9. Recommended Direction

For Mycel, Option A is the clearest long-term direction:

- it matches the conceptual split between content creation and acceptance governance
- it fits the profile-governed accepted-head model
- it avoids leaking governance authority into revision authorship

If we want the least disruptive migration path, Option B is the easier short-term step.

## 10. Suggested Future Normative Language

Possible future spec wording:

- An editor-maintainer MAY publish Patch and Revision objects that create new candidate heads.
- A view-maintainer MAY publish View objects that contribute governance signals to accepted-head selection.
- Selector weight MUST be derived from View-maintainer behavior only, unless a future profile explicitly defines another signal source.
- Holding editor-maintainer status MUST NOT, by itself, grant selector weight.
- A single key MAY hold both editor-maintainer and view-maintainer status.

## 11. Open Questions

- Should editor-maintainer admission be protocol-defined or left to profile policy?
- Should all valid revisions be candidate heads, or only revisions published by admitted editor-maintainers?
- Should a dual-role key share one identity record or two role-specific records?
- Should the implementation checklist split writer, editor-maintainer, and view-maintainer flows?
