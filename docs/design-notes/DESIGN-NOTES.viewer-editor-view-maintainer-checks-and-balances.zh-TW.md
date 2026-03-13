# Viewer-Editor-View-Maintainer Checks and Balances

狀態：design draft

這份筆記提議一套 Mycel 的三方制衡模型，讓以下 3 種角色各自保有不同權力：

- `viewer`
- `editor-maintainer`
- `view-maintainer`

目標是讓 editors 提出內容候選版本、讓 view maintainers 治理 accepted-head selection，並讓 viewers 能提供受限的公眾制衡訊號，而不把 Mycel 直接變成純人氣系統。

用比較直白的話說，這個原則是：「我希望我的反對者存在，不然我一定會自己玩到爆掉。」

這份模型也遵守一條治理原則：Mycel 應保留有意義的反對力量。若一個角色長期不再面對可信、持續存在的反對者，它就更容易在沒有外部制衡的情況下自我強化、過度擴張，最後把系統推向失衡。保留反對者的意義，不是為了製造敵意，而是為了讓系統持續保有煞車、質疑與糾偏能力。

相關文件：

- `DESIGN-NOTES.two-maintainer-role.*`
- `DESIGN-NOTES.client-non-discretionary-multi-view.*`
- `DESIGN-NOTES.maintainer-conflict-flow.*`

## 0. 目標

保留：

- 決定性的 accepted-head selection
- 明確的 candidate-head authorship
- 由 profile 規則治理的 view-maintainer authority
- 可審計的 decision trace

加入：

- 有意義的 viewer-side checks
- 受限的 challenge power
- 更清楚的 proposal、ratification 與 public objection 分工

避免：

- 把 accepted-head selection 變成單純比按讚數
- 讓任何單一角色能單方面定案
- 讓 viewer challenge path 容易被 Sybil 濫用

## 1. 角色模型

### 1.1 Viewer

`viewer` 負責消費 accepted output，也可以發布受限的 public-confidence signals。

能力：

- 閱讀 accepted heads 與 alternatives
- 發出 `approval`
- 發出 `objection`
- 發出 `challenge`
- 發出較低嚴重度的 `flag`

預設不具備：

- 不直接擁有 selector weight
- 不能單方面 override accepted head
- 不會只因為 viewer 身分就能發布 maintainer-grade revisions 或 governance Views

### 1.2 Editor Maintainer

`editor-maintainer` 負責提出候選內容狀態。

能力：

- 發布 `patch`
- 發布 `revision`
- 建立 candidate heads

預設不具備：

- 不自動取得 selector weight
- 不具單方面 accepted-head finality

### 1.3 View Maintainer

`view-maintainer` 負責治理 accepted-head selection。

能力：

- 發布 `View` governance signals
- 依照 profile 規則累積 `effective_weight`
- 在候選版本中進行 ratification

預設不具備：

- 不會只因治理身分就自動取得直接改寫內容的權力

## 2. 憲政類比

這個模型大致可類比為：

- editors 是提案者或起草者
- view maintainers 是具有治理權重的 ratifiers
- viewers 是受限的公眾制衡力量

accepted head 仍然是依規則選出的當前有效輸出。
真正的憲政層則仍然是 profile。

## 3. 核心原則

系統應刻意拆開 3 種權力：

1. proposal power
2. ratification power
3. public-confidence challenge power

任何單一角色都不應同時壟斷這三條路徑。

## 4. Viewer 訊號類型

viewer 的影響力不應只被建模成一種模糊的票數。

至少要區分：

- `approval`：正向支持，主要偏 advisory
- `objection`：反對，但不一定附完整證據
- `challenge`：更強的主張，表示這個 candidate 應進入正式審查
- `flag`：較低嚴重度的提醒或 review request

這些訊號不應有完全相同的治理效果。

## 5. 治理效果

### 5.1 Approval

`approval` 應：

- 表達公眾接受度
- 可選擇性提供受限的 public-confidence bonus
- 不取代 view-maintainer selector weight

### 5.2 Objection

`objection` 應：

- 表達有意義的公眾反對
- 在達到門檻時提高即時採認的門檻
- 能觸發 `delay`

### 5.3 Challenge

`challenge` 應：

- 比 objection 更強
- 最好帶有 reason code、citation 或 evidence reference
- 能觸發 `review`
- 只有在高門檻下，才可能促成 `temporary_freeze`

### 5.4 Flag

`flag` 應：

- 記錄低嚴重度的疑慮
- 支援 moderation 或 review triage
- 不應只靠自己就直接凍結採認

## 6. 雙層採認

最乾淨的結構，是採用雙層採認模型。

### Layer A：Candidate Formation

這一層回答：

- 哪些 revisions 在結構上有效
- 哪些 heads 是 eligible candidates
- editor admission 規則是否要進一步縮小候選集

### Layer B：Governance and Public Confidence

這一層回答：

- 哪個 candidate 擁有最高的 view-maintainer selector support
- viewer objection 或 challenge 是否足以延緩、審查，或暫停採認

換句話說：

- editors 建立候選
- view maintainers 在候選中進行 ratification
- viewers 以受限規則延緩或挑戰 ratification

## 7. Delay、Review、Temporary Freeze

viewer 訊號通常不應直接硬選 accepted head。
它更適合用來控制 escalation。

### 7.1 Delay

`delay` 是最輕的介入。

適用於：

- viewer objection 明顯升高
- 有爭議，但證據尚不足以進入硬性審查

效果：

- 讓 candidate head 延後進入 active 狀態，先進入短暫等待期

### 7.2 Review

`review` 是正式升級。

適用於：

- viewer challenge 跨過門檻
- challenge evidence 並非空泛
- governance path 需要明確重新檢查

效果：

