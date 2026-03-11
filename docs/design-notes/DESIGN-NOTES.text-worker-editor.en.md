# Text-worker Editor

Status: design draft

This note describes a future editor surface for serious text work on top of Mycel.

The main design principle is:

- the editor should feel powerful enough for real text workers
- the editor should not turn presentation-layer convenience into protocol truth
- structured history and accepted-state derivation should remain visible
- rich authoring should arrive after reader-first protocol validation, not before it

See also:

- [`DESIGN-NOTES.first-client-scope-v0.1.en.md`](./DESIGN-NOTES.first-client-scope-v0.1.en.md) for why rich editing is deferred from the first client
- [`DESIGN-NOTES.mycel-app-layer.en.md`](./DESIGN-NOTES.mycel-app-layer.en.md) for the client/runtime/effect split
- [`DESIGN-NOTES.canonical-text-profile.en.md`](./DESIGN-NOTES.canonical-text-profile.en.md) for a likely reading-oriented profile family
- [`DESIGN-NOTES.commentary-citation-schema.en.md`](./DESIGN-NOTES.commentary-citation-schema.en.md) for commentary and annotation-related structures

## 0. Goal

Define what a rich editor for text workers should look like once Mycel is ready for an authoring-heavy client layer.

The target user is not a casual note taker.
The target user is a text worker such as:

- an editor of long-lived reference texts
- a scholar or commentator
- a translator
- a legal or policy drafter
- an archivist or textual curator
- a maintainer of governed documentation

The goal is not to clone one existing word processor.
The goal is to identify what those users actually need, and how that should map onto Mycel objects, history, governance, and accepted reading.

## 1. Why This May Become the First Major App-layer Demand

The text-worker editor is likely to become the first major app-layer demand on top of Mycel.

That does not mean it should be the first fully delivered app.
It means this is the first place where Mycel's protocol value turns into an obvious product need.

This demand is strong because serious text work often requires all of the following at once:

- long-lived revision history
- heavy annotation and commentary
- multiple candidate readings or branches
- explicit citation and provenance
- a visible distinction between draft state and accepted state

Those needs appear directly in legal drafting, scriptural and commentary work, historical scholarship, governed reference texts, and archival curation.

In those domains, ordinary word processors are familiar but insufficient, and protocol-inspection tooling is precise but insufficient.
Mycel therefore needs an authoring client that can bridge the two.

This is why the editor should be treated as a likely first major app-layer demand, even if reader-first protocol validation still comes earlier in the build order.

## 2. Current Tooling and Its Pain Points

Text workers who produce or maintain heavily annotated texts do not use one perfect editor today.
They usually assemble a tool stack.

### 2.1 Word Processors

Common tools:

- Microsoft Word
- LibreOffice Writer

Common users:

- legal drafters
- policy writers
- historians writing long-form prose with notes
- many religious-studies and commentary authors during drafting

Strengths:

- mature footnotes and endnotes
- comments and tracked changes
- familiar rich-text editing
- print-oriented output

Pain points:

- long documents with dense notes become hard to manage
- version branches are awkward and often collapse into file duplication
- commentary, body text, and source logic are mixed together
- revision history is visible but not strongly verifiable

### 2.2 Scholarly Typesetting Systems

Common tools:

- LaTeX
- Overleaf

Common users:

- historians
- textual scholars
- religious and classical studies authors
- citation-heavy academic writers

Strengths:

- strong footnotes, references, and cross-references
- good control over long-form structure
- strong export and publication output

Pain points:

- authoring is not comfortable for many non-technical text workers
- collaboration and commentary often feel bolted on
- the system is strong at typesetting but weak at representing competing accepted readings
- branch meaning is still mostly external to the text model

### 2.3 Structured Text and Encoding Editors

Common tools:

- TEI XML workflows
- oXygen XML Editor

Common users:

- scripture editors
- critical edition teams
- digital-humanities projects
- archivists and textual curators

Strengths:

- excellent structural precision
- explicit markup for variants, references, and semantic units
- good fit for preservation-oriented text engineering

Pain points:

- editing feels far from ordinary writing tools
- training cost is high
- reading and editing surfaces are often separate
- structure is explicit, but accepted-state and governance reasoning are still not naturally presented to end users

### 2.4 PDF, Annotation, and Research Stacks

Common tools:

- Adobe Acrobat
- Zotero
- Hypothes.is
- Obsidian

Common users:

- legal researchers
- historians
- religion and scripture researchers
- commentary-heavy scholars

Strengths:

