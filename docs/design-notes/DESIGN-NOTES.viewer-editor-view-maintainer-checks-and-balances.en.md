# Viewer-Editor-View-Maintainer Checks and Balances

Status: design draft

This note proposes a checks-and-balances model for Mycel with three distinct roles:

- `viewer`
- `editor-maintainer`
- `view-maintainer`

The goal is to let editors propose content, let view maintainers govern accepted-head selection, and let viewers apply bounded public-pressure signals without turning Mycel into a pure popularity system.

Related notes:

- `DESIGN-NOTES.two-maintainer-role.*`
- `DESIGN-NOTES.client-non-discretionary-multi-view.*`
- `DESIGN-NOTES.maintainer-conflict-flow.*`

## 0. Goal

Preserve:

- deterministic accepted-head selection
- explicit candidate-head authorship
- profile-governed view-maintainer authority
- auditable decision traces

Add:

- meaningful viewer-side checks
- bounded challenge power
- clearer separation between proposal, ratification, and public objection

Avoid:

- turning accepted-head selection into simple like-counting
- letting any one role unilaterally finalize outcomes
- making viewer challenge paths trivially Sybilable

## 1. Role Model

### 1.1 Viewer

A `viewer` consumes accepted output and may emit bounded public-confidence signals.

Capabilities:

- read accepted heads and alternatives
- emit `approval`
- emit `objection`
- emit `challenge`
- emit low-severity `flag`

Non-capabilities by default:

- no direct selector weight
- no unilateral accepted-head override
- no ability to publish maintainer-grade revisions or governance Views by viewer status alone

### 1.2 Editor Maintainer

An `editor-maintainer` proposes candidate content states.

Capabilities:

- publish `patch`
- publish `revision`
- create candidate heads

Non-capabilities by default:

- no automatic selector weight
- no unilateral accepted-head finality

### 1.3 View Maintainer

A `view-maintainer` governs accepted-head selection.

Capabilities:

- publish `View` governance signals
- accumulate `effective_weight` under profile rules
- participate in accepted-head ratification

Non-capabilities by default:

- no direct content rewrite power merely by governance status

## 2. Constitutional Analogy

This model is roughly analogous to:

- editors as proposers or drafters
- view maintainers as governance-weighted ratifiers
- viewers as a bounded civic check

The accepted head remains the current valid output selected under the profile rules.
The profile itself remains the constitutional layer.

## 3. Core Principle

The system should separate three powers:

1. proposal power
2. ratification power
3. public-confidence challenge power

No role should collapse all three into one path.

## 4. Viewer Signal Types

Viewer influence should not be modeled as one undifferentiated vote count.

At minimum, distinguish:

- `approval`: positive support, mostly advisory
- `objection`: negative sentiment without full evidentiary burden
- `challenge`: a stronger claim that the candidate requires formal review
- `flag`: low-severity warning or review request

These signal types should not have identical governance effects.

## 5. Governance Effects

### 5.1 Approval

`approval` should:

- express audience acceptance
- optionally contribute a bounded public-confidence bonus
- not replace view-maintainer selector weight

### 5.2 Objection

`objection` should:

- express meaningful public resistance
- raise the bar for immediate acceptance when it crosses a threshold
- be able to trigger `delay`

### 5.3 Challenge

`challenge` should:

- require a stronger form than objection
- preferably include a reason code, citation, or evidence reference
- be able to trigger `review`
- be able to contribute to `temporary_freeze` only at a high threshold

### 5.4 Flag

`flag` should:

- record low-severity concerns
- support moderation or review triage
- not directly freeze acceptance by itself

## 6. Two-Layer Acceptance

The cleanest structure is a two-layer acceptance model.

### Layer A: Candidate Formation

This layer answers:

- which revisions are structurally valid
- which heads are eligible candidates
- whether editor admission rules narrow the candidate set

### Layer B: Governance and Public Confidence

This layer answers:

