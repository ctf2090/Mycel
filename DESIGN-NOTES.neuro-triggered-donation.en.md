# Neuro-triggered Donation App Layer

Status: design draft

This note describes an app-layer design where a runtime observes machine-derived user-state events and may create donation-related records under a pre-authorized consent policy.

The main design principle is:

- Mycel carries consent state, session state, derived user-state events, donation intents or pledges, settlement receipts, and audit history
- a client lets users inspect and configure the flow
- a sensing and payment runtime performs external observation and payment side effects
- the core protocol remains neutral and purely technical

## 0. Goal

Enable Mycel to carry a traceable neuro-triggered donation workflow without turning the core protocol into a sensor processor or payment engine.

Keep in Mycel:

- consent policy state
- session summaries
- derived user-state events
- donation intents or pledges
- settlement receipts
- audit and dispute history

Keep outside Mycel core:

- raw brain-signal capture
- low-level sensor interpretation
- payment execution
- secret handling
- irreversible settlement side effects

## 1. Design Rules

The app should follow six rules.

1. Revision replay MUST remain side-effect free.
2. Raw sensor streams MUST NOT be replicated through normal Mycel state.
3. A derived user-state event MUST NOT by itself equal payment consent.
4. Auto-triggered donation behavior MUST require an explicit consent profile.
5. Payment-side effects MUST happen outside the core protocol.
6. A donor MUST be able to revoke, pause, or dispute the flow.

## 2. Four Layers

### 2.1 Client Layer

The client is the user-facing layer.

Responsibilities:

- display consent profile state
- display session history and derived events
- let the user enable, pause, or revoke the feature
- show donation pledges, intents, receipts, and disputes
- show why a donation was or was not triggered

Non-responsibilities:

- do not interpret raw signals directly by default
- do not execute payment side effects
- do not bypass consent policy

### 2.2 Sensor Runtime Layer

The sensor runtime observes device output and derives high-level state events.

Responsibilities:

- connect to an approved sensor interface
- summarize one sensing session
- derive explicit high-level events such as `stable-focus` or `stable-rest`
- sign or publish session evidence summaries if required by deployment policy

Non-responsibilities:

- do not publish raw signal streams into replicated state
- do not directly settle payments

### 2.3 Payment Runtime Layer

The payment runtime executes payment-side effects.

Responsibilities:

- read accepted consent and trigger state
- decide whether a donation pledge or intent may be created
- execute or prepare external payment steps where allowed
- publish settlement receipts or failure receipts

Non-responsibilities:

- do not redefine consent rules
- do not use unaccepted branch state as payment input
- do not treat sensor events as unlimited authorization

### 2.4 Effect Layer

The effect layer explicitly represents external observation and payment actions.

Examples:

- create sensor session
- derive high-level user-state summary
- create payment session
- check settlement result
- send donor notification

## 3. Core Objects

### 3.1 Consent Profile

Defines what the user has pre-authorized.

Suggested fields:

- `consent_id`
- `user_ref`
- `trigger_mode`
- `allowed_amount`
- `currency`
- `cooldown_seconds`
- `max_triggers_per_day`
- `runtime_policy_ref`
- `status`
- `created_at`
- `updated_at`

Typical `status` values:

- `active`
- `paused`
- `revoked`
- `expired`

### 3.2 Session Record

Represents one sensing session summary.

Suggested fields:

- `session_id`
- `user_ref`
- `device_ref`
- `runtime_ref`
- `started_at`
- `ended_at`
- `summary_hash`
- `status`

Typical `status` values:

- `complete`
- `failed`
- `discarded`

### 3.3 User-State Event

Represents one derived high-level event created from a completed session.

Suggested fields:

- `event_id`
- `session_id`
- `user_ref`
- `state_label`
- `stability_score`
- `duration_ms`
- `trigger_eligible`
- `created_at`

Typical `state_label` values:

- `stable-focus`
- `stable-rest`
- `transition-state`

### 3.4 Donation Pledge or Intent

Represents what the system is allowed to do after a qualifying event.

Suggested fields:

- `intent_id`
- `user_ref`
- `consent_id`
- `trigger_event_id`
- `intent_kind`
- `amount`
- `currency`
- `payment_method`
- `status`
- `created_at`
- `updated_at`

Recommended `intent_kind` values:

- `manual-confirmation`
- `pledge`
- `pre-authorized-payment`

### 3.5 Donation Receipt

Represents settlement or payment confirmation.

Suggested fields:

- `receipt_id`
- `intent_id`
- `executor`
- `payment_ref`
- `amount_received`
- `currency`
- `status`
- `settled_at`
- `processor_summary`
- `error_summary`

### 3.6 Dispute or Revocation Record

Represents a user challenge, pause, or rollback request.

Suggested fields:

- `record_id`
- `user_ref`
- `related_intent_id`
- `related_receipt_id`
- `action_kind`
- `reason`
- `created_at`

Typical `action_kind` values:

- `pause`
- `revoke`
- `dispute`
- `refund-request`

## 4. Recommended Trigger Policy

For a first client, I recommend a conservative trigger policy:

1. a user must first create an explicit consent profile
2. the consent profile must cap amount and frequency
3. the sensor runtime must derive a high-level event from a bounded session
4. the event must meet a minimum duration threshold
5. the cooldown window must have elapsed
6. the system should create a `pledge` or `manual-confirmation` intent before any direct payment

This keeps derived user-state and payment authorization clearly separated.

## 5. Accepted-State Driven Execution

The runtimes should execute external actions only from accepted state.

Recommended rule:

1. read accepted consent and session state
2. identify newly accepted derived events
3. evaluate them under the active consent profile
4. create a pledge or payment intent if allowed
5. execute external payment steps only where policy permits
6. publish receipts and any dispute records

## 6. Privacy and Data Minimization

This app must strongly minimize sensitive data.

Recommended rules:

- store session summaries, not raw signals
- store derived state labels and evidence hashes, not full waveform data
- separate user identity from device identity where possible
- keep payment references separate from user-facing records
- allow deployments to use pseudonymous user references

## 7. Safety Guardrails

I recommend the following hard guardrails:

- no auto-trigger without prior consent
- no unlimited amount or unlimited frequency
- no raw-signal replication
- no silent runtime-side rule changes
- no direct trigger from unverified or failed sessions
- no hidden fallback from `manual-confirmation` to direct payment

## 8. Minimal v0.1 Profile

For a first implementation, I recommend a narrow profile.

- one consent profile per user
- one approved sensor runtime family
- one derived state label used for triggering
- only `pledge` or `manual-confirmation`
- no direct automatic settlement
- explicit user pause and revoke controls

Tradeoff:

- lower automation
- much lower safety risk
- easier auditability

## 9. Open Questions

- Should first-client deployments allow `pre-authorized-payment` at all, or only `pledge`?
- How should runtimes prove that a session summary was derived from approved hardware?
- Should dispute records be local-only, or replicated as normal app records?