- 在最終生效前，要求額外的 view-maintainer review、moderation，或 dispute handling

### 7.3 Temporary Freeze

`temporary_freeze` 是最強的介入。

只應用於：

- viewer challenge 既大量又高可信
- 出現 policy violation、procedural abuse，或緊急風險證據

效果：

- 在 review path 完成前，先阻止 candidate 進入 active 狀態

它應比 delay 或 review 更難觸發，而且應屬少見事件。

## 8. 為什麼一定需要 Anti-Sybil

一旦 viewers 可以觸發 delay、review 或 freeze，raw viewer count 就變成治理相關訊號。

如果沒有 anti-Sybil：

- 單一行動者可以生出很多假 viewers
- editor 可以自導自演大量 public approval
- 對手也可以大量灌 objection，讓內容永遠無法採認

所以 viewer influence 至少需要下列其中一種保護：

- identity cost
- reputation accumulation
- governance admission
- 嚴格受限的 viewer powers

## 9. Viewer Anti-Sybil 選項

### Option A：Costly Identity

要求 stake、等待期，或其他不可忽略成本，之後 viewer challenge power 才完整生效。

取捨：

- anti-Sybil 較強
- onboarding 較慢

### Option B：Reputation-Based Viewer Weight

讓 viewer 的 challenge strength 只能在長期正常參與後逐步增加。

取捨：

- 更符合長期 civic trust
- 設計較複雜

### Option C：Governance-Admitted Viewers

要求先被授予資格，viewer 才能發出 challenge-grade signals。

取捨：

- 較容易控制濫用
- 中心化程度較高

### Option D：Bounded Civic Signals

允許廣泛 viewer 參與，但只讓 viewers 觸發 `delay` 或 `review_request`，不直接給 freeze power。

取捨：

- 遷移最安全
- checks 較弱

## 10. 建議方向

對 Mycel 而言，最安全的第一步是：

- 仍以 view-maintainer selector weight 作為主 ratification 機制
- 新增 viewer `approval`、`objection`、`challenge`、`flag`
- 讓 objection 觸發 `delay`
- 讓 challenge 觸發 `review`
- 只有在更高門檻、且最好搭配更強 anti-Sybil 條件或 maintainer corroboration 時，才允許 `temporary_freeze`

這樣可以保留目前治理骨架，同時加入真正的 viewer-side checks。

## 11. 最小政策欄位形狀

未來 profile 可以定義像這樣的欄位：

- `viewer_objection_delay_threshold`
- `viewer_challenge_review_threshold`
- `viewer_freeze_threshold`
- `viewer_signal_cost_model`
- `viewer_signal_weight_cap`
- `viewer_challenge_requires_evidence`

這些應維持為 profile-level rules，而不是臨時本地 client settings。

## 12. Viewer 制衡力評估

按照目前這份提案，viewer 的制衡力是非對稱的。

它對 `editor-maintainer` overreach 相對有力，原因是：

- viewers 可以延緩 candidate activation
- viewers 可以把高爭議 candidate 升級進入 review
- editors 不能只靠 proposal power 就立即取得 accepted 狀態

但它對 `view-maintainer` 的協同行為則較弱，原因是：

- viewers 仍然不掌握 primary selector weight
- viewers 不能直接指定 accepted head
- 一旦 review 壓力被解除，形成協調共識的 view-maintainer 多數通常仍能完成定案

所以目前這版 draft 比較準確的讀法是：

- 對 editors 有較強的程序性制衡
- 對 view maintainers 有中等的程序性制衡
- 對公眾直接否決權則維持受限

## 13. 補強方案

如果我們希望 viewer 的制衡更有力、但又不把系統直接改成人氣治理，最相容的補強方式有 3 種：

### 13.1 Mandatory Re-Review

高可信 viewer challenge 可以強制要求 candidate 在生效前多進一輪 view-maintainer review。

取捨：

- viewer check 會明顯變強
- 爭議案例的採認速度會變慢

### 13.2 High-Threshold Freeze

viewer challenge 可以觸發 `temporary_freeze`，但其門檻必須比一般 review 更高，並搭配更強 anti-Sybil 與 evidence 條件。

取捨：

- civic check 最強
- 若 anti-Sybil 不夠，濫用風險最高

### 13.3 Corroborated Freeze Release

如果 candidate 已被 freeze，解除 freeze 不應只靠原本那批狹窄 maintainer 聯盟簡單重投。

可行模式：

- 要求更大的 view-maintainer quorum
- 要求最短 delay window
- 要求獨立的 challenge resolution 或 moderation review

取捨：

- 可避免自我快速洗白
- 會增加程序成本

對 Mycel 而言，最平衡的下一步大概會是：

- 仍不讓 viewers 進 primary selector weight
- 讓 viewer challenge 能強制觸發 mandatory re-review
- 把 freeze 保留給高信任、高證據門檻的案例

## 14. 取捨

好處：

- 權力分工更清楚
- 更能抑制 maintainer overreach
- 公眾信心訊號更可見
- 在爭議內容正式生效前，有更好的升級機制

成本：

- protocol 與 profile 都會更複雜
- anti-Sybil 會變成無法逃避的設計題
- challenge spam 與 moderation burden 會變成真問題
- accepted-head activation 在爭議情況下會較不即時

## 15. 開放問題

- viewers 是否應該永遠只有 escalation power，而不直接拿 selector weight？
- viewer approvals 應只影響 tie-break，還是可提供受限的 score bonus？
- `temporary_freeze` 是否應要求 viewer challenge 與 view-maintainer concurrence 同時成立？
- viewer challenge identity 應該是 profile-local、network-global，還是 application-specific？
- 低信任 viewers 是否只能觸發 review，而不能觸發 freeze？
