# Forum App Layer

Status: design draft

This note describes how Mycel could carry a forum-style application layer while keeping forum semantics outside the protocol core.

The main design principle is:

- Mycel carries forum state, governance state, moderation history, and audit traces
- a client renders boards, threads, and replies from verified objects
- multiple candidate moderation outcomes may coexist
- one accepted forum reading is derived under the active profile
- the core protocol remains neutral and purely technical

See also:

- [`DESIGN-NOTES.forum-qa-relationship.en.md`](./DESIGN-NOTES.forum-qa-relationship.en.md) for the boundary between the Forum and Q&A app-layer examples
- [`DESIGN-NOTES.text-worker-editor.en.md`](./DESIGN-NOTES.text-worker-editor.en.md) for the authoring surface that would consume document-anchored discussion
- [`DESIGN-NOTES.commentary-citation-schema.en.md`](./DESIGN-NOTES.commentary-citation-schema.en.md) for commentary records that discussion outcomes may promote into

## 0. Goal

Enable Mycel to carry a durable forum system without turning the core protocol into a forum-specific primitive set.

Keep in Mycel:

- forum app definition
- board state
- thread state
- post state
- document-anchored discussion threads
- promotion records that lift discussion outcomes into commentary or annotation candidates
- moderation actions
- accepted-thread and accepted-board reading state
- optional notification or indexing effect history

Keep outside Mycel core:

- ranking algorithms as protocol rules
- anti-spam heuristics as protocol rules
- private trust scores
- delivery infrastructure
- search-engine internals
- secrets and runtime credentials

## 1. Design Rules

The forum app layer should follow eight rules.

1. Boards, threads, and posts are app-layer objects, not protocol primitives.
2. Individual posts should be independently addressable and independently replicable.
3. A conforming reader client should show one accepted reading of a thread or board under the active profile.
4. Alternative moderation outcomes should remain auditable and visible as alternatives when policy allows.
5. Moderation history should be explicit objects, not silent state mutation.
6. Large forum views should be derived from verified local indexes rather than one monolithic thread object.
7. Threads may target a document, commentary section, or annotation anchor rather than only a general topic.
8. Discussion outcomes that should affect document or annotation state must be promoted through explicit candidate records rather than by treating forum posts as canonical edits.

## 2. Recommended Shape

The recommended shape is:

- `board`, `thread`, and `post` as separate object families
- thread roots that may point to a document, commentary work, or annotation anchor
- one promotion family that lifts discussion or debate outcomes into candidate commentary or annotation records
- one or more resolution documents that derive accepted forum reading
- moderation as explicit signed actions
- optional derived local indexes for fast list and thread rendering

This is closer to a post-centric forum than a document-centric forum.

Why this shape is preferred:

- a single reply can replicate independently
- moderation can target one post without rewriting the whole thread
- thread rendering can scale through derived indexes
- branch divergence remains easier to inspect

## 3. Three Layers

### 3.1 Client Layer

The client is the reader and authoring surface.

Responsibilities:

- browse boards
- open threads
- render replies in chronological or policy-derived order
- show accepted reading and moderation status
- create post intents and moderation intents
- inspect history and alternatives

Non-responsibilities:

- do not redefine accepted-reading rules
- do not hide audit history that policy requires to remain visible
- do not silently mutate forum state outside Mycel objects

### 3.2 Runtime Layer

The runtime is optional and assistive.

Responsibilities:

- maintain derived board and thread indexes
- generate notification effects
- support bounded search or feed materialization
- publish effect receipts for optional external actions

Non-responsibilities:

- do not decide accepted moderation state by itself
- do not override profile-governed visibility
- do not treat ranking heuristics as protocol truth

### 3.3 Effect Layer

The effect layer is optional.

Examples:

- subscriber notification delivery
- digest generation
- bounded search-index refresh
- bridge delivery to an approved external surface

Effect objects should remain explicit, auditable, and replay-safe.

## 4. Core Forum Objects

### 4.1 Forum App Manifest

Defines the forum application itself.

Suggested fields:

