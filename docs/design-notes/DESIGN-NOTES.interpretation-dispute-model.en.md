# Interpretation Dispute Model

Status: design draft

This note describes how Mycel can model, preserve, and arbitrate interpretation disputes without silently replacing root text or forcing all deployments into one universal reading.

Related notes:

- `DESIGN-NOTES.two-maintainer-role.*` for the split between candidate-head publication and accepted-head governance
- `DESIGN-NOTES.maintainer-conflict-flow.*` for the ordinary maintainer-conflict path before or alongside formal dispute escalation
- `DESIGN-NOTES.client-non-discretionary-multi-view.*` for how accepted outputs should be derived and displayed under fixed profile rules

The main design principle is:

- root text and interpretation must remain distinct
- contradictory interpretation should become an explicit dispute object, not an invisible overwrite
- dispute outcomes may differ by profile
- alternative interpretations should remain auditable even when one interpretation becomes the active accepted reading

## 0. Goal

Enable Mycel deployments to handle interpretation disputes in a way that:

- preserves cited root text
- preserves rival interpretations
- records why one interpretation is accepted under a profile
- keeps readers aware that acceptance is governed, not absolute

This note is neutral with respect to content domain.

It can apply to any reference text system in which interpretation may diverge.

## 1. Core Rule

An interpretation that materially reverses, narrows, broadens, or redirects the apparent meaning of cited root text must not silently replace the root reading.

Instead, it should remain explicitly represented as:

- disputed
- alternative
- profile-specific
- or rejected under a defined governance process

## 2. Dispute Layers

Interpretation disputes should be handled across three layers.

### 2.1 Text Layer

This layer answers:

- what root anchors are being interpreted
- what witnesses are relevant
- whether the interpretation cites the text faithfully

This layer does not decide the final accepted interpretation by itself.

### 2.2 Interpretation Layer

This layer carries competing readings, explanations, or applications.

This is where:

- interpretation records
- interpretation disputes
- interpretation relationships

should live.

### 2.3 Governance Layer

This layer decides which interpretation becomes active under a given profile.

This may result in:

- one accepted interpretation
- one accepted interpretation plus alternatives
- no accepted interpretation yet
- profile-specific divergence

## 3. Dispute Types

Not all disagreements are the same.

Recommended dispute classes:

- `textual-meaning`
- `scope-of-application`
- `contextual-reading`
- `translation-reading`
- `commentary-conflict`
- `practical-application`
- `doctrinal-divergence`

These classes help clients and governance processes distinguish lightweight disagreement from high-stakes contradiction.

## 4. Core Record Families

### 4.1 Interpretation Record

Represents one proposed interpretation.

Suggested fields:

- `interpretation_id`
- `work_id`
- `witness_id`
- `anchor_refs`
- `interpretation_kind`
- `body`
- `submitted_by`
- `source_mode`
- `created_at`

Suggested `interpretation_kind` values:

- `direct-reading`
- `contextual-reading`
- `comparative-reading`
- `commentary`
- `practical-application`
- `minority-reading`

### 4.2 Interpretation Dispute Record

Represents a formal dispute around one or more interpretations.

Suggested fields:

- `dispute_id`
- `work_id`
- `anchor_refs`
- `dispute_type`
- `candidate_interpretations`
- `dispute_reason`
- `opened_by`
- `opened_at`
- `status`

Suggested `status` values:

- `open`
- `under-review`
- `accepted-under-profile`
- `closed-with-alternatives`
- `rejected`

### 4.3 Interpretation Resolution Record

Represents the governed outcome of a dispute.

Suggested fields:

- `resolution_id`
- `dispute_id`
- `accepted_interpretation`
- `alternative_interpretations`
- `rejected_interpretations`
- `accepted_under_profile`
- `decision_trace_ref`
- `rationale_summary`
- `resolved_at`

### 4.4 Interpretation Citation Set

Represents the textual basis for an interpretation or dispute.

Suggested fields:

- `citation_set_id`
- `interpretation_id`
- `anchor_refs`
- `quoted_segments`
- `supporting_notes`
- `alignment_refs`

## 5. Minimum Admissibility Rule

An interpretation should not enter formal dispute review unless it provides:

- explicit anchor references
- a readable rationale
- enough citation context to be auditable

This does not guarantee acceptance.

It only guarantees that the interpretation is reviewable.

## 6. Root Text Protection

The following should be forbidden:

- silently rewriting cited root text to fit one interpretation
- treating commentary as if it were the root witness
- collapsing all rival interpretations into one edited synthetic reading without preserved history

The following should be allowed:

- explicit rival interpretations
- profile-specific acceptance
- minority or alternative interpretation preservation

## 7. Governance Outcomes

Interpretation review should support more than binary win or lose outcomes.

Recommended outcomes:

- `accepted-as-default-reading`
- `accepted-as-profile-specific-reading`
- `accepted-as-minority-reading`
- `kept-as-commentary-only`
- `rejected-as-contrary-to-cited-text`
- `left-unresolved`

This gives deployments a richer way to preserve disagreement without pretending every conflict must collapse into one absolute result.

## 8. Role Boundaries

This model should distinguish at least:

- text-maintenance roles
- interpretation-publication roles
- dispute-review roles
- accepted-reading governance roles

These roles may overlap, but one role should not automatically imply all others.

In particular:

- publication of an interpretation should not automatically grant governance weight
- governance acceptance should not imply that root text itself has changed

## 9. Client Behavior

A conforming reader client should:

- show one active accepted interpretation if the active profile defines one
- show when an interpretation is disputed
- keep alternative interpretations visible
- show citations and dispute rationale
- distinguish root text from commentary and interpretation

The client should not:

- silently hide every rival interpretation
- present profile acceptance as if it were universal truth

## 10. Example Flow

Recommended flow:

1. a reader or maintainer submits an interpretation record
2. another party opens an interpretation dispute
3. citation sets and anchor references are attached
4. reviewers or maintainers examine the dispute
5. a profile-specific interpretation resolution is published
6. reader clients display the accepted interpretation plus visible alternatives

## 11. Relation to Canonical Text Profile

This model is an upper-layer companion to the canonical text profile.

The canonical text profile handles:

- works
- witnesses
- anchors
- commentary layers
- accepted readings

This dispute model adds:

- rival interpretation records
- dispute objects
- interpretation resolution outcomes

## 12. Minimal First Version

A minimal first deployment of this model should require only:

- interpretation records
- dispute records
- resolution records
- citation support through anchor references
- one accepted profile-aware outcome

It should not require:

- automated semantic classification
- complex scoring of interpretive schools
- global agreement across all deployments

## 13. Recommended Next Step

After this note, the next practical step should be:

- a minimal interpretation dispute schema
- one example dispute flow over a canonical text
- client mockups for `Why this interpretation` and `Alternative interpretations`
