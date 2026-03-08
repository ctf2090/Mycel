# Q&A App Layer

Status: design draft

This note describes a Q&A application layer carried by Mycel while keeping doctrine, interpretation, and pastoral workflows outside the core protocol.

The main design principle is:

- Mycel carries question state, answer state, citations, governance signals, and audit history
- a client lets users ask, browse, and inspect accepted answers
- an optional runtime assists with retrieval, notification, or draft preparation
- the core protocol remains neutral and purely technical

## 0. Goal

Enable Mycel to carry a durable Q&A system without turning the core protocol into a doctrine engine.

Keep in Mycel:

- Q&A app definition
- question documents
- answer documents
- citation links
- accepted-answer governance state
- optional retrieval or notification effect history

Keep outside Mycel core:

- doctrinal truth claims
- religious endorsement
- private counseling judgments
- external search or HTTP execution
- secrets and runtime credentials

## 1. Design Rules

The Q&A App Layer should follow five rules.

1. Questions and answers are app-level state, not protocol primitives.
2. Multiple candidate answers may coexist.
3. A conforming reader client should display one active accepted answer under the fixed active profile.
4. Alternative answers should remain auditable and visible as non-active branches or alternatives.
5. Runtime-assisted output must remain candidate content until accepted through normal governance.

## 2. Three Layers

### 2.1 Client Layer

The client is the user-facing layer.

Responsibilities:

- create question intents
- display accepted question and answer state
- show citations, answer history, and alternative answers
- show whether an answer is human-authored, runtime-assisted, or mixed
- show governance and audit traces

Non-responsibilities:

- do not redefine accepted-answer rules
- do not directly decide doctrinal acceptance
- do not hide alternative answers when the protocol requires audit visibility

### 2.2 Runtime Layer

The runtime is optional and assistive.

Responsibilities:

- read accepted Q&A state
- perform approved retrieval or indexing effects
- publish effect receipts
- optionally prepare draft answers or citation suggestions

Non-responsibilities:

- do not publish accepted answers by itself
- do not bypass view-maintainer governance
- do not treat draft output as accepted doctrine

### 2.3 Effect Layer

The effect layer describes optional side effects used by the Q&A app.

Examples:

- corpus retrieval
- notification delivery
- HTTP fetch against an approved source
- background indexing

Effect objects should remain explicit, auditable, and replay-safe.

## 3. Core Q&A Objects

### 3.1 Q&A App Manifest

Defines the Q&A application itself.

Suggested fields:

- `app_id`
- `app_version`
- `question_documents`
- `answer_documents`
- `resolution_documents`
- `allowed_effect_types`
- `citation_policy`
- `runtime_profile`

Purpose:

- identify the Q&A app
- declare participating document families
- declare citation and runtime expectations

### 3.2 Question Document

Represents one question thread.

Suggested fields:

- `question_id`
- `app_id`
- `asked_by`
- `title`
- `body`
- `topic_tags`
- `target_corpus`
- `status`
- `created_at`
- `updated_at`

Typical `status` values:

- `open`
- `under-review`
- `answered`
- `accepted`
- `superseded`
- `closed`

### 3.3 Answer Document

Represents one candidate answer to a question.

Suggested fields:

- `answer_id`
- `question_id`
- `answered_by`
- `answer_kind`
- `body`
- `citations`
- `source_mode`
- `created_at`
- `supersedes_answer`

Suggested `answer_kind` values:

- `direct-answer`
- `commentary`
- `clarification`
- `objection`
- `pastoral-guidance`

Suggested `source_mode` values:

- `human-authored`
- `runtime-assisted`
- `mixed`

### 3.4 Resolution Document

Represents accepted-answer state for one question.

Suggested fields:

- `resolution_id`
- `question_id`
- `candidate_answers`
- `accepted_answer`
- `alternative_answers`
- `accepted_under_profile`
- `rationale_summary`
- `updated_at`

Purpose:

- identify the currently accepted answer
- preserve alternative answers
- show the governing profile used to derive that result

### 3.5 Citation Set

Represents the textual basis for an answer.

Suggested fields:

- `citation_id`
- `question_id`
- `answer_id`
- `references`
- `quote_policy`
- `notes`

Purpose:

- connect answers to source texts or prior commentary
- support auditability
- reduce purely discretionary answer acceptance

### 3.6 Optional Effect Request and Effect Receipt

These follow the App Layer pattern already described elsewhere.

Typical uses in a Q&A app:

- request retrieval from an approved corpus index
- request notification to subscribers
- record runtime indexing completion

## 4. Governance Model

The Q&A app should use the same accepted-head principles as the rest of Mycel.

Recommended rules:

- question and answer documents may branch
- view-maintainers publish signed governance signals
- the active accepted answer is derived from the fixed active profile
- editor-maintainers may publish candidate answers but do not gain acceptance weight by default
- reader clients show the accepted answer as the default result
- reader clients also expose alternative answers and resolution history for auditability

This keeps client behavior constrained while preserving multi-answer history.

## 5. Suggested Flows

### 5.1 Human-Curated Flow

1. a user submits a question
2. editor-maintainers publish one or more candidate answers
3. citations are attached or verified
4. view-maintainers publish governance signals
5. the active profile derives one accepted answer
6. the client displays that answer as default and keeps alternatives inspectable

### 5.2 Runtime-Assisted Flow

1. a user submits a question
2. the runtime retrieves relevant materials or prepares a draft
3. the runtime publishes an effect receipt and optional draft answer
4. editor-maintainers revise or endorse the draft
5. view-maintainers govern accepted-answer state as usual

This keeps machine assistance subordinate to normal governance.

## 6. Guardrails

The Q&A app should adopt conservative guardrails.

- Answers that claim doctrinal authority should carry citations by default.
- Runtime-generated drafts should be clearly labeled.
- The accepted answer should never be treated as protocol-level universal truth.
- Reader clients should distinguish `accepted answer` from `only possible answer`.
- Private counseling or confidential guidance should not be stored in broadly replicated public documents unless the deployment explicitly intends that.
- Moderation or safety filtering may exist, but in a conforming reader client it should not silently rewrite the fixed-profile accepted answer.

## 7. Minimal v0.1 Q&A Profile

For a first implementation, I recommend a narrow profile.

- Human-authored questions and answers only
- Citations required for `direct-answer` and `commentary`
- No automatic runtime answer publication
- Runtime limited to retrieval, indexing, and notification
- One accepted answer plus visible alternatives
- Resolution state stored in a normal document family, not a new core protocol primitive

Tradeoff:

- lower automation
- higher governance clarity
- easier interoperability

## 8. Open Questions

- Should `resolution` be a dedicated document family or just a normal state document inside the app?
- Should citation policy vary by answer kind?
- Should runtime-assisted drafts require an explicit marker in accepted state?
- Should the app support multiple accepted answers under one profile, or only one primary answer plus alternatives?