- convenient highlighting, side notes, and research collection
- useful for source reading and excerpt gathering
- good for personal workflows

Pain points:

- annotations often remain trapped in the reading tool
- reading notes and accepted text history are split across systems
- export may preserve content but not the reasoning chain behind accepted results
- these tools help collect evidence, but they do not provide a governed text surface

### 2.5 Shared Cross-domain Pain Points

Across law, scriptural work, and historical scholarship, the repeated problem is not lack of editing tools.
The repeated problem is that the important layers are fragmented.

What is usually split apart today:

- main text
- annotations and commentary
- source references
- branch alternatives
- review and acceptance logic
- durable, replayable history

This fragmentation produces the same recurring failures:

- users can see revisions but cannot easily verify how a final state was formed
- multiple valid alternatives exist, but the default reading is hard to explain
- exported artifacts preserve text better than they preserve decision logic
- the platform or file format often becomes the hidden authority

Mycel's opportunity is not to build another generic rich-text editor.
It is to build a text-worker editor where text, annotation, branching, accepted reading, and governance remain part of one explicit and inspectable system.

## 3. Why This Needs Its Own Note

Mycel can succeed as a protocol before it succeeds as a rich editor.
That does not mean the editor question is secondary in the long run.

If Mycel is meant for serious text systems, a strong text-worker editor is eventually necessary because:

- serious users will not stay inside CLI or inspector-only tooling
- plain textarea authoring is too weak for long and structured texts
- governance without practical authoring becomes curator-only rather than worker-usable
- preservation-quality text work often includes structure, commentary, revision comparison, and controlled formatting

The editor therefore deserves explicit design boundaries rather than being treated as a generic future UI task.

## 4. Core Design Rule

The editor should behave like a capable word-processor-style client at the surface layer while preserving Mycel's protocol discipline underneath.

That means:

- users may see polished authoring affordances
- the protocol still stores canonical objects rather than opaque editor snapshots
- visual convenience must not erase branch history, object history, or governance context
- layout choices should not silently redefine accepted textual state

In short:

- rich surface
- explicit structure
- replay-safe history
- profile-governed reading

## 5. What Must Feel Familiar

The editor should feel familiar to text workers in at least these ways:

- direct text editing without visible object mechanics during ordinary writing
- block-level structure that can be moved, revised, split, merged, and annotated
- headings, lists, quotations, inline emphasis, citations, notes, and references
- robust undo and redo
- visible change tracking or revision comparison
- export or print-friendly reading views
- keyboard-centric workflows for heavy users
- copy/paste that does not destroy structure

Users should feel that the tool is capable of real editorial work, not merely protocol inspection.

## 6. What Must Not Be Copied Blindly

A Word-like experience is useful as a usability reference, but not as a storage model.

The editor should not assume:

- one mutable document blob as the canonical truth
- invisible silent overwrite of prior state
- formatting-only edits as if they were detached from structured history
- private local state as the authoritative reading state
- ad hoc local merge rules that bypass profile-governed results

The editor must not make Mycel look simpler by hiding the very properties that make Mycel valuable.

## 7. Recommended Model Split

The text-worker editor should be treated as three coordinated surfaces:

1. authoring surface
2. reading surface
3. history and governance surface

These surfaces may live in one client, but they should remain conceptually separate.

### 7.1 Authoring Surface

Used for writing, restructuring, and preparing candidate revisions.

It should optimize for:

- fast editing
- structural clarity
- annotation workflows
- revision production

### 7.2 Reading Surface

Used for reading accepted text or an explicitly chosen alternative branch.

It should optimize for:

- stable presentation
- profile-aware reading
- citation and note visibility
- minimal noise

### 7.3 History and Governance Surface

Used for understanding why one reading is accepted and what alternatives exist.

It should optimize for:

- branch inspection
- revision comparison
- provenance inspection
- accepted-head explanation
- profile and view visibility

The editor should not collapse these three concerns into one undifferentiated screen.

## 8. Recommended Text Model

The editor should be block-aware and structure-aware.

At minimum, it should assume:

- text is not one giant mutable field
- blocks are real operational units
- inline markup exists inside block content
- metadata can attach to documents, blocks, or commentary objects
- annotations may be represented by separate linked structures rather than inline-only hacks

The editor may render a smooth continuous page.
It should still preserve block identity and block history underneath.

Recommended visible units:

- title
- section heading
- paragraph
- quote
- list item
- code or literal block when relevant
- footnote or note reference
- citation anchor

## 9. Formatting Philosophy

Formatting should support meaning rather than dominate meaning.

The editor should prioritize:

- semantic structure first
- stable text reconstruction second
- rich visual presentation third

The protocol-facing authoring model should favor:

- heading levels rather than arbitrary font-size mutation
- quotation blocks rather than arbitrary indentation-only styling
- explicit citation and note structures rather than purely visual superscripts
- reusable inline marks with bounded semantics

The editor may allow presentation-oriented controls, but those should map to constrained structures where possible.

## 10. Revision and Change Tracking

A serious text-worker editor must make revision history usable.

Required behaviors:

- show draft changes before publication
- compare one candidate revision to another
- compare accepted text to a branch candidate
- show block-level additions, removals, moves, and replacements
- expose authorship and signing context where policy allows

Recommended behaviors:

- a side-by-side diff mode
- a reading-focused "clean accepted text" mode
- a "what changed and why" summary mode

The editor should not pretend that all edits are equivalent.
Structural edits, textual edits, and governance effects should remain distinguishable.

## 11. Branches, Alternatives, and Acceptance

This is where the editor most strongly differs from ordinary word processors.

The editor must make all of the following understandable:

- there may be multiple valid heads
- one accepted head is derived under the active profile
- accepted reading is not identical to "latest thing somebody typed"
- alternative heads remain meaningful, not merely stale drafts

The editor should therefore provide:

- clear branch indicators
- accepted-head visibility
- one-click reading of alternatives
- "why accepted" context
- visible separation between local draft, candidate revision, and accepted text

If this is not legible, Mycel's governance model will feel arbitrary.

## 12. Commentary and Annotation

Text work often includes more than linear prose authoring.

The editor should support:

- commentary attached to ranges or blocks
- citation anchors
- source references
- editor notes
- optional interpretation layers
- bounded dispute markers when profiles support them

Recommended rule:

- commentary should be representable as explicit objects or linked structures, not merely as visual margin text

This keeps annotation durable, addressable, and auditable.

## 13. Offline and Local-first Behavior

The editor should assume intermittent connectivity and delayed publication.

Recommended behavior:

- local drafting without immediate sync
- local persistence of drafts and branch context
- explicit publish or submit steps
- visible reconciliation after reconnect

The editor should avoid:

- pretending that unpublished local state is already globally accepted
- silently rewriting local authoring history after sync
- hiding failed publication or signature checks

## 14. Import and Export

A serious editor should not trap users in a purely internal format.

Minimum import and export goals:

- import bounded rich-text structures from common editorial sources
- export accepted text into stable reading formats
- export branch comparisons when needed for review
- preserve citations and notes where feasible

The preferred direction is:

- import convenience
- protocol normalization inside Mycel
- explicit export views out of Mycel

Mycel should not treat arbitrary external word-processor markup as canonical truth.

## 15. Explicit Non-goals

The first version of a text-worker editor should not attempt all of the following at once:

- perfect layout parity with desktop publishing tools
- full spreadsheet-like tables and page-layout engines
- unconstrained embedded objects
- real-time arbitrary multiplayer editing as the first requirement
- secret or proprietary cloud-only collaboration assumptions
- plugin ecosystems before the text core is stable

The first serious editor should be text-first, structure-first, and history-first.

## 16. Recommended Build Sequence

The text-worker editor should be built only after the reader-first core is stable.

Recommended phases:

1. accepted-text reader with branch and history inspection
2. narrow structured draft editor
3. candidate revision workflow with diff and publish
4. commentary and citation tooling
5. richer word-processor-style surface polish

This sequence matters.
If the project starts with surface polish before accepted-state clarity, the editor will look powerful while hiding the protocol's real model.

## 17. Minimal Success Criteria

A first serious text-worker editor is successful if an editor can:

1. open one document family
2. read the accepted text cleanly
3. inspect alternative heads
4. draft a structured revision without seeing raw protocol mechanics
5. compare that revision to the current accepted text
6. publish a candidate revision with explicit authorship and verification checks
7. review commentary, citations, and branch context without leaving the client

If those seven things work, Mycel has moved from protocol-only credibility to real text-work usability.

## 18. Open Questions

- Should comments and annotations be separate commentary objects by default, or may some profiles allow inline note objects inside the main text flow?
- How much presentation freedom should the editor allow before it begins to undermine canonical-text stability?
- Should publication of candidate revisions be restricted to editor-maintainers in some profiles, or should ordinary authors draft freely while governance decides visibility later?
- What is the smallest interoperable import target that makes migration from existing editorial tools practical?
- When should collaborative presence or live cursors appear, if at all, relative to accepted-state and branch clarity?