- which candidate has the highest view-maintainer selector support
- whether viewer objection or challenge should slow, review, or temporarily pause acceptance

In other words:

- editors create candidates
- view maintainers ratify among candidates
- viewers can slow or challenge ratification under bounded rules

## 7. Delay, Review, and Temporary Freeze

Viewer signals should not usually hard-select the accepted head.
They should instead control escalation.

### 7.1 Delay

`delay` is the lightest intervention.

Use it when:

- viewer objection is meaningfully elevated
- there is controversy but not enough evidence for hard review

Effect:

- postpone activation of the candidate head for a short review window

### 7.2 Review

`review` is a formal escalation step.

Use it when:

- viewer challenge crosses a threshold
- challenge evidence is non-trivial
- the governance path needs explicit re-examination

Effect:

- require additional view-maintainer review, moderation, or dispute handling before final activation

### 7.3 Temporary Freeze

`temporary_freeze` is the strongest intervention.

Use it only when:

- viewer challenge is both high-volume and high-confidence
- there is evidence of policy violation, procedural abuse, or urgent risk

Effect:

- block the candidate from becoming active until the review path resolves

This should be rare and harder to trigger than delay or review.

## 8. Why Anti-Sybil Is Required

If viewers can trigger delay, review, or freeze, then raw viewer counts become governance-relevant.

Without anti-Sybil protection:

- one actor can spawn many fake viewers
- an editor can simulate public approval
- an opponent can flood objections to permanently stall acceptance

So viewer influence requires at least one of:

- identity cost
- reputation accumulation
- governance admission
- sharply bounded viewer powers

## 9. Viewer Anti-Sybil Options

### Option A: Costly Identity

Require stake, waiting period, or another non-trivial cost before full viewer challenge power becomes active.

Tradeoff:

- stronger anti-Sybil protection
- slower onboarding

### Option B: Reputation-Based Viewer Weight

Let viewer challenge strength grow only after a history of non-abusive participation.

Tradeoff:

- aligned with long-lived civic trust
- more moving parts

### Option C: Governance-Admitted Viewers

Require explicit admission before a viewer can emit challenge-grade signals.

Tradeoff:

- easier to control abuse
- more centralized

### Option D: Bounded Civic Signals

Allow broad viewer participation, but restrict viewers to weak effects such as `delay` or `review_request`, not direct freeze power.

Tradeoff:

- safest migration path
- weaker checks

## 10. Recommended Direction

For Mycel, the safest first step is:

- keep view-maintainer selector weight as the primary ratification mechanism
- add viewer `approval`, `objection`, `challenge`, and `flag`
- let objection trigger `delay`
- let challenge trigger `review`
- reserve `temporary_freeze` for high-threshold challenge paths, ideally with stronger anti-Sybil conditions or maintainer corroboration

This preserves the current governance spine while creating real viewer-side checks.

## 11. Example Minimal Policy Shape

A future profile could define fields such as:

- `viewer_objection_delay_threshold`
- `viewer_challenge_review_threshold`
- `viewer_freeze_threshold`
- `viewer_signal_cost_model`
- `viewer_signal_weight_cap`
- `viewer_challenge_requires_evidence`

These should remain profile-level rules, not ad hoc local client settings.

## 12. Tradeoffs

Benefits:

- clearer separation of powers
- better resistance to maintainer overreach
- more visible public confidence signals
- better escalation before controversial content becomes active

Costs:

- more protocol and profile complexity
- anti-Sybil design becomes unavoidable
- challenge spam and moderation burden become real concerns
- accepted-head activation becomes less immediate in controversial cases

## 13. Open Questions

- Should viewers ever receive direct selector weight, or only escalation power?
- Should viewer approvals affect only tie-breaks, or contribute bounded score bonuses?
- Should `temporary_freeze` require both viewer challenge and view-maintainer concurrence?
- Should viewer challenge identity be profile-local, network-global, or application-specific?
- Should low-trust viewers be allowed to trigger review but not freeze?
