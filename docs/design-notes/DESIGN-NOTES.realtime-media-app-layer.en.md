# Realtime Media App Layer

Status: design draft

This note describes how Mycel can support realtime audio/video services without turning the core protocol or wire layer into a media-streaming transport.

Plainly put, Mycel can carry the control plane, record plane, and audit plane for a media service, while specialized media protocols carry the actual live audio/video stream.

## 0. Goal

Enable Mycel to support live and recorded media workflows while keeping:

- media-packet transport outside the core protocol
- accepted state, moderation, access policy, and record history inside Mycel
- replay deterministic and side-effect free

Keep in Mycel:

- stream definitions
- room or channel state
- access and role policy
- moderation actions
- recording metadata
- subtitle or caption history
- chapter markers
- derived playback or publication state
- audit and dispute history

Keep outside Mycel core:

- low-latency media packet delivery
- codec negotiation
- adaptive bitrate control
- TURN/ICE/WebRTC transport mechanics
- live audio/video mixing
- CDN-side segment distribution

## 1. Design Rules

The media app should follow six rules.

1. Revision replay MUST remain side-effect free.
2. Raw live media streams MUST NOT be replicated through normal Mycel state.
3. Access policy and accepted publication state MAY be carried by Mycel.
4. Live transport SHOULD use specialized media protocols.
5. Recording, subtitle, moderation, and publication history SHOULD remain auditable in Mycel.
6. A node MUST be able to participate without storing every media asset globally.

## 2. Layer Split

### 2.1 Client Layer

The client is the user-facing layer.

Responsibilities:

- show live-session metadata
- show accepted channel or room state
- display access conditions and participant roles
- display moderation state
- display recording, caption, and publication history
- let users create approved app-layer intents such as publish, annotate, caption, or moderate

Non-responsibilities:

- do not carry raw live media packets through normal Mycel objects
- do not redefine transport or codec behavior
- do not bypass access policy or moderation rules

### 2.2 Media Runtime Layer

The media runtime executes live-service behavior outside Mycel core.

Responsibilities:

- operate WebRTC, RTMP, HLS, SRT, or similar media transport
- manage media ingest, forwarding, or segment generation
- read accepted Mycel state for channel, access, and moderation policy
- publish receipts or summaries back into Mycel

Non-responsibilities:

- do not redefine protocol verification
- do not treat non-accepted branch state as live policy
- do not make media transport truth outrank signed Mycel records

### 2.3 Record and Effect Layer

The effect layer is the explicit representation of external media-side actions and results.

Examples:

- start livestream session
- rotate stream key
- create recording
- publish subtitle batch
- revoke viewer role
- mark recording as published
- record moderation enforcement result

## 3. Core Object Families

### 3.1 Channel or Room Document

Represents one long-lived media space.

Suggested fields:

- `channel_id`
- `display_name`
- `role_policy`
- `access_policy`
- `recording_policy`
- `moderation_policy`
- `publication_policy`
- `active_runtime_refs`

Purpose:

- define one room, stream, or channel
- declare who may publish, moderate, or view
- define which runtimes are allowed to operate it

### 3.2 Session Document

Represents one live session or broadcast window.

Suggested fields:

- `session_id`
- `channel_id`
- `started_at`
- `ended_at`
- `runtime_ref`
- `status`
- `ingest_summary`
- `session_digest`

Typical `status` values:

- `scheduled`
- `live`
- `ended`
- `failed`

### 3.3 Access and Role Document

Represents accepted viewer, publisher, moderator, or relay-side permissions.

Suggested fields:

- `subject_ref`
- `channel_id`
- `role`
- `grant_state`
- `granted_by`
- `granted_at`
- `revoked_at`

Typical `role` values:

- `viewer`
- `publisher`
- `moderator`
- `captioner`

### 3.4 Moderation Document

Represents explicit moderation state and actions.

Suggested fields:

- `action_id`
- `channel_id`
- `target_ref`
- `action_kind`
- `reason`
- `issued_by`
- `issued_at`
- `status`

Examples:

- mute publisher
- remove viewer
- pause chat-linked annotation
- delist recording

### 3.5 Recording Document

Represents one recording or archive object.

Suggested fields:

- `recording_id`
- `session_id`
- `storage_ref`
- `media_digest`
- `duration_ms`
- `published_state`
- `visibility`
- `created_at`

Purpose:

- identify the recording
- bind it to a session
- declare published or hidden state
- support audit without storing raw media in normal Mycel state

### 3.6 Subtitle / Caption Document

Represents subtitle or caption history.

Suggested fields:

- `caption_batch_id`
- `session_id`
- `language`
- `segment_refs`
- `editor_ref`
- `created_at`
- `revision_digest`

This is a strong fit for Mycel because subtitle and caption history benefit from verifiable revision trails.

### 3.7 Publication and Playback View

Represents the accepted default playback state.

Examples:

- which recording is the default published version
- which caption track is the default accepted track
- which moderation state is currently in force
- which chapter markers are accepted by default

This is where Mycel's accepted-state model is useful: the default media view does not need to be global consensus, only the result derived under fixed rules.

## 4. Recommended Execution Flow

1. A client or service creates a session intent or session update.
2. The media runtime operates the live transport outside Mycel.
3. The runtime publishes session summaries, recording metadata, and effect receipts into Mycel.
4. Moderation, captions, and publication decisions accumulate as signed history.
5. A fixed profile or app policy derives the accepted default playback state.
6. Clients render the accepted playback state while still allowing audit of alternate valid branches.

## 5. Why Mycel Fits This Layer

Mycel is useful for media services because it can preserve:

- verifiable revision history for captions, moderation, and publication state
- accepted default playback or reading state under fixed rules
- decentralized replication of metadata and governance state
- audit trails for why one published view became the default

It is not trying to replace:

- WebRTC
- RTMP
- HLS
- media CDN delivery
- codec or jitter control

## 6. Recommended Mycel Position

For now:

- keep realtime media as an app-layer design-note concept
- keep live audio/video transport outside Mycel core and wire protocol
- let Mycel carry the accepted state, policy, history, and audit surface
- revisit formal runtime/profile patterns later if a media-oriented `M5` app-layer expansion becomes active
