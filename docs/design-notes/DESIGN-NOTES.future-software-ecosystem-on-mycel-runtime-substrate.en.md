# Future Software Ecosystem on Mycel Runtime Substrate

Status: design draft

This note imagines a future where Mycel-style signed, on-demand runtime loading is no longer a niche architecture choice, but part of the mainstream software environment.

The goal is not to predict one exact future market outcome.

It is to describe what would likely change if software moved away from install-first packaging and toward trusted hosts that fetch, verify, and execute signed modules on demand.

Related notes:

- `DESIGN-NOTES.dynamic-module-loading.*` for the module-level execution model
- `DESIGN-NOTES.signed-on-demand-runtime-substrate.*` for the broader runtime-substrate framing
- `DESIGN-NOTES.minimal-mycel-host-bootstrap.*` for the smallest trusted local host that could carry this model

## 0. Goal

Describe how mainstream software culture, distribution, trust, and product structure might change if Mycel Runtime Substrate became a dominant computing model.

This note focuses on:

- software distribution
- trust and policy markets
- application structure
- user expectations
- ecosystem risks

## 1. The Primary Shift

The deepest change would not be:

- "people use Mycel apps instead of ordinary apps"

It would be:

- software stops being centered on installed packages
- software becomes centered on signed, fetchable, policy-gated capabilities

In this world, the everyday unit of software is less likely to be:

- a permanently installed app bundle

and more likely to be:

- a set of signed modules
- a host policy
- a state model
- a set of capabilities granted at runtime

## 2. Installation Becomes Secondary

In a mainstream Mycel Runtime Substrate world, "installing software" becomes less central than "authorizing a host to load certain capabilities."

Typical user flow would shift from:

- download app
- install app
- update app

to:

- open state, workflow, or document
- allow the host to resolve and load missing modules
- grant or deny requested capabilities
- keep only the modules worth caching locally

This does not eliminate installation completely.

It does make installation less like the default center of the software experience.

## 3. App Stores Become Trust and Policy Markets

Current app stores are largely:

- download catalogs
- payment channels
- approval gates

In a Mycel-dominant world, their most important role would become:

- trust distribution
- signer reputation
- capability policy packaging
- module-family admission
- compatibility guarantees

The central questions would no longer be only:

- where do I download this app?

but also:

- which signer do I trust?
- which host policies should I adopt?
- which module families are accepted under this profile or organization?

## 4. The Software Product Gets Decomposed

A mainstream software product would no longer appear mainly as one sealed binary artifact.

It would more often appear as an artifact graph:

- state schemas
- module metadata
- module blobs
- UI renderers
- policy helpers
- execution modules
- audit and receipt modules

This would increase:

- reuse
- auditability
- portability

But it would also increase:

- policy complexity
- compatibility management
- signer and dependency governance

## 5. State Becomes More Valuable Than the App Shell

A major cultural shift would be that application state becomes more important than the current app wrapper.

What users would care about most is not only:

- which app they installed

but:

- whether their state remains portable
- whether their state remains readable under different trusted module sets
- whether governance and execution history remain verifiable

This makes software feel less like:

- owning a product package

and more like:

- maintaining continuity over trusted state

## 6. Frontend, Backend, and Plugin Boundaries Blur

Today, software ecosystems often draw hard lines between:

- frontend
- backend
- plugin
- local app
- cloud service

Under a Mycel Runtime Substrate model, many of these become execution-context distinctions rather than hard software categories.

The same logical function might exist as:

- a local renderer
- a server-side policy worker
- a CLI transformer
- a browser-hosted presentation module

The important boundary becomes:

- where it runs
- what capabilities it has
- which state surfaces it may interpret or mutate

## 7. Security Moves From App Trust to Capability Trust

Today many users answer one coarse question:

- do I trust this app?

In the substrate world, the more important questions become:

- do I trust this signer?
- do I trust this module family?
- do I want this capability grant?
- may this module affect accepted-state derivation?
- may this module trigger external side effects?

