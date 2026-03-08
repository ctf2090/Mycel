# Q&A Minimal Schema

Status: design draft

This note defines a minimal app-level schema for a Mycel-carried Q&A application.

These schemas are not new core protocol primitives.
They are logical record shapes intended to live inside normal Mycel documents governed by the Q&A app.

## 0. Scope

This minimal schema covers four record families:

- `question`
- `answer`
- `resolution`
- `citation_set`

The goal is to make first-client implementation easier while keeping the schema narrow.

## 1. General Rules

1. All records are app-level JSON payloads and should follow Mycel canonical serialization rules when signed or hashed through normal document flows.
2. IDs shown here are logical app IDs, not new protocol object types.
3. Unknown fields should be rejected in a strict profile and ignored only if an app profile explicitly allows extensions.
4. Timestamps are integer Unix seconds.
5. Reader clients should treat `resolution.accepted_answer` as the default visible answer under the active profile.

## 2. Shared Field Conventions

### 2.1 Common Required Fields

Each record family should carry:

- `type`
- `app_id`
- `created_at`
- `updated_at`

### 2.2 Recommended ID Prefixes

Recommended logical ID prefixes:

- `q:` for questions
- `ans:` for answers
- `res:` for resolutions
- `cit:` for citation sets

These prefixes are only app-level conventions.

## 3. Question Schema

### 3.1 Required Fields

- `type`: must be `question`
- `question_id`: logical question ID
- `app_id`: Q&A app identifier
- `asked_by`: submitting actor key or app-level actor ID
- `title`: short question summary
- `body`: full question text
- `status`: one of `open`, `under-review`, `answered`, `accepted`, `superseded`, `closed`
- `created_at`
- `updated_at`

### 3.2 Optional Fields

- `topic_tags`: array of short topic strings
- `target_corpus`: array of corpus or document-set references
- `language`: BCP 47 language tag
- `answer_count`: cached count for UI convenience
- `current_resolution_id`: logical resolution ID if one exists

### 3.3 Example

```json
{
  "type": "question",
  "question_id": "q:7d9120aa",
  "app_id": "app:qa-reference",
  "asked_by": "pk:user-001",
  "title": "What does this term mean in this corpus?",
  "body": "I need a concise explanation of the term as used in document set A.",
  "status": "under-review",
  "topic_tags": ["terminology", "glossary"],
  "target_corpus": ["corpus:main-a"],
  "language": "en",
  "answer_count": 3,
  "current_resolution_id": "res:4f21a8c9",
  "created_at": 1772941800,
  "updated_at": 1772942400
}
```

## 4. Answer Schema

### 4.1 Required Fields

- `type`: must be `answer`
- `answer_id`: logical answer ID
- `app_id`
- `question_id`
- `answered_by`: author key or app-level actor ID
- `answer_kind`: one of `direct-answer`, `commentary`, `clarification`, `objection`, `applied-guidance`
- `body`
- `source_mode`: one of `human-authored`, `runtime-assisted`, `mixed`
- `created_at`
- `updated_at`

### 4.2 Optional Fields

- `citations`: array of citation-set IDs or inline references
- `supersedes_answer`: prior answer ID
- `summary`: short preview text
- `confidence_label`: app-defined non-numeric label such as `draft`, `reviewed`, `stable`
- `runtime_receipt_refs`: related effect receipt IDs when runtime-assisted

### 4.3 Example

```json
{
  "type": "answer",
  "answer_id": "ans:19bc44e2",
  "app_id": "app:qa-reference",
  "question_id": "q:7d9120aa",
  "answered_by": "pk:editor-014",
  "answer_kind": "direct-answer",
  "body": "In document set A, the term is used as a shorthand for the shared review process rather than for a single object.",
  "source_mode": "human-authored",
  "citations": ["cit:18f1d2ab"],
  "summary": "The term refers to the shared review process.",
  "confidence_label": "reviewed",
  "created_at": 1772942100,
  "updated_at": 1772942280
}
```

## 5. Resolution Schema

### 5.1 Required Fields

- `type`: must be `resolution`
- `resolution_id`
- `app_id`
- `question_id`
- `candidate_answers`: non-empty array of answer IDs
- `accepted_answer`: answer ID that must also appear in `candidate_answers`
- `alternative_answers`: array of answer IDs
- `accepted_under_profile`: active profile or policy identifier
- `updated_at`

### 5.2 Optional Fields

- `decision_trace_ref`
- `rationale_summary`
- `supersedes_resolution`
- `state_label`: app-defined label such as `draft`, `active`, `superseded`
- `created_at`

### 5.3 Validation Rules

- `accepted_answer` must be unique.
- `alternative_answers` must not contain `accepted_answer`.
- Every ID in `alternative_answers` must also appear in `candidate_answers`.
- `candidate_answers` should not contain duplicates.

### 5.4 Example

```json
{
  "type": "resolution",
  "resolution_id": "res:4f21a8c9",
  "app_id": "app:qa-reference",
  "question_id": "q:7d9120aa",
  "candidate_answers": [
    "ans:19bc44e2",
    "ans:73a0d5c1",
    "ans:9ef2210b"
  ],
  "accepted_answer": "ans:19bc44e2",
  "alternative_answers": [
    "ans:73a0d5c1",
    "ans:9ef2210b"
  ],
  "accepted_under_profile": "policy:community-main-v1",
  "decision_trace_ref": "trace:84c0f117",
  "rationale_summary": "Selected under the active profile because it has the strongest cited support and matching governance signals.",
  "state_label": "active",
  "created_at": 1772942100,
  "updated_at": 1772942400
}
```

