# Mycel Maintainer Conflict Flow

Status: design draft

This note describes how a Mycel deployment should model and preserve a conflict between maintainers over a document idea or direction without collapsing the conflict into silent overwrite.

The main design principle is:

- maintainers should compete through explicit candidate heads and governance signals
- disagreement should remain auditable even when one result becomes accepted
- accepted-head selection should resolve active presentation, not erase rival history
- conflict should be modeled as a governed flow, not as a last-writer-wins overwrite

Related notes:

- `DESIGN-NOTES.two-maintainer-role.*` for the editor-maintainer and view-maintainer split
- `DESIGN-NOTES.client-non-discretionary-multi-view.*` for accepted-head derivation rules
- `DESIGN-NOTES.interpretation-dispute-model.*` for preserving rival interpretations when the conflict is substantive rather than merely editorial
- `DESIGN-NOTES.governance-history-security.*` for preserving proposal, approval, conflict, and receipt history

## 0. Goal

Enable a deployment to answer:

- how two maintainers express rival document ideas
- how one result becomes the active accepted head
- how the losing or rival path remains visible and reviewable

This note focuses on document and interpretation conflict carried inside Mycel history.

## 1. Core Rule

Two maintainers should not "fight" by repeatedly overwriting the same visible state until one version disappears.

Instead, a conflict should proceed through:

1. rival candidate publication
2. explicit governance signaling
3. accepted-head computation
4. preserved conflict history

## 2. Roles in the Conflict

### 2.1 Editor-maintainer

The editor-maintainer may:

- publish new revisions
- create new candidate heads
- propose merges
- revise prior content in a new branch

The editor-maintainer does not automatically decide what readers see as accepted.

### 2.2 View-maintainer

The view-maintainer may:

- publish signed governance signals
- support one candidate head over another
- contribute to accepted-head selection

The view-maintainer does not automatically gain content-authoring authority merely by governing selection.

### 2.3 Reader client

The reader client should:

- compute the accepted head from verified objects and fixed profile rules
- display alternatives and trace material without silently promoting them

## 3. Basic Conflict Flow

### Step 1: One maintainer publishes a candidate

Maintainer A publishes a revision expressing idea A.

Result:

- a candidate head exists
- nothing has yet forced universal acceptance

### Step 2: Another maintainer publishes a rival candidate

Maintainer B publishes a different revision expressing idea B.

Result:

- the graph now contains rival candidate heads
- both may be legitimate candidates

### Step 3: Governance signals accumulate

View-maintainers publish signed View objects or equivalent governance signals.

Those signals may:

- support A
- support B
- remain undecided
- split by profile or epoch

### Step 4: Accepted-head selection runs

The active accepted head is computed from:

- eligible candidate heads
- valid governance signals
- fixed profile rules
- tie-break logic

Result:

- one candidate may become the active accepted head for a given profile and selection time

### Step 5: Rival history remains preserved

The non-accepted head is not deleted.

It remains:

- a branch candidate
- an alternative reading
- possible future accepted material
- evidence of the conflict itself

## 4. Two Main Conflict Types

### 4.1 Editorial Conflict

This is a conflict over wording, structure, inclusion, or document direction.

Recommended treatment:

- preserve both candidate heads
- let governance signals select the currently accepted one
- keep branch visibility for audit and future reconsideration

### 4.2 Interpretation Conflict

This is a conflict where the disagreement materially changes meaning, scope, doctrine, or application.

Recommended treatment:

- do not treat it as a mere branch preference
- open an explicit dispute object or equivalent interpretation-dispute path
- preserve both the rival interpretations and the governing resolution

## 5. What "Winning" Means

In Mycel, "winning" a maintainer conflict should usually mean:

- becoming the active accepted head under one profile and selection window

It should not mean:

- deleting the rival branch
- erasing the rival interpretation
- rewriting the conflict out of history

## 6. What Clients Should Show

### 6.1 Reader UI

A reader-oriented client should show:

- the active accepted head
- the governing profile
- enough trace information to explain why that head is active
- alternative heads as alternatives, not as invisible trash

### 6.2 Governance UI

A governance-capable client should show:

- which maintainer keys published which candidates
- which governance signals supported which head
- whether the conflict is unresolved, accepted, or disputed
- whether the disagreement is editorial or interpretive

## 7. Escalation Path

Not every conflict should escalate immediately into a formal dispute.

A practical escalation path is:

1. rival candidate heads appear
2. governance signals attempt ordinary accepted-head selection
3. if the disagreement remains high-stakes or meaning-altering, open an explicit dispute flow
4. preserve the later resolution as a separate governed record

## 8. Failure Cases

### 8.1 Silent overwrite

One maintainer's revision replaces another in the visible UI without preserved branch or trace.

Result:

- readers lose auditability
- conflict becomes invisible

### 8.2 Writer power mistaken for selector power

An editor-maintainer is treated as if publishing more revisions automatically grants acceptance weight.

Result:

- governance and authorship collapse into one authority

### 8.3 Interpretation conflict treated as minor editing

A meaning-changing dispute is handled only as a branch preference.

Result:

- the system hides substantive disagreement

### 8.4 Accepted result without visible rationale

A client shows one accepted head but not the signal path that selected it.

Result:

- acceptance looks arbitrary or absolute instead of governed

## 9. Practical Rule

If two maintainers disagree about a document idea, the system should preserve all three of these:

- the rival candidate content
- the governance process that selected the active result
- the possibility that a later profile, epoch, or dispute outcome may favor another path

That is how Mycel turns maintainer conflict into governed history rather than destructive overwrite.