- `app_id`
- `app_version`
- `board_documents`
- `thread_documents`
- `post_documents`
- `resolution_documents`
- `moderation_documents`
- `allowed_effect_types`
- `runtime_profile`

Purpose:

- identify the forum app
- declare participating document families
- declare allowed effect classes

### 4.2 Board Document

Represents one board or category.

Suggested fields:

- `board_id`
- `app_id`
- `slug`
- `title`
- `description`
- `posting_policy`
- `moderation_policy_ref`
- `created_at`
- `updated_at`

Purpose:

- define one forum surface
- declare local policy context
- provide a stable target for threads

### 4.3 Thread Document

Represents one thread root.

Suggested fields:

- `thread_id`
- `board_id`
- `opened_by`
- `title`
- `opening_post`
- `subject_kind`
- `subject_document_id`
- `subject_commentary_id`
- `subject_annotation_ref`
- `subject_anchor_refs`
- `status`
- `tags`
- `created_at`
- `updated_at`

Suggested `status` values:

- `open`
- `locked`
- `resolved`
- `archived`
- `hidden`

Suggested `subject_kind` values:

- `general-thread`
- `document-thread`
- `commentary-thread`
- `annotation-thread`

Purpose:

- support ordinary forum threads
- support threads attached to a source document or commentary work
- support discussions anchored to one or more document or annotation targets

### 4.4 Post Document

Represents one independently replicated post.

Suggested fields:

- `post_id`
- `thread_id`
- `board_id`
- `reply_to`
- `posted_by`
- `body`
- `edit_policy`
- `created_at`
- `supersedes_post`

Purpose:

- carry one atomic forum contribution
- support reply trees
- preserve edit history through supersession rather than silent overwrite

### 4.5 Discussion Promotion Document

Represents an explicit attempt to promote discussion output into a text-facing candidate record.

Suggested fields:

- `promotion_id`
- `thread_id`
- `source_post_ids`
- `target_kind`
- `target_ref`
- `proposed_record_kind`
- `proposed_payload_ref`
- `issued_by`
- `issued_at`
- `status`
- `supersedes_promotion`

Suggested `target_kind` values:

- `document`
- `commentary_section`
- `annotation`

Suggested `proposed_record_kind` values:

- `commentary_section`
- `citation_set`
- `annotation_note`
- `document_revision_candidate`

Suggested `status` values:

- `draft`
- `submitted`
- `accepted`
- `rejected`
- `withdrawn`

Purpose:

- keep discussion posts distinct from text-facing records
- let a discussion or debate result become a candidate commentary or annotation object
- preserve traceability from final text-facing output back to the originating thread

### 4.6 Moderation Action Document

Represents a signed moderation action.

Suggested fields:

- `moderation_action_id`
- `target_kind`
- `target_id`
- `action_kind`
- `issued_by`
- `reason_code`
- `reason_summary`
- `issued_at`
- `supersedes_action`

Suggested `action_kind` values:

- `hide-post`
- `unhide-post`
- `lock-thread`
- `unlock-thread`
- `pin-thread`
- `unpin-thread`
- `move-thread`
- `label-thread`
- `archive-thread`

Moderation should remain explicit and inspectable.

### 4.7 Thread Resolution Document

Represents accepted reading state for one thread.

Suggested fields:

- `thread_resolution_id`
- `thread_id`
- `accepted_posts`
- `hidden_posts`
- `pinned_reply_order`
- `accepted_under_profile`
- `decision_trace_ref`
- `updated_at`

Purpose:

- define what the default reader should see
- preserve visibility and ordering decisions
- point to the profile under which that result is derived

### 4.8 Board Resolution Document

Represents accepted board-level state.

Suggested fields:

- `board_resolution_id`
- `board_id`
- `visible_threads`
- `pinned_threads`
- `hidden_threads`
- `accepted_under_profile`
- `updated_at`

Purpose:

- define default board listing state
- support pinned and hidden thread behavior
- keep board rendering profile-governed rather than ad hoc

## 5. Example Thread Resolution

