# Viewer-Editor-View-Maintainer Checks and Balances

Status: design draft

This note proposes a checks-and-balances model for Mycel with three distinct roles:

- `viewer`
- `editor-maintainer`
- `view-maintainer`

The goal is to let editors propose content, let view maintainers govern accepted-head selection, and let viewers apply bounded public-check signals without turning Mycel into a pure popularity system.

Put more bluntly, the principle is: "I want my opponents to exist, otherwise I'll end up pushing myself until I blow past the limit."

This model also follows a governance principle: Mycel should preserve meaningful opposition. If a role no longer faces credible, durable opposition, it becomes easier for that role to reinforce itself, overextend, and push the system out of balance. The point of preserving opposition is not to manufacture hostility, but to keep braking power, scrutiny, and course correction alive inside the system.

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

### 3.1 How this principle is implemented once `viewer` enters `selector_score`

If `viewer` ever enters `selector_score` in a bounded way, the line "I want my opponents to exist, otherwise I'll end up pushing myself until I blow past the limit." stops being only an attitude statement and becomes a concrete institutional requirement:

- opponents must be able to leave measurable resistance inside the decision model, not just express sentiment from the outside
- that resistance must remain bounded, so the system does not collapse into raw popularity voting
- opposition should affect not only score but, at higher thresholds, `review` or `temporary_freeze`
- opponents themselves must still be constrained by anti-Sybil, eligibility, and signal-quality rules so fake opposition is not mistaken for real balancing power

In other words, the institutional form of this principle is not "let viewers win." It is:

- prevent `editor-maintainer` from advancing on proposal power alone
- prevent `view-maintainer` from absorbing all external dissent through a narrow internal consensus
- give `viewer` a formal but bounded channel for resistance rather than consequence-free spectator expression

In formula language, the direction is closer to:

`maintainer_score + bounded_viewer_bonus - bounded_viewer_penalty`

Combined with:

- high-confidence `viewer_challenge_pressure` can trigger `review`
- higher-threshold, higher-evidence challenge can trigger `temporary_freeze`

That is how preserving opponents stops being political rhetoric and becomes a safety mechanism inside accepted-head governance.

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

### 5.5 Penalties for Editor and View Maintainers

If three-role balancing is meant to hold, viewer challenge should not only delay outcomes. Under higher-threshold, higher-evidence conditions, it should also be able to open a formal penalty path for both `editor-maintainer` and `view-maintainer`.

`editor-maintainer` penalties are appropriate for cases such as:

- repeated submission of low-quality, spam-like, or procedurally abusive candidates
- repeated use of noisy revisions to exhaust review capacity
- coordination with fake viewer support or similar manipulation

Possible effects:

- proposal throttling or temporary proposal cooldown
- stricter candidate admission
- requiring a higher threshold before formal ratification
- in severe cases, maintainer suspension or revocation review

`view-maintainer` penalties are appropriate for cases such as:

- repeatedly ignoring high-quality challenge or clear evidence
- repeatedly absorbing outside dissent through a narrow internal bloc
- publishing procedurally abusive, misleading, or clearly bad-faith governance signals

Possible effects:

- lower or temporarily zeroed `effective_weight`
- larger corroboration quorum requirements
- suspension from solo participation in high-impact ratification
- in severe cases, maintainer suspension or revocation review

The key distinction is:

- not every viewer objection should directly penalize a maintainer
- penalties should require stronger evidence, review conclusions, or multi-party corroboration
- penalties should be treated as role-accountability measures, not just candidate-level intervention like `delay`, `review`, or `temporary_freeze`

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

`delay` is the lightest temporary intervention.

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

`temporary_freeze` is the strongest temporary intervention.

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

### If biometric authentication becomes viable

If human biometric authentication becomes reliable, privacy-preserving, and deployable at acceptable cost, it would materially change the viewer anti-Sybil design space.

Possible gains:

- easier approximation of one natural person per challenge-capable identity
- broader viewer participation can coexist with stronger civic checks
- higher-impact viewer powers such as `temporary_freeze` become easier to justify

But it still would not solve everything:

- biometrics can help distinguish people, but not guarantee judgment quality
- reputation, evidence requirements, delay windows, and abuse recovery would still matter
- privacy, exclusion, and credential-custody risks would become much more important

So even in that future, Mycel should treat biometrics as anti-Sybil substrate, not as governance legitimacy by itself.

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

### 11.1 Example `viewer` signal shape

If `viewer` ever affects `selector_score` directly, the minimal viable design should not be a single `like` counter. It should be a verifiable, bounded, typed signal shape.

Suggested minimum fields:

- `signal_id`
- `viewer_id`
- `candidate_revision_id`
- `signal_type`
- `confidence_level`
- `evidence_ref`
- `created_at`
- `expires_at`

Where:

- `signal_type` should at least distinguish `approval`, `objection`, and `challenge`
- `confidence_level` distinguishes low-cost expression from higher-commitment signaling
- `evidence_ref` is mainly for `challenge`, so it does not collapse into a heavier dislike
- `expires_at` prevents very old signals from sticking to a candidate forever

For safer deployment, signal-adjacent eligibility and weighting fields are also needed:

- `viewer_identity_tier`
- `viewer_reputation_band`
- `eligible_for_selector_bonus`
- `effective_signal_weight`

A safer direction is:

- let `approval` and `objection` enter only a bounded score channel
- let `challenge` primarily affect `review` / `freeze`, not rewrite the main score directly
- compute final `effective_signal_weight` from profile rules rather than self-reported viewer input

## 12. Viewer Balancing Strength

Under the current proposal, viewer balancing power is asymmetric.

It is relatively strong against `editor-maintainer` overreach because:

- viewers can slow candidate activation
- viewers can escalate controversial candidates into review
- editors cannot rely on proposal power alone to produce immediate accepted status

It is weaker against `view-maintainer` coordination because:

- viewers still do not control primary selector weight
- viewers cannot directly choose the accepted head
- a coordinated view-maintainer majority can usually still finalize outcomes once review pressure is cleared

So the current draft should be read as:

- strong procedural checks on editors
- moderate procedural checks on view maintainers
- limited direct public veto power

## 13. Reinforcement Options

If stronger viewer balancing is desired without turning the system into raw popularity rule, the most compatible reinforcements are:

### 13.1 Mandatory Re-Review

High-confidence viewer challenge can require an additional round of view-maintainer review before activation.

Tradeoff:

- materially stronger viewer check
- slower acceptance in contentious cases

### 13.2 High-Threshold Freeze

Viewer challenge can trigger `temporary_freeze`, but only under stricter anti-Sybil and evidence conditions than ordinary review.

Tradeoff:

- strongest civic check
- highest abuse risk if anti-Sybil is weak

### 13.3 Corroborated Freeze Release

If a freeze occurs, release should require more than the same narrow maintainer coalition that approved the candidate the first time.

Possible patterns:

- a larger view-maintainer quorum
- a minimum delay window
- independent challenge resolution or moderation review

Tradeoff:

- prevents trivial self-clearance
- adds procedural overhead

The most balanced next step for Mycel is likely:

- keep viewers out of primary selector weight
- let viewer challenge force mandatory re-review
- reserve freeze for high-trust, high-evidence cases

## 14. Three-Role Comparison: Viewer Inside vs Outside `selector_score`

If `viewer` stays outside `selector_score`:

- `editor-maintainer` remains primarily constrained by `view-maintainer` ratification, with added viewer `delay` / `review` / `freeze` pressure
- `view-maintainer` remains the primary decider, while viewer acts mainly as a procedural check
- `viewer` gains braking and challenge power, but not direct finality power

This structure tends to produce:

- strong checks on `editor-maintainer`
- moderate, mostly procedural checks on `view-maintainer`
- stronger constraints on `viewer` itself, reducing the chance of sliding into popularity capture

If `viewer` enters `selector_score` directly:

- `editor-maintainer` must win both maintainer support and viewer score
- `view-maintainer` shifts from primary governor toward a co-governor of accepted-head selection alongside viewers
- `viewer` shifts from brake to substantive governor

This structure tends to produce:

- the strongest checks on `editor-maintainer`
- the strongest checks on `view-maintainer`
- the weakest self-constraints on `viewer`, making anti-Sybil, identity admission, and signal-quality control much more central

Considering all three roles together:

- keeping `viewer` outside `selector_score` is closer to "editor proposal + maintainer ratification + viewer procedural check"
- putting `viewer` into `selector_score` is closer to "editor proposal + maintainer-viewer mixed governance"

For Mycel as it exists today, the more stable path is still to keep viewers out of primary selector weight first, while strengthening viewer challenge into mandatory re-review and high-threshold freeze.

## 15. Tradeoffs

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

## 16. Open Questions

- Should viewers ever receive direct selector weight, or only escalation power?
- Should viewer approvals affect only tie-breaks, or contribute bounded score bonuses?
- Should `temporary_freeze` require both viewer challenge and view-maintainer concurrence?
- Should viewer challenge identity be profile-local, network-global, or application-specific?
- Should low-trust viewers be allowed to trigger review but not freeze?