## 6. Citation Set Schema

### 6.1 Required Fields

- `type`: must be `citation_set`
- `citation_id`
- `app_id`
- `question_id`
- `answer_id`
- `references`: non-empty array
- `created_at`
- `updated_at`

### 6.2 Reference Item Shape

Each item in `references` should contain:

- `source_id`
- `locator`

Optional per-reference fields:

- `quote`
- `note`

### 6.3 Optional Top-Level Fields

- `quote_policy`
- `notes`
- `source_bundle_id`

### 6.4 Example

```json
{
  "type": "citation_set",
  "citation_id": "cit:18f1d2ab",
  "app_id": "app:qa-reference",
  "question_id": "q:7d9120aa",
  "answer_id": "ans:19bc44e2",
  "references": [
    {
      "source_id": "doc:glossary-main",
      "locator": "block:term-review-process",
      "quote": "review process"
    },
    {
      "source_id": "doc:usage-notes",
      "locator": "block:usage-14",
      "note": "Explains the shorthand usage in document set A."
    }
  ],
  "quote_policy": "short-excerpts-only",
  "notes": "Both references use the term in the same procedural sense.",
  "created_at": 1772942160,
  "updated_at": 1772942160
}
```

## 7. Minimal Client Requirements

A minimal Q&A client should:

- list questions by `question_id`, `title`, and `status`
- show one default answer from `resolution.accepted_answer`
- expose alternative answers from `resolution.alternative_answers`
- show citation references for the displayed answer
- show trace links such as `resolution_id` and `decision_trace_ref`

## 8. Minimal Runtime Requirements

If runtime assistance is enabled outside `qa-strict-v0.1`, the runtime should:

- write draft answers as normal `answer` records
- label them through `source_mode`
- store related effect receipts separately
- never mark an answer as accepted by itself

## 9. Open Questions

- Should `question` and `resolution` always live in separate documents, or can they be co-located in one app state document?
- Should future non-strict profiles allow inline citations in addition to citation-set IDs?
- Should future non-strict profiles keep `confidence_label` free-form, or standardize a larger enum?

## 10. Strict Profile v0.1

For a first interoperable client, I recommend one narrow strict profile:

- `profile_name`: `qa-strict-v0.1`
- `app_id`: implementation-defined, but fixed per deployment
- objective: reduce parser and UI discretion for the first client

### 10.1 Global Strict Rules

The strict profile should enforce all of the following:

1. Unknown fields are rejected in all four record families.
2. All required fields must be present and must have the exact expected type.
3. Empty strings are rejected for `title`, `body`, and all logical IDs.
4. Arrays that are required to be non-empty must be rejected if empty.
5. Duplicate IDs inside `candidate_answers`, `alternative_answers`, or `references` are rejected.
6. `created_at` must be less than or equal to `updated_at`.
7. One stored record must contain one primary logical object only.

### 10.2 Strict Question Rules

- `language` is required.
- `topic_tags` may be present, but each item must be a non-empty string.
- `answer_count` is forbidden because it is cache-like and can drift.
- `current_resolution_id` is optional.

Allowed fields only:

- `type`
- `question_id`
- `app_id`
- `asked_by`
- `title`
- `body`
- `status`
- `topic_tags`
- `target_corpus`
- `language`
- `current_resolution_id`
- `created_at`
- `updated_at`

### 10.3 Strict Answer Rules

- `source_mode` must be `human-authored`.
- `citations` is required for `direct-answer`, `commentary`, and `applied-guidance`.
- `citations`, if present, must contain citation-set IDs only.
- Inline citation objects are forbidden.
- `runtime_receipt_refs` is forbidden.
- `confidence_label`, if present, must be one of `draft`, `reviewed`, `stable`.

Allowed fields only:

- `type`
- `answer_id`
- `app_id`
- `question_id`
- `answered_by`
- `answer_kind`
- `body`
- `source_mode`
- `citations`
- `supersedes_answer`
- `summary`
- `confidence_label`
- `created_at`
- `updated_at`

### 10.4 Strict Resolution Rules

- `decision_trace_ref` is required.
- `rationale_summary` is required.
- `state_label`, if present, must be one of `draft`, `active`, `superseded`.
- `created_at` is required.
- There must be exactly one `accepted_answer`.
- There must be at most one active resolution per `(question_id, accepted_under_profile)`.

Allowed fields only:

- `type`
- `resolution_id`
- `app_id`
- `question_id`
- `candidate_answers`
- `accepted_answer`
- `alternative_answers`
- `accepted_under_profile`
- `decision_trace_ref`
- `rationale_summary`
- `supersedes_resolution`
- `state_label`
- `created_at`
- `updated_at`

### 10.5 Strict Citation-Set Rules

- `references` must contain only objects with `source_id`, `locator`, and optional `quote` / `note`.
- `quote_policy` is required.
- `source_bundle_id` is forbidden.
- Each citation set must reference exactly one `answer_id`.

Allowed fields only:

- `type`
- `citation_id`
- `app_id`
- `question_id`
- `answer_id`
- `references`
- `quote_policy`
- `notes`
- `created_at`
- `updated_at`

### 10.6 Strict Runtime Boundary

In `qa-strict-v0.1`:

- runtime may retrieve, index, or notify
- runtime must not publish accepted answers
- runtime must not publish `answer` records
- any machine assistance must remain outside the strict accepted-answer path

### 10.7 First-Client Benefit

This strict profile intentionally trades flexibility for predictable implementation:

- fewer parsing branches
- less UI ambiguity
- less governance drift
- easier interoperability testing
