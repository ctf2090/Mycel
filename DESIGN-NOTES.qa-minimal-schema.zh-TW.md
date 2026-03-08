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

若啟用 runtime assistance，runtime 應做到：

- 把草稿答案寫成一般 `answer` records
- 透過 `source_mode` 標示來源
- 把相關 effect receipts 分開保存
- 不可自行把答案標成 accepted

## 9. Open Questions

- `question` 與 `resolution` 是否一定要分成不同 documents，還是可共置在同一個 app state document？
- strict profile 是否允許 inline citations，還是只允許 citation-set IDs？
- `confidence_label` 是否應維持 free-form，還是第一個 profile 就把它收斂成固定 enum？
