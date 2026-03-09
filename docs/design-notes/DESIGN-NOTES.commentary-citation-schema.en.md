# Commentary and Citation Schema

Status: design draft

This note defines a narrow app-level schema for maintainer-authored commentary works that heavily cite one or more source documents without overwriting them.

These schemas are not new core protocol primitives.
They are logical record shapes intended to live inside normal Mycel documents as an app-layer model.

## 0. Goal

Enable Mycel deployments to support this pattern:

- one source document remains the root text
- one maintainer or editor publishes a separate commentary work
- the commentary work cites the source text heavily
- readers can inspect commentary and source together without collapsing them into one rewritten document

This note is neutral with respect to content domain.

It can apply to legal commentary, technical glosses, scholarly notes, editorial explanation, or profile-governed interpretation layers.

## 1. Core Rule

A commentary work must remain distinct from the cited source work.

The following should be preserved as separate things:

- source text
- commentary text
- citation links
- profile-specific acceptance of commentary or interpretation

Commentary must not silently masquerade as the source witness.

## 2. Scope

This minimal schema covers four record families:

- `commentary_work`
- `commentary_section`
- `citation_set`
- `commentary_resolution`

The goal is to make heavily cross-referenced commentary reviewable, machine-checkable, and UI-friendly without expanding the protocol core.

## 3. General Rules

1. All records are app-level JSON payloads and should follow Mycel canonical serialization rules when signed or hashed through normal document flows.
2. IDs shown here are logical app IDs, not new protocol object types.
3. Unknown fields should be rejected in a strict profile and ignored only if an app profile explicitly allows extensions.
4. Timestamps are integer Unix seconds.
5. A commentary document should be its own `doc_id` even when it is tightly linked to one source work.
6. Cross-document citation should prefer stable logical references such as `source_id` plus `locator`; a profile may also require version locking fields.

## 4. Shared Field Conventions

### 4.1 Common Required Fields

Each record family should carry:

- `type`
- `app_id`
- `created_at`
- `updated_at`

### 4.2 Recommended ID Prefixes

Recommended logical ID prefixes:

- `cw:` for commentary works
- `cs:` for commentary sections
- `cit:` for citation sets
- `cr:` for commentary resolutions

These prefixes are only app-level conventions.

## 5. Commentary Work Schema

### 5.1 Required Fields

- `type`: must be `commentary_work`
- `commentary_id`: logical commentary work ID
- `app_id`
- `doc_id`: Mycel document ID carrying the commentary text
- `title`
- `commentary_kind`: one of `editorial`, `gloss`, `interpretation`, `study-note`, `practical-guidance`, `comparative-note`
- `source_documents`: non-empty array of cited source document IDs
- `authored_by`: maintainer key or app-level actor ID
- `created_at`
- `updated_at`

### 5.2 Optional Fields

- `language`
- `summary`
- `audience_label`
- `supersedes_commentary`
- `default_citation_policy`
- `active_resolution_id`

### 5.3 Example

```json
{
  "type": "commentary_work",
  "commentary_id": "cw:main-commentary-a",
  "app_id": "app:commentary-reference",
  "doc_id": "doc:commentary-main-a",
  "title": "Maintainer Notes on Source A",
  "commentary_kind": "editorial",
  "source_documents": ["doc:source-a"],
  "authored_by": "pk:maintainer-014",
  "language": "en",
  "summary": "A section-by-section commentary on Source A.",
  "default_citation_policy": "exact-or-locator-only",
  "created_at": 1772941800,
  "updated_at": 1772942400
}
```

## 6. Commentary Section Schema

### 6.1 Required Fields

- `type`: must be `commentary_section`
- `section_id`
- `app_id`
- `commentary_id`
- `section_kind`: one of `overview`, `line-note`, `anchor-note`, `cross-reference`, `application-note`, `dispute-note`
- `body`
- `order_key`
- `created_at`
- `updated_at`

### 6.2 Optional Fields

- `title`
- `anchor_refs`: array of source anchors or block references
- `citation_ids`: array of citation-set IDs
- `supersedes_section`
- `visibility_label`

### 6.3 Validation Rules

- `commentary_id` must resolve to an existing `commentary_work`.
- `citation_ids`, if present, must resolve to citation sets belonging to the same `commentary_id`.
- `anchor_refs` should not point to commentary sections when the section is describing source text.

### 6.4 Example

```json
{
  "type": "commentary_section",
  "section_id": "cs:note-001",
  "app_id": "app:commentary-reference",
  "commentary_id": "cw:main-commentary-a",
  "section_kind": "line-note",
  "title": "Why this phrase matters",
  "body": "This phrase narrows the scope of the surrounding obligation and should be read together with the next paragraph.",
  "order_key": "0001",
  "anchor_refs": ["block:source-a-14"],
  "citation_ids": ["cit:note-001"],
  "created_at": 1772942100,
  "updated_at": 1772942280
}
```