This gives much better security granularity.

It also creates a new ecosystem problem:

- capability fatigue

## 8. Software Companies Change Shape

If this model became mainstream, software companies would increasingly operate as:

- signer maintainers
- policy maintainers
- schema maintainers
- compatibility maintainers
- audit and trust maintainers

Competitive advantage would shift away from:

- controlling one sealed client bundle

toward:

- maintaining trusted module families
- offering stable, well-governed schemas
- providing high-quality policy defaults
- earning long-term signer reputation

## 9. Open Source Becomes Stronger and More Structured

Open ecosystems could become more powerful in such a world because:

- modules are easier to reuse
- state formats can remain portable
- trust decisions become explicit

But open ecosystems would also become more structured around:

- signer governance
- capability review
- artifact retention
- compatibility policy

The central open-source question would shift from:

- can I build this package?

to:

- is this artifact family trusted, reviewable, and safely admissible in mainstream hosts?

## 10. Offline Use Becomes a Quality Discipline

In a fetch-on-demand world, online resolution becomes natural.

So good products would distinguish themselves by handling:

- pinned critical modules
- offline continuity
- warm-cache execution
- safe cold-cache failure

Offline support would stop being a vague marketing checkbox.

It would become a concrete discipline of:

- what was pinned
- what can be reconstructed
- what cannot be safely executed without network resolution

## 11. The Operating System Becomes More Host-Like

Traditional operating systems would not disappear, but their visible role would change.

They would increasingly be perceived as:

- host runtimes
- verifier shells
- trust and capability mediators
- module cache managers

Rather than:

- the main place where software identity lives

This makes the visible center of the stack shift upward toward:

- trusted host policy
- state model
- runtime substrate

## 12. New Power Structures Appear

This future would not eliminate power concentration.

It would change where power sits.

Likely new centers of power:

- trust-anchor maintainers
- major host vendors
- large policy registries
- artifact retention providers
- compatibility-profile authorities

This means the future ecosystem could be more open in one sense while becoming more politically structured in another.

## 13. Likely Failure Modes

Several new ecosystem-wide failure modes would become common.

### 13.1 Trust-List Monopoly

If a few hosts control the default accepted signer sets, they become the new platform gatekeepers.

### 13.2 Capability Fatigue

If capability and policy prompts are too frequent or too complex, users stop making meaningful trust decisions.

### 13.3 Artifact Availability Politics

If critical module blobs are not durably mirrored, practical software freedom collapses into whoever preserves the artifacts.

### 13.4 Governance Overload

If every useful module family requires heavy governance overhead, the ecosystem becomes unreasonably hard to use.

### 13.5 Host Vendor Overreach

If hosts become too opinionated, the supposedly open substrate becomes a disguised platform lock-in layer.

## 14. Everyday Computing in This World

For ordinary users, the dominant software experience might look like this:

- identity and state persist across hosts
- interfaces change more fluidly than underlying state
- "opening a workspace" matters more than "launching an app"
- execution becomes a matter of admitted capabilities rather than fixed installations

The result would feel less like:

- launching boxed software products

and more like:

- entering trusted computation spaces with dynamically assembled tools

## 15. Practical Summary

If Mycel Runtime Substrate became mainstream, the software ecosystem would likely move:

from:

- install-first
- app-bundle-centric
- platform-siloed

to:

- state-first
- signer- and policy-mediated
- capability-gated
- fetch-on-demand

The ecosystem would become more composable and more auditable.

It would also become more dependent on good trust governance and better user-facing policy design.

## 16. Open Questions

- Who should control the default trust anchors in mainstream hosts?
- How should capability UX avoid becoming unusable?
- Which module classes should be universally mirrorable for long-term software continuity?
- How should mainstream ecosystems distinguish portable state from signer-specific execution logic?
- At what point does a host cease to be neutral and become a platform governor?
