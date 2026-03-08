# Q&A App Layer

狀態：design draft

這份筆記描述一個由 Mycel 承載的 Q&A 應用層，同時把特定內容領域的分析與諮詢流程留在核心協議之外。

核心原則是：

- Mycel 承載問題狀態、回答狀態、引用、治理訊號與審計歷史
- client 讓使用者提問、瀏覽並檢查 accepted answers
- optional runtime 可協助做檢索、通知或草稿準備
- core protocol 保持中立且純技術化

## 0. 目標

讓 Mycel 可以承載一個可持久保存的 Q&A 系統，同時不把核心協議變成特定領域的問答引擎。

放在 Mycel 裡：

- Q&A app definition
- question documents
- answer documents
- citation links
- accepted-answer governance state
- optional retrieval 或 notification effect history

留在 Mycel core 外：

- 特定領域的真值主張
- 世界觀式背書
- 私密諮詢判斷
- external search 或 HTTP execution
- secrets 與 runtime credentials

## 1. 設計規則

Q&A App Layer 應遵守五條規則。

1. 問題與回答是 app-level state，不是 protocol primitives。
2. 多個 candidate answers 可以並存。
3. 合規的 reader client 應在固定 active profile 下顯示一個 active accepted answer。
4. alternative answers 應保留可審計性，並以非 active branches 或 alternatives 的形式可見。
5. runtime-assisted output 在經過正常治理接受前，必須只被視為 candidate content。

## 2. 三層分工

### 2.1 Client Layer

client 是面向使用者的層。

責任：

- 建立 question intents
- 顯示 accepted 的 question 與 answer state
- 顯示 citations、answer history 與 alternative answers
- 顯示一個答案是 human-authored、runtime-assisted 或 mixed
- 顯示 governance 與 audit traces

非責任：

- 不重定義 accepted-answer 規則
- 不直接決定特定領域的採信結果
- 當 protocol 要求保留審計可見性時，不得隱藏 alternative answers

### 2.2 Runtime Layer

runtime 是 optional 且 assistive 的層。

責任：

- 讀取 accepted Q&A state
- 執行已核准的 retrieval 或 indexing effects
- 發布 effect receipts
- 視情況準備 draft answers 或 citation suggestions

非責任：

- 不可自行發布 accepted answers
- 不可繞過 view-maintainer governance
- 不可把草稿輸出當成已採用的最終內容

### 2.3 Effect Layer

effect layer 描述 Q&A app 可能使用的 optional side effects。

例子：

- corpus retrieval
- notification delivery
- 對已核准來源做 HTTP fetch
- background indexing

effect objects 應保持顯式、可審計、且 replay-safe。

## 3. 核心 Q&A 物件

### 3.1 Q&A App Manifest

定義 Q&A 應用本身。

建議欄位：

- `app_id`
- `app_version`
- `question_documents`
- `answer_documents`
- `resolution_documents`
- `allowed_effect_types`
- `citation_policy`
- `runtime_profile`

用途：

- 識別 Q&A app
- 宣告參與的 document families
- 宣告 citation 與 runtime 的預期

### 3.2 Question Document

表示一個問題 thread。

建議欄位：

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

典型 `status` 值：

- `open`
- `under-review`
- `answered`
- `accepted`
- `superseded`
- `closed`

### 3.3 Answer Document

表示對一個問題的某個 candidate answer。

建議欄位：

- `answer_id`
- `question_id`
- `answered_by`
- `answer_kind`
- `body`
- `citations`
- `source_mode`
- `created_at`
- `supersedes_answer`

建議 `answer_kind` 值：

- `direct-answer`
- `commentary`
- `clarification`
- `objection`
- `applied-guidance`

建議 `source_mode` 值：

- `human-authored`
- `runtime-assisted`
- `mixed`

### 3.4 Resolution Document

表示一個問題的 accepted-answer 狀態。

建議欄位：

- `resolution_id`
- `question_id`
- `candidate_answers`
- `accepted_answer`
- `alternative_answers`
- `accepted_under_profile`
- `rationale_summary`
- `updated_at`

用途：

- 識別目前 accepted 的答案
- 保留替代答案
- 顯示導出這個結果所使用的 governing profile

範例：

```json
{
  "type": "qa_resolution",
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
  "rationale_summary": "Selected because it has the strongest accepted citation set and matching governance support under the active profile.",
  "updated_at": 1772942400
}
```

這個範例展示了一個常見的 Q&A 模式：

- 一個 question
- 多個 candidate answers
- 一個目前 active 的 accepted answer
- 可見的 alternatives
- 明確的 profile reference
- 一個讓 client 解釋「為什麼現在顯示這個答案」的 trace handle

### 3.5 Citation Set

表示一個答案的文本依據。

建議欄位：

- `citation_id`
- `question_id`
- `answer_id`
- `references`
- `quote_policy`
- `notes`

用途：

- 把答案連到 source texts 或 prior commentary
- 支援 auditability
- 降低純自由裁量式的答案採信

### 3.6 Optional Effect Request 與 Effect Receipt