## 7. Citation Set Schema

### 7.1 Required Fields

- `type`: must be `citation_set`
- `citation_id`
- `app_id`
- `commentary_id`
- `section_id`
- `references`: non-empty array
- `created_at`
- `updated_at`

### 7.2 Reference Item Shape

Each item in `references` should contain:

- `source_id`
- `locator`
- `relation_kind`: one of `supports`, `interprets`, `contrasts`, `applies`, `questions`

Optional per-reference fields:

- `quote`
- `note`
- `source_revision_id`
- `source_head`
- `source_profile_id`
- `anchor_hash`

### 7.3 Optional Top-Level Fields

- `quote_policy`
- `notes`
- `source_bundle_id`

### 7.4 Validation Rules

- `source_id` should resolve to a cited source document or source bundle.
- `locator` should point to a stable logical target such as a block ID, anchor ID, witness segment, or app-defined source locator.
- A strict profile may require at least one version-locking field such as `source_revision_id`, `source_head`, or `anchor_hash`.
- `quote`, if present, should be auditable against the cited source under the active profile.

### 7.5 Example

```json
{
  "type": "citation_set",
  "citation_id": "cit:note-001",
  "app_id": "app:commentary-reference",
  "commentary_id": "cw:main-commentary-a",
  "section_id": "cs:note-001",
  "references": [
    {
      "source_id": "doc:source-a",
      "locator": "block:source-a-14",
      "relation_kind": "interprets",
      "quote": "review process",
      "source_revision_id": "rev:source-a-r14"
    },
    {
      "source_id": "doc:source-a",
      "locator": "block:source-a-15",
      "relation_kind": "supports",
      "note": "Read together with the immediately following block.",
      "source_head": "head:source-a-main"
    }
  ],
  "quote_policy": "exact-or-locator-only",
  "created_at": 1772942100,
  "updated_at": 1772942280
}
```

## 8. Commentary Resolution Schema

### 8.1 Required Fields

- `type`: must be `commentary_resolution`
- `resolution_id`
- `app_id`
- `commentary_id`
- `candidate_sections`: non-empty array of section IDs
- `accepted_sections`: array of section IDs
- `accepted_under_profile`
- `updated_at`

### 8.2 Optional Fields

- `alternative_sections`
- `decision_trace_ref`
- `rationale_summary`
- `state_label`
- `created_at`

### 8.3 Purpose

This record is optional.

It exists for deployments that need governed acceptance of commentary layers, such as:

- one accepted maintainer commentary under a profile
- one accepted commentary plus visible alternatives
- commentary sections marked as advisory only

Deployments that do not govern commentary acceptance can omit this record family.

## 9. Minimum Admissibility Rule

A commentary section should not be treated as reviewable commentary unless it provides:

- readable body text
- at least one explicit anchor reference or citation set for source-facing claims
- enough citation context to be auditable

This does not guarantee acceptance.

It only guarantees that the commentary can be reviewed.

## 10. Version-Locking Options

Different deployments may want different citation strength.

Recommended tiers:

- `locator-only`: cite by `source_id` and `locator`
- `revision-locked`: require `source_revision_id`
- `accepted-head-locked`: require `source_head` under a named profile
- `anchor-hash-locked`: require a stable anchor hash or witness-segment hash

This should remain a profile choice rather than a mandatory core protocol rule.

## 11. Client Behavior

A conforming reader client should:

- present source text and commentary as separate layers
- allow readers to inspect the cited source target for each note
- distinguish commentary acceptance from source acceptance
- surface unresolved or alternative commentary when the active profile defines commentary governance

The client should not:

- silently rewrite source text with commentary wording
- present commentary as if it were the source witness
- discard citation context when showing commentary snippets

## 12. Example Flow

Recommended flow:

1. a maintainer publishes a new commentary document as its own `doc_id`
2. the document carries one `commentary_work` record and multiple `commentary_section` records
3. each section attaches one or more `citation_set` records to cited source blocks or anchors
4. a client renders the source text and lets the reader inspect commentary side by side
5. if the deployment governs commentary acceptance, it publishes a `commentary_resolution`

## 13. Relation to Other Notes

This note is a companion to:

- `DESIGN-NOTES.mycel-app-layer`
- `DESIGN-NOTES.qa-minimal-schema`
- `DESIGN-NOTES.interpretation-dispute-model`
- `DESIGN-NOTES.canonical-text-profile`

It stays out of the protocol core and defines only a narrow app-layer shape for commentary-heavy documents.
