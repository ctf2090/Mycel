# Two-Maintainer-Role Model

狀態：design draft

這份筆記提議把 maintainer 責任拆成兩種明確角色：

- 一種能發布新的 candidate heads
- 一種能影響 accepted-head selection

相關文件：

- `DESIGN-NOTES.maintainer-conflict-flow.*`：當 rival maintainer ideas 變成 competing heads 時的完整流程
- `DESIGN-NOTES.interpretation-dispute-model.*`：當分歧已嚴重到成為正式 interpretation dispute 時的處理模型
- `DESIGN-NOTES.client-non-discretionary-multi-view.*`：client 端如何依這些 governance signals 計算 accepted head

目標是避免把「能寫內容」和「能決定 reader 看什麼」預設成同一種權力。

## 0. 目標

拆開：

- content-production authority
- acceptance-governance authority

保留：

- multi-head 文件歷史
- signed governance signals
- profile-governed accepted-head selection

允許：

- 同一把 key 可持有其中一種角色，或同時持有兩種角色

## 1. 提議角色

### 1.1 Editor Maintainer

`editor-maintainer` 可以建立新的 candidate document heads。

能力：

- 發布 `patch`
- 發布 `revision`
- 建立新的 branch head
- 若 profile 允許，可建立 merge revisions

預設不具備：

- 不會自動影響 accepted-head selection
- 不會自動獲得 selector weight

### 1.2 View Maintainer

`view-maintainer` 可以發布影響 accepted-head selection 的 signed governance signals。

能力：

- 發布 `view`
- 貢獻 selector signals
- 依 profile 規則累積 selector weight

預設不具備：

- 不會因為能發布 views 就自動取得發布 revisions 的權利

### 1.3 Dual-Role Maintainer

同一把 key 可以同時擁有：

- `editor-maintainer`
- `view-maintainer`

這樣某個 maintainer 既能建立新 heads，也能參與 accepted-head selection，但兩種權力在概念上仍然分離。

## 2. 為什麼要拆角色

如果不拆，協議很容易隱含：

- 寫越多內容的人，也應該越能決定 reader 看什麼
- curator influence 和 content authorship 是同一種權力
- 活躍編修者會因為高產而自然壟斷 selection

拆開後，治理會更清楚：

- editors 建立候選內容
- view maintainers 在候選內容中做治理選擇

## 3. 協議解讀

在這個模型下：

- `revision` 發布屬於 editor 側行為
- `view` 發布屬於 governance 側行為
- accepted-head selection 仍然由 `view` signals 驅動，而不是單靠 revision authorship

這表示：就算某個新 head 沒有立即得到 accepted-head 支持，它仍然可以合法存在於 revision graph 裡。

## 4. 準入與權重

### 4.1 Editor-Maintainer 準入

network profile 可以定義誰能發布 maintainer-grade revisions。

可行策略：

- 所有有效 author keys 都可開放發布
- 只有 admitted editor-maintainer keys 才能發布官方 candidate heads
- 混合模式：所有 authors 都能發布 revisions，但只有 editor-maintainers 的 heads 會被標成正式候選

### 4.2 View-Maintainer 準入

view-maintainer 的準入應獨立存在。

selector weight 應只由以下來源導出：

- valid View publication history
- profile-defined admission rules
- profile-defined penalty rules

revision 產出本身不應直接變成 selector weight。

## 5. 建議的規則邊界

最乾淨的規則邊界是：

- `patch` / `revision` 權力不代表 governance weight
- `view` 權力不代表 content-publishing authority
- dual-role key 必須分別滿足兩條準入路徑

這樣能降低權力意外集中。

## 6. Accepted-Head Selection

accepted-head selection 仍應使用：

- eligible revisions 作為 candidates
- View objects 作為 governance signals
- 固定 profile 規則決定 weights 與 tie-breaks

editor-maintainers 重要，是因為他們建立 candidate heads。
view-maintainers 重要，是因為他們影響哪個 candidate 會成為 active accepted head。

## 7. Reader 與 Curator 行為

### 7.1 Reader Client

reader client 應：

- 顯示由 View-maintainer signals 導出的 accepted heads
- 把 editor 產生的其他 heads 顯示成 branch candidates
- 除非同一把 key 也有 view-maintainer 角色，否則不要把 editor authority 當成 selector authority

### 7.2 Curator 或 Governance Client

具治理能力的 client 應：

- 驗證哪些 keys 具有 editor-maintainer 身分
- 驗證哪些 keys 具有 view-maintainer 身分
- 讓這兩種角色指派可被審計

## 8. Data Model 選項

有 3 個可行表示法。

### Option A: Two Explicit Role Types

直接定義兩種角色：

- `editor-maintainer`
- `view-maintainer`

取捨：

- 語義最清楚
- protocol change 較大

### Option B: One Maintainer Type, Two Capabilities

保留單一 maintainer 概念，但定義兩種 capability：

- `can_publish_revision`
- `can_publish_view`

取捨：

- spec change 較小
- 概念清晰度較弱

### Option C: Authors + View Maintainers

讓所有 authors 都能建立 revisions，只把治理權留給 view maintainers。

取捨：

- governance model 最簡單
- ordinary authors 和 high-trust editors 的差異較弱

## 9. 建議方向

對 Mycel 而言，Option A 是最清楚的長期方向：

- 它符合內容創作與採信治理分離的概念
- 它和 profile-governed accepted-head 模型相容
- 它能避免把治理權悄悄滲進 revision authorship

如果想要最少破壞性的遷移路徑，Option B 則是比較容易的短期步驟。

## 10. 建議的未來規範語句

未來 spec 可考慮加入：

- An editor-maintainer MAY publish Patch and Revision objects that create new candidate heads.
- A view-maintainer MAY publish View objects that contribute governance signals to accepted-head selection.
- Selector weight MUST be derived from View-maintainer behavior only, unless a future profile explicitly defines another signal source.
- Holding editor-maintainer status MUST NOT, by itself, grant selector weight.
- A single key MAY hold both editor-maintainer and view-maintainer status.

## 11. 開放問題

- editor-maintainer admission 應由 protocol 定義，還是留給 profile policy？
- 所有有效 revisions 都該成為 candidate heads，還是只限 admitted editor-maintainers 發布的 revisions？
- dual-role key 應共用一個 identity record，還是拆成兩個 role-specific records？
- implementation checklist 是否應再拆成 writer、editor-maintainer、view-maintainer 三條流程？