這些物件沿用其他 App Layer 筆記已描述的模式。

在 Q&A app 裡的典型用途：

- 請求從已核准的 corpus index 做 retrieval
- 請求通知訂閱者
- 紀錄 runtime 完成 indexing

## 4. 治理模型

Q&A app 應沿用 Mycel 其他部分相同的 accepted-head 原則。

建議規則：

- question 與 answer documents 可以分支
- view-maintainers 發布 signed governance signals
- active accepted answer 由固定 active profile 導出
- editor-maintainers 可以發布 candidate answers，但預設不取得 acceptance weight
- reader clients 以 accepted answer 作為預設顯示結果
- reader clients 也要暴露 alternative answers 與 resolution history，供 audit 使用

這樣可以在保留多答案歷史的同時，限制 client 的自由裁量。

## 5. Answer Traceability

Q&A app 應讓 client 能把任何收到的答案，同時沿著內容歷史與採信歷史一路追回去。

建議的追溯鏈：

1. 識別目前顯示的 `answer_id`
2. 把它對應到上層的 `question_id`
3. 找到該答案的 revision 歷史與 signed authorship trail
4. 檢查它的 citations 與被引用的 source texts
5. 檢查把它標成 accepted 或 alternative 的 `resolution` document
6. 檢查讓 client 把它顯示為 active accepted answer 的 fixed profile 與 decision trace
7. 若有 runtime assistance，則檢查相關 effect requests 與 effect receipts

這表示一個答案至少應能沿五個維度被追溯：

- content history
- authorship 與 signatures
- citations 與 source basis
- resolution state
- governance 與 selector output

### 5.1 Minimum Client Trace View

合規的 Q&A client 對目前顯示的答案，至少應能顯示下列欄位：

- `answer_id`
- `question_id`
- 目前的 `revision_id`
- `answered_by`
- `source_mode`
- citation references
- `resolution_id`
- 它是 active `accepted_answer` 還是 `alternative_answer`
- `accepted_under_profile`
- decision-trace reference 或 summary

### 5.2 Why-Am-I-Seeing-This View

我建議 reader clients 暴露一個專門的檢查檢視，例如 `Why this answer`。

這個檢視應解釋：

- 目前哪個答案是 active
- 是哪個 resolution document 選中了它
- 是哪個 fixed profile 治理了這個結果
- 哪些 signed governance signals 參與了這個結果
- 還有哪些 alternatives 仍然可用

### 5.3 Runtime Contribution Trace

若一個答案帶有 runtime assistance，追溯資訊也應暴露：

- 相關的 `effect_request_id`
- 相關的 `effect_receipt_id`
- executor 身分
- retrieval 或 generation mode
- accepted answer 之後是否又被 human editor-maintainer 修訂

runtime contribution 必須從屬於正常的 answer governance。
traceability 應把這條邊界清楚呈現給使用者。

## 6. 建議流程

### 6.1 Human-Curated Flow

1. 使用者提交一個問題
2. editor-maintainers 發布一個或多個 candidate answers
3. citations 被附上或驗證
4. view-maintainers 發布治理訊號
5. active profile 導出一個 accepted answer
6. client 以該答案為預設顯示，並保留可檢查的 alternatives

### 6.2 Runtime-Assisted Flow

1. 使用者提交一個問題
2. runtime 檢索相關資料或準備草稿
3. runtime 發布 effect receipt 與 optional draft answer
4. editor-maintainers 修訂或背書該草稿
5. view-maintainers 依照平常規則治理 accepted-answer 狀態

這樣可以讓 machine assistance 從屬於正常治理。

## 7. Guardrails

Q&A app 應採保守的 guardrails。

- 宣稱具有權威性地位的答案，預設應帶 citations。
- runtime 產生的草稿應清楚標示。
- accepted answer 不應被視為 protocol-level 的全域真理。
- reader clients 應明確區分 `accepted answer` 與 `only possible answer`。
- private counseling 或 confidential guidance，除非部署本來就打算如此，否則不應存進大範圍複製的公開 documents。
- moderation 或 safety filtering 可以存在，但在合規 reader client 中，不應靜默改寫 fixed-profile accepted answer。

## 8. 最小 v0.1 Q&A Profile

對第一版實作，我建議先採較窄的 profile。

- 只支援 human-authored 的問題與回答
- `direct-answer` 與 `commentary` 必須附 citations
- 不自動發布 runtime answer
- runtime 只限於 retrieval、indexing 與 notification
- 一個 accepted answer 加上可見的 alternatives
- resolution state 放在普通 document family，而不是新增 core protocol primitive

取捨：

- automation 較低
- governance clarity 較高
- interoperability 較容易

## 9. Open Questions

- `resolution` 應是 dedicated document family，還是 app 內的一般 state document？
- citation policy 是否應依 `answer_kind` 改變？
- runtime-assisted draft 是否應在 accepted state 中強制標記？
- 一個 profile 下應支援多個 accepted answers，還是只保留一個 primary answer 加上 alternatives？
