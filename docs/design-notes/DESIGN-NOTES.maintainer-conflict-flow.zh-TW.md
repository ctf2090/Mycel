# Mycel Maintainer Conflict Flow

狀態：design draft

這份文件描述 Mycel deployment 應如何建模並保留 maintainers 對某份文件的想法或方向產生衝突時的流程，而不是把衝突壓平成靜默覆蓋。

核心設計原則是：

- maintainers 應透過明確的 candidate heads 與 governance signals 競爭
- 即使某個結果成為 accepted，分歧本身仍應可審計
- accepted-head selection 解決的是 active presentation，不是抹去 rival history
- conflict 應被建模成 governed flow，而不是 last-writer-wins 的覆蓋模型

相關文件：

- `DESIGN-NOTES.two-maintainer-role.*`：editor-maintainer 與 view-maintainer 的角色拆分
- `DESIGN-NOTES.client-non-discretionary-multi-view.*`：accepted-head derivation 規則
- `DESIGN-NOTES.interpretation-dispute-model.*`：當衝突屬於實質解釋分歧時，如何保留 rival interpretations
- `DESIGN-NOTES.governance-history-security.*`：如何保留 proposal、approval、conflict 與 receipt 歷史

## 0. 目標

讓 deployment 能回答：

- 兩位 maintainers 如何表達 rival document ideas
- 一個結果如何成為 active accepted head
- 輸掉或未被接受的路徑如何仍保持可見、可審查

這份文件聚焦在由 Mycel history 承載的 document 與 interpretation conflict。

## 1. 核心規則

兩位 maintainers 不應透過反覆覆寫同一份可見 state，直到其中一方消失來「打架」。

相反地，conflict 應通過以下流程：

1. rival candidate publication
2. explicit governance signaling
3. accepted-head computation
4. preserved conflict history

## 2. 衝突中的角色

### 2.1 Editor-maintainer

editor-maintainer 可以：

- 發布新的 revisions
- 建立新的 candidate heads
- 提出 merges
- 在新 branch 上修訂既有內容

但 editor-maintainer 不會自動決定讀者最終看到哪個 accepted 結果。

### 2.2 View-maintainer

view-maintainer 可以：

- 發布 signed governance signals
- 支持某個 candidate head
- 參與 accepted-head selection

但 view-maintainer 不會只因為能治理 selection，就自動獲得 content-authoring authority。

### 2.3 Reader client

reader client 應：

- 從 verified objects 與固定 profile 規則計算 accepted head
- 顯示 alternatives 與 trace material，但不可靜默把它們升格為 active result

## 3. 基本衝突流程

### 第一步：一位 maintainer 發布候選版本

Maintainer A 發布一個表達 idea A 的 revision。

結果：

- 一個 candidate head 出現
- 但此時尚未強制成為普遍 accepted

### 第二步：另一位 maintainer 發布 rival candidate

Maintainer B 發布另一個表達 idea B 的 revision。

結果：

- revision graph 內出現 rival candidate heads
- 兩者都可能是合法候選

### 第三步：治理訊號累積

View-maintainers 發布 signed View objects 或等價的 governance signals。

這些訊號可能：

- 支持 A
- 支持 B
- 保持未決
- 依 profile 或 epoch 出現分裂

### 第四步：執行 accepted-head selection

active accepted head 由以下元素共同計算：

- eligible candidate heads
- valid governance signals
- 固定的 profile 規則
- tie-break 邏輯

結果：

- 對某個 profile 與 selection time 而言，某一個 candidate 可能成為 active accepted head

### 第五步：rival history 保留

未被接受的 head 不會被刪除。

它仍然保留為：

- branch candidate
- alternative reading
- 未來可能被 accepted 的材料
- conflict 本身的證據

## 4. 兩種主要衝突類型

### 4.1 Editorial Conflict

這是對 wording、結構、收錄範圍或文件方向的衝突。

建議處理方式：

- 保留雙方 candidate heads
- 由 governance signals 選出目前 accepted 的版本
- 保留 branch 可見性，供 audit 與未來重新考量

### 4.2 Interpretation Conflict

這是會實質改變 meaning、scope、doctrine 或 application 的衝突。

建議處理方式：

- 不要只把它當作 branch preference
- 應開啟明確的 dispute object 或等價的 interpretation-dispute 流程
- 保留 rival interpretations 與之後的 governed resolution

## 5. 什麼叫做「贏」

在 Mycel 裡，maintainer conflict 的「贏」，通常應表示：

- 在某個 profile 與 selection window 下，成為 active accepted head

它不應表示：

- 刪除 rival branch
- 抹去 rival interpretation
- 把 conflict 從歷史中改寫掉

## 6. Client 應如何呈現

### 6.1 Reader UI

reader-oriented client 應顯示：

- active accepted head
- governing profile
- 足以解釋該 head 為何成為 active 的 trace information
- 將 alternative heads 顯示為 alternatives，而不是當成不可見垃圾

### 6.2 Governance UI

governance-capable client 應顯示：

- 哪些 maintainer keys 發布了哪些 candidates
- 哪些 governance signals 支持哪個 head
- 衝突目前是 unresolved、accepted 還是 disputed
- 該分歧屬於 editorial 還是 interpretive

## 7. 升級路徑

不是每一個 conflict 都應立刻升級成正式 dispute。

一個實務上的升級路徑是：

1. 出現 rival candidate heads
2. governance signals 先嘗試一般 accepted-head selection
3. 若分歧仍屬高風險或改變實質意義，則開啟 explicit dispute flow
4. 把後續 resolution 保留為獨立的 governed record

## 8. 常見 Failure Cases

### 8.1 Silent overwrite

某位 maintainer 的 revision 在 UI 中覆蓋另一位 maintainer 的版本，但 branch 或 trace 沒有保留。

結果：

- readers 失去 auditability
- conflict 變成不可見

### 8.2 把 Writer Power 當成 Selector Power

把 editor-maintainer 寫出較多 revisions，錯當成自動取得 acceptance weight。

結果：

- governance 與 authorship 坍縮為單一 authority

### 8.3 把 Interpretation Conflict 當成小編輯

一個會改變意義的衝突，只被當成普通 branch preference。

結果：

- 系統掩蓋了實質性的分歧

### 8.4 Accepted 結果沒有可見理由

client 只顯示一個 accepted head，卻沒有顯示選出它的 signal path。

結果：

- acceptance 看起來像任意或絕對，而不是 governed

## 9. 實務判準

如果兩位 maintainers 對某個文件想法有衝突，系統至少應保留這三件事：

- rival candidate content
- 選出 active result 的 governance process
- 未來由另一個 profile、epoch 或 dispute outcome 改採別條路徑的可能性

這就是 Mycel 把 maintainer conflict 轉成 governed history，而不是 destructive overwrite 的方式。
