# Interpretation Dispute Model

狀態：design draft

這份文件描述 Mycel 應如何建模、保留並仲裁 interpretation disputes（解釋爭議），同時避免靜默覆蓋 root text，或強迫所有 deployment 收斂成單一 universal reading。

相關文件：

- `DESIGN-NOTES.two-maintainer-role.*`：candidate-head 發布權與 accepted-head governance 的角色拆分
- `DESIGN-NOTES.maintainer-conflict-flow.*`：進入正式 dispute 前後的一般 maintainer-conflict 路徑
- `DESIGN-NOTES.client-non-discretionary-multi-view.*`：在固定 profile 規則下如何導出並呈現 accepted outputs

核心設計原則是：

- root text 與 interpretation 必須分離
- 相互矛盾的 interpretation 應成為明確的 dispute object，而不是看不見的覆寫
- dispute outcome 可以依 profile 而不同
- 即使某個 interpretation 成為 active accepted reading，其他 alternatives 仍應保持可審計

## 0. Goal

讓 Mycel deployments 能以以下方式處理解釋爭議：

- 保留被引用的 root text
- 保留相互競爭的 interpretations
- 記錄某個 interpretation 為何在某個 profile 下被採納
- 讓 reader 明白「採信是治理結果」，而不是絕對真理

這份文件對內容領域保持中立。

只要是存在 interpretation divergence（解釋分歧）的 reference text system，都可適用。

## 1. Core Rule

若某個 interpretation 在實質上逆轉、限縮、擴張或導向了所引用 root text 的表面文義，它不得靜默取代 root reading。

相反地，它應保持為以下其中一種明確狀態：

- disputed
- alternative
- profile-specific
- 或在明確治理流程下被拒絕

## 2. Dispute Layers

Interpretation disputes 應分成三層處理。

### 2.1 Text Layer

這一層回答：

- 正在被解釋的是哪些 root anchors
- 哪些 witnesses 與此相關
- 這個 interpretation 是否忠實引用文本

但這一層本身不直接決定最終 accepted interpretation。

### 2.2 Interpretation Layer

這一層承載互相競爭的 reading、explanation 或 application。

以下物件應位於此層：

- interpretation records
- interpretation disputes
- interpretation relationships

### 2.3 Governance Layer

這一層決定在某個 profile 下，哪個 interpretation 會成為 active。

可能結果包括：

- 一個 accepted interpretation
- 一個 accepted interpretation 加上 alternatives
- 尚未有 accepted interpretation
- profile-specific divergence

## 3. Dispute Types

不是所有分歧都屬於同一種衝突。

建議的 dispute classes：

- `textual-meaning`
- `scope-of-application`
- `contextual-reading`
- `translation-reading`
- `commentary-conflict`
- `practical-application`
- `doctrinal-divergence`

這些分類可幫助 clients 與治理流程區分一般分歧與高風險矛盾。

## 4. Core Record Families

### 4.1 Interpretation Record

表示一個提出中的 interpretation。

建議欄位：

- `interpretation_id`
- `work_id`
- `witness_id`
- `anchor_refs`
- `interpretation_kind`
- `body`
- `submitted_by`
- `source_mode`
- `created_at`

建議 `interpretation_kind`：

- `direct-reading`
- `contextual-reading`
- `comparative-reading`
- `commentary`
- `practical-application`
- `minority-reading`

### 4.2 Interpretation Dispute Record

表示針對一個或多個 interpretations 的正式爭議。

建議欄位：

- `dispute_id`
- `work_id`
- `anchor_refs`
- `dispute_type`
- `candidate_interpretations`
- `dispute_reason`
- `opened_by`
- `opened_at`
- `status`

建議 `status`：

- `open`
- `under-review`
- `accepted-under-profile`
- `closed-with-alternatives`
- `rejected`

### 4.3 Interpretation Resolution Record

表示爭議在治理層的結果。

建議欄位：

- `resolution_id`
- `dispute_id`
- `accepted_interpretation`
- `alternative_interpretations`
- `rejected_interpretations`
- `accepted_under_profile`
- `decision_trace_ref`
- `rationale_summary`
- `resolved_at`

### 4.4 Interpretation Citation Set

表示某個 interpretation 或 dispute 的文本依據。

建議欄位：

- `citation_set_id`
- `interpretation_id`
- `anchor_refs`
- `quoted_segments`
- `supporting_notes`
- `alignment_refs`

## 5. Minimum Admissibility Rule

若某個 interpretation 要進入正式 dispute review，至少應具備：

- 明確的 anchor references
- 可讀的 rationale
- 足以審計的 citation context

這並不保證它會被接受。

這只保證它具備可審查性。

## 6. Root Text Protection

以下行為應禁止：

- 為了符合某個 interpretation 而靜默改寫 root text
- 把 commentary 當成 root witness
- 把所有 rival interpretations 合成單一編輯過的 synthetic reading，卻不保留歷史

以下則應允許：

- 明確的 rival interpretations
- profile-specific acceptance
- minority 或 alternative interpretation 的保存

## 7. Governance Outcomes

Interpretation review 不應只有單純的輸贏二分。

建議的 outcomes：

- `accepted-as-default-reading`
- `accepted-as-profile-specific-reading`
- `accepted-as-minority-reading`
- `kept-as-commentary-only`
- `rejected-as-contrary-to-cited-text`
- `left-unresolved`

這樣 deployments 才能在保留分歧的同時，不假裝所有衝突都一定要收束成單一絕對結果。

## 8. Role Boundaries

這個 model 至少應區分：

- text-maintenance roles
- interpretation-publication roles
- dispute-review roles
- accepted-reading governance roles

這些角色可以重疊，但不應自動等同。

尤其：

- 發布 interpretation 不應自動帶來 governance weight
- governance acceptance 也不代表 root text 本身被改寫

## 9. Client Behavior

一個 conforming reader client 應：

- 若 active profile 有定義，則顯示一個 active accepted interpretation
- 顯示某個 interpretation 是否處於 disputed 狀態
- 保留 alternative interpretations 的可見性
- 顯示 citations 與 dispute rationale
- 清楚區分 root text、commentary 與 interpretation

Client 不應：

- 靜默隱藏所有 rival interpretations
- 把 profile-level 的 acceptance 呈現得像 universal truth

## 10. Example Flow

建議流程：

1. 某位 reader 或 maintainer 提交 interpretation record
2. 另一方開啟 interpretation dispute
3. 附上 citation sets 與 anchor references
4. reviewers 或 maintainers 進行審查
5. 發布 profile-specific 的 interpretation resolution
6. reader clients 顯示 accepted interpretation 與可見的 alternatives

## 11. Relation to Canonical Text Profile

這個 model 是 canonical text profile 的上層補件。

Canonical text profile 處理的是：

- works
- witnesses
- anchors
- commentary layers
- accepted readings

而這份 dispute model 追加的是：

- rival interpretation records
- dispute objects
- interpretation resolution outcomes

## 12. Minimal First Version

這個 model 的最小第一版 deployment 只需要求：

- interpretation records
- dispute records
- resolution records
- 透過 anchor references 進行 citation support
- 一個 profile-aware 的 accepted outcome

它不應要求：

- 自動語義分類
- 複雜的 interpretive school scoring
- 跨所有 deployments 的全球一致

## 13. Recommended Next Step

在這份文件之後，最實際的下一步是：

- 一份最小 interpretation dispute schema
- 一條建立在 canonical text 上的 example dispute flow
- `Why this interpretation` 與 `Alternative interpretations` 的 client mockups