```json
{
  "type": "forum_thread_resolution",
  "thread_resolution_id": "tres:8c0b7d10",
  "app_id": "app:forum-main",
  "thread_id": "thr:92ab771e",
  "accepted_posts": [
    "post:001",
    "post:002",
    "post:004"
  ],
  "hidden_posts": [
    "post:003"
  ],
  "pinned_reply_order": [
    "post:001",
    "post:002",
    "post:004"
  ],
  "accepted_under_profile": "policy:forum-main-v1",
  "decision_trace_ref": "trace:3f91aa72",
  "updated_at": 1772942400
}
```

This shows a common Mycel-style forum pattern:

- multiple posts exist in thread history
- not all posts are default-visible
- visibility is explicit
- default order is explicit
- one profile determines accepted reading

## 6. Document-anchored Discussion Flow

The forum app should support a text-worker discussion flow in which discussion remains discussion, but useful outcomes can be promoted into text-facing candidate records.

Recommended flow:

1. open a thread attached to a document, commentary work, or annotation anchor
2. let participants reply, cite source passages, and debate interpretations
3. keep those replies as normal forum posts rather than silently rewriting the target text
4. when useful material emerges, issue a `discussion_promotion` record
5. link that promotion to a candidate commentary section, citation set, annotation note, or document revision candidate
6. let the relevant text-oriented profile decide whether the promoted record becomes part of an accepted reading

This preserves a clean boundary:

- forum discussion remains conversational and auditable
- promoted text-facing records remain explicit and reviewable
- accepted document or annotation state still comes from profile-governed derivation rather than thread popularity

## 7. Accepted Reading Model

The forum app should follow the same accepted-head principles as the rest of Mycel.

Recommended reader behavior:

1. load the accepted board resolution for one board
2. load the accepted thread resolution for one thread
3. fetch the referenced posts
4. verify all objects locally
5. render one accepted forum reading
6. expose alternatives and history on demand

This means:

- "what I see by default" is profile-governed
- "what else exists" remains auditable
- moderation is not hidden behind local discretionary state

## 8. Moderation Model

Moderation should be modeled as signed app-layer state, not a hidden database flag.

Recommended moderation split:

- maintainers or moderators issue `moderation_action` objects
- resolution documents incorporate those actions into accepted reading
- clients show both the resulting state and enough trace context to explain it

The client should be able to answer:

- why is this post hidden
- why is this thread locked
- which profile or maintainer set made that outcome active

## 9. Forks and Disputes

A forum app on Mycel should not pretend that moderation disputes never branch.

Possible outcomes:

- two moderator sets publish different thread resolutions
- one reader profile accepts one branch
- another profile accepts another branch
- both remain inspectable

This is one of the strongest reasons to keep forum semantics in Mycel:

- disputes remain visible
- history remains replayable
- one side does not need to erase the other to publish its own accepted reading

## 10. Scaling and Local Indexes

A practical forum client should use rebuildable local indexes.

Useful indexes include:

- board-to-thread index
- thread-to-post index
- subject-anchor-to-thread index
- promotion-by-target index
- reply-tree index
- moderation-action-by-target index
- accepted-resolution index

These indexes should be:

- derived from verified objects
- rebuildable from canonical data alone
- treated as local acceleration structures, not portable truth

## 11. Non-goals

This note does not propose:

- protocol-level forum primitives
- protocol-level ranking rules
- global upvote consensus
- spam prevention solved in protocol core
- private-message secrecy model
- large-scale public search architecture

These are either app-policy problems, runtime problems, or future deployment problems.

## 12. Why This Fits Mycel

A forum app matches Mycel unusually well because forums need:

- durable text history
- explicit moderation history
- document-anchored discussion around text and annotation
- a traceable bridge from discussion to candidate commentary or annotation records
- branch tolerance during disputes
- default reading derived by governance
- object-level replication

A forum app is therefore a natural app-layer example for Mycel, even though it should remain outside the protocol core.

## 13. Recommended Next Step

If this direction is pursued, the next concrete step should be one of:

1. a minimal forum schema note with example JSON envelopes
2. fixture-backed sample objects for one document-anchored thread, one promotion record, and one moderation split
3. a reader-surface note for thread rendering, anchor inspection, and promotion trace inspection
