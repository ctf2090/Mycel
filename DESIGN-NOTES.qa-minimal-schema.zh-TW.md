# Q&A Minimal Schema

狀態：design draft

這份筆記定義一套由 Mycel 承載的 Q&A 應用最小 app-level schema。

這些 schema 不是新的 core protocol primitives。
它們是要放在一般 Mycel documents 裡、並由 Q&A app 治理的邏輯記錄形狀。

## 0. 範圍

這份最小 schema 涵蓋四種 record families：

- `question`
- `answer`
- `resolution`
- `citation_set`

目標是在保持 schema 狹窄的前提下，讓第一版 client 更容易開工。

## 1. 一般規則

1. 所有 records 都是 app-level JSON payload；若經過一般 document flow 做簽章或雜湊，應遵守 Mycel 的 canonical serialization 規則。
2. 這裡出現的 ID 都是 app-level logical IDs，不是新的 protocol object types。
3. 在 strict profile 下，未知欄位應拒絕；只有 app profile 明確允許 extension 時，才可忽略未知欄位。
4. 時間戳一律使用 Unix 秒整數。
5. reader client 在 active profile 下，應把 `resolution.accepted_answer` 視為預設顯示答案。

## 2. 共用欄位慣例

### 2.1 共用必要欄位

每種 record family 都應帶有：

- `type`
- `app_id`
- `created_at`
- `updated_at`

### 2.2 建議 ID Prefixes

建議的 logical ID prefixes：

- `q:` 給 questions
- `ans:` 給 answers
- `res:` 給 resolutions
- `cit:` 給 citation sets

這些 prefixes 只是 app-level 慣例。

## 3. Question Schema

### 3.1 必要欄位

- `type`：必須是 `question`
- `question_id`：問題的 logical ID
- `app_id`：Q&A app 識別碼
- `asked_by`：提交者金鑰或 app-level actor ID
- `title`：問題短標題
- `body`：問題全文
- `status`：`open`、`under-review`、`answered`、`accepted`、`superseded`、`closed` 之一
- `created_at`
- `updated_at`

### 3.2 可選欄位

- `topic_tags`：短主題字串陣列
- `target_corpus`：corpus 或 document-set references 陣列
- `language`：BCP 47 語言標籤
- `answer_count`：為 UI 方便而快取的計數
- `current_resolution_id`：若已存在 resolution，則填其 logical ID

### 3.3 範例

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

### 4.1 必要欄位

- `type`：必須是 `answer`
- `answer_id`：答案的 logical ID
- `app_id`
- `question_id`
- `answered_by`：作者金鑰或 app-level actor ID
- `answer_kind`：`direct-answer`、`commentary`、`clarification`、`objection`、`applied-guidance` 之一
- `body`
- `source_mode`：`human-authored`、`runtime-assisted`、`mixed` 之一
- `created_at`
- `updated_at`

### 4.2 可選欄位

- `citations`：citation-set IDs 或 inline references 的陣列
- `supersedes_answer`：前一版答案 ID
- `summary`：短摘要
- `confidence_label`：app 自定的非數值標籤，例如 `draft`、`reviewed`、`stable`
- `runtime_receipt_refs`：若為 runtime-assisted，則可連到相關 effect receipt IDs

### 4.3 範例

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

### 5.1 必要欄位

- `type`：必須是 `resolution`
- `resolution_id`
- `app_id`
- `question_id`
- `candidate_answers`：非空答案 ID 陣列
- `accepted_answer`：答案 ID，而且必須同時存在於 `candidate_answers`
- `alternative_answers`：答案 ID 陣列
- `accepted_under_profile`：active profile 或 policy 識別碼
- `updated_at`

### 5.2 可選欄位

- `decision_trace_ref`
- `rationale_summary`
- `supersedes_resolution`
- `state_label`：app 自定標籤，例如 `draft`、`active`、`superseded`
- `created_at`

### 5.3 驗證規則

- `accepted_answer` 必須唯一。
- `alternative_answers` 不可包含 `accepted_answer`。
- `alternative_answers` 中的每個 ID 都必須同時存在於 `candidate_answers`。
- `candidate_answers` 不應出現重複值。

### 5.4 範例

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

### 6.1 必要欄位

- `type`：必須是 `citation_set`
- `citation_id`
- `app_id`
- `question_id`
- `answer_id`
- `references`：非空陣列
- `created_at`
- `updated_at`

### 6.2 Reference Item Shape

`references` 裡的每個 item 應包含：

- `source_id`
- `locator`

每個 reference 的可選欄位：

- `quote`
- `note`

### 6.3 頂層可選欄位

- `quote_policy`
- `notes`
- `source_bundle_id`

### 6.4 範例

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

## 7. 最小 Client Requirements

最小 Q&A client 應做到：

- 以 `question_id`、`title`、`status` 列出 questions
- 從 `resolution.accepted_answer` 顯示一個預設答案
- 暴露 `resolution.alternative_answers` 中的 alternatives
- 顯示目前答案對應的 citation references
- 顯示像 `resolution_id` 與 `decision_trace_ref` 這類 trace links

## 8. 最小 Runtime Requirements

若在 `qa-strict-v0.1` 之外啟用 runtime assistance，runtime 應做到：

- 把草稿答案寫成一般 `answer` records
- 透過 `source_mode` 標示來源
- 把相關 effect receipts 分開保存
- 不可自行把答案標成 accepted

## 9. Open Questions

- `question` 與 `resolution` 是否一定要分成不同 documents，還是可共置在同一個 app state document？
- 未來的非 strict profiles 是否應允許 inline citations，而不只接受 citation-set IDs？
- 未來的非 strict profiles 是否應讓 `confidence_label` 維持 free-form，還是收斂成更大的固定 enum？

## 10. Strict Profile v0.1

對第一個可互通 client，我建議先固定一個較窄的 strict profile：

- `profile_name`: `qa-strict-v0.1`
- `app_id`: 由部署自行決定，但對單一部署必須固定
- objective: 降低第一版 client 的 parser 與 UI 自由裁量

### 10.1 全域 Strict Rules

strict profile 應強制下列規則：

1. 四種 record families 都拒絕未知欄位。
2. 所有必要欄位都必須存在，且型別必須完全符合預期。
3. `title`、`body` 與所有 logical IDs 不可為空字串。
4. 規定為非空的陣列若為空，必須拒絕。
5. `candidate_answers`、`alternative_answers`、`references` 內若有重複 ID，必須拒絕。
6. `created_at` 必須小於或等於 `updated_at`。
7. 一筆儲存的 record 只能包含一個 primary logical object。

### 10.2 Strict Question Rules

- `language` 為必要欄位。
- `topic_tags` 可以存在，但每個 item 都必須是非空字串。
- `answer_count` 禁止，因為它屬於 cache-like 欄位，容易漂移。
- `current_resolution_id` 可選。

只允許下列欄位：

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

- `source_mode` 必須是 `human-authored`。
- `direct-answer`、`commentary`、`applied-guidance` 必須帶 `citations`。
- `citations` 若存在，內容只能是 citation-set IDs。
- 禁止 inline citation objects。
- 禁止 `runtime_receipt_refs`。
- `confidence_label` 若存在，只能是 `draft`、`reviewed`、`stable` 之一。

只允許下列欄位：

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

- `decision_trace_ref` 為必要欄位。
- `rationale_summary` 為必要欄位。
- `state_label` 若存在，只能是 `draft`、`active`、`superseded` 之一。
- `created_at` 為必要欄位。
- 必須且只能有一個 `accepted_answer`。
- 對每個 `(question_id, accepted_under_profile)`，最多只能有一個 active resolution。

只允許下列欄位：

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

- `references` 裡只能有包含 `source_id`、`locator`、可選 `quote` / `note` 的 objects。
- `quote_policy` 為必要欄位。
- 禁止 `source_bundle_id`。
- 每個 citation set 只能對應一個 `answer_id`。

只允許下列欄位：

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

在 `qa-strict-v0.1` 中：

- runtime 可以做 retrieval、indexing、notification
- runtime 不可發布 accepted answers
- runtime 不可發布 `answer` records
- 任何 machine assistance 都必須留在 strict accepted-answer path 之外

### 10.7 First-Client Benefit

這個 strict profile 刻意用彈性換取可預測實作：

- 較少的 parsing branches
- 較少的 UI ambiguity
- 較少的 governance drift
- 較容易做 interoperability testing
