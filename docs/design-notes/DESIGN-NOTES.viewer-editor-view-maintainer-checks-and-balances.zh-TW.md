# Viewer-Editor-View-Maintainer Checks and Balances

狀態：design draft

這份筆記提議一套 Mycel 的三方制衡模型，讓以下 3 種角色各自保有不同權力：

- `viewer`
- `editor-maintainer`
- `view-maintainer`

目標是讓 editors 提出內容候選版本、讓 view maintainers 治理 accepted-head selection，並讓 viewers 能提供有界的公眾制衡訊號，而不把 Mycel 直接變成純人氣系統。

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
- 有界的 viewer selector participation
- 更清楚的 proposal、ratification 與 public objection 分工

避免：

- 把 accepted-head selection 變成單純比按讚數
- 讓任何單一角色能單方面定案
- 讓 viewer challenge path 容易被 Sybil 濫用

## 0.1 bounded viewer-in-selector lane 的決策摘要

對目前這條 `M3` follow-up 來說，讓 `viewer` 以有界方式進入 `selector_score` 已是這條 lane 的專案方向；這份筆記要做的是定義一個明確版本，而不是讓整條 lane 維持開放待定：

- 仍由 `view-maintainer` support 擔任主 ratification score
- `viewer` 只能透過兩條有界 score channels 參與：
  - 來自合格 `approval` 的 `bounded_viewer_bonus`
  - 來自合格 `objection` 的 `bounded_viewer_penalty`
- `challenge` 與 `flag` 不進主 score path；它們屬於 `review`、`delay`、`temporary_freeze` 等 escalation path
- raw viewer counts 永遠不能直接當 selector input
- viewer contribution 永遠不能與 `view-maintainer` 的 ratification weight 等權

因此，目標 selector shape 是：

`selector_score = maintainer_score + bounded_viewer_bonus - bounded_viewer_penalty`

並搭配以下 guardrails：

- `maintainer_score` 仍是主導性的 governance input
- viewer bonus 與 penalty 都必須由 profile 規則各自設 cap
- 每個 profile 都可以直接停用 viewer selector participation
- 更高影響的 viewer effects 必須走獨立的 anti-Sybil 與 evidence gates，而不是單純把 raw counts 放大

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

- 不擁有無上限或與 `view-maintainer` 等權的 selector weight
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

### 3.1 這句原則在 `viewer` 進入 `selector_score` 後怎麼實踐

在這份提案裡，`viewer` 會以有界方式進入 `selector_score`；因此這句「我希望我的反對者存在，不然我一定會自己玩到爆掉。」就不再只是態度聲明，而會落成幾個具體制度要求：

- 反對者必須能留下可計算的阻力，而不是只能在外圍表達情緒
- 這個阻力必須是有界的，不能把系統直接改成原始人氣計票
- 反對不只影響分數，也應能在高門檻下觸發 `review` 或 `temporary_freeze`
- 反對者本身也必須受 anti-Sybil、eligibility 與 signal-quality 條件約束，避免系統把假性 opposition 誤認成真正的制衡

換句話說，這條原則的制度化版本不是「讓 viewer 贏」，而是：

- 讓 `editor-maintainer` 不能只靠 proposal power 一路推進
- 讓 `view-maintainer` 不能只靠狹窄內部共識就完全消化外部反對
- 讓 `viewer` 有正式但受限的阻力渠道，而不是只有無後果的旁觀表態

若用公式語言濃縮，方向會更接近：

`maintainer_score + bounded_viewer_bonus - bounded_viewer_penalty`

再搭配：

- 高可信 `viewer_challenge_pressure` 可觸發 `review`
- 更高門檻且高證據的 challenge 可觸發 `temporary_freeze`

這樣保留反對者的意義，才會從一句政治語言，變成 accepted-head governance 的安全機制。

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

### 5.5 Editor 與 View-Maintainer 的 Penalty

若三角色制衡要成立，viewer 的 challenge 不應只有延緩效果，也應在高門檻且高證據條件下，能對 `editor-maintainer` 與 `view-maintainer` 形成正式 penalty path。

`editor-maintainer` 的 penalty 可用於：

- 持續提交低品質、明顯濫發或程序上有問題的 candidate
- 重複利用噪音 revisions 消耗 review 能量
- 配合假性 viewer support 或其他操弄行為

可能效果：

- 提案節流或短期 proposal cooldown
- 更嚴格的 candidate admission
- 在更高門檻下才允許進入正式 ratification
- 嚴重時進入 maintainer suspension / revocation review

`view-maintainer` 的 penalty 可用於：

- 重複忽略高品質 challenge 或明確 evidence
- 持續用狹窄同盟消化外部異議
- 發布程序上惡意、失實，或明顯濫權的 governance signals

可能效果：

- `effective_weight` 降低或暫時歸零
- 要求更大的 corroboration quorum
- 暫停其單獨參與高影響 ratification 的能力
- 嚴重時進入 maintainer suspension / revocation review

這裡的關鍵是：

- 不是每個 viewer objection 都直接懲罰 maintainer
- penalty 應要求更高的 evidence、review 結論，或多方 corroboration
- penalty 應被視為比 `delay` / `review` / `temporary_freeze` 更接近角色責任處分，而不只是 candidate-level intervention

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
- viewer 的有界 score channel 是否足以改變排序或提高採認門檻
- viewer objection 或 challenge 是否足以延緩、審查，或暫停採認

換句話說：

- editors 建立候選
- view maintainers 在候選中進行主 ratification
- viewers 以有界 score channel 與 escalation 規則共同影響 ratification

## 7. Delay、Review、Temporary Freeze

viewer 訊號不應單獨硬選 accepted head，但在這份提案中，它們會同時影響有界 selector input 與 escalation。

### 7.1 Delay

`delay` 是最輕的暫時性介入。

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

`temporary_freeze` 是最強的暫時性介入。

只應用於：

- viewer challenge 既大量又高可信
- 出現 policy violation、procedural abuse，或緊急風險證據

效果：

- 在 review path 完成前，先阻止 candidate 進入 active 狀態

它應比 delay 或 review 更難觸發，而且應屬少見事件。

## 8. 為什麼一定需要 Anti-Sybil

一旦 viewers 以有界方式進入 `selector_score`，並且還能觸發 delay、review 或 freeze，raw viewer count 就會變成更高風險的治理相關訊號。

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

### Option D：Bounded Civic Score Channel

允許廣泛 viewer 參與，並讓 viewers 以有界 score channel 影響排序，同時把高影響動作仍限制在 `delay`、`review_request` 或高門檻 freeze path。

取捨：

- 比 escalation-only viewers 更有力
- anti-Sybil 與權重治理壓力更高

### 未來若有生物特徵認證

如果未來出現成本、隱私與可靠性都可接受的人類生物特徵認證，它會明顯改變 viewer anti-Sybil 的設計空間。

可能帶來的好處：

- 更容易接近「一個自然人對應一個 challenge-capable identity」
- 廣泛 viewer participation 與強 civic checks 可以同時存在
- `temporary_freeze` 或其他高影響 viewer power 比較有機會安全開放

但它不會自動解掉所有問題：

- 生物特徵只能較好處理「是不是不同人」，不能保證「是不是有良好判斷」
- 仍然需要 reputation、evidence requirement、delay window 與 abuse recovery
- 會引入更重的隱私、排除性、與 credential custody 風險

因此，即使未來真的有成熟的生物特徵認證，Mycel 也更適合把它視為 anti-Sybil substrate，而不是直接把它等同於治理正當性本身。

## 9.1 在 viewer score 能影響 selector outcome 之前的最小 gating model

在任何 viewer-derived score 能進入 `selector_score` 之前，profile 應先定義一個最小 gating model，至少拆成 3 個獨立輸入：

- `viewer_identity_tier`
- `viewer_admission_status`
- `viewer_reputation_band`

這三個輸入分別處理不同問題：

- `viewer_identity_tier` 回答這個 viewer 的 anti-Sybil substrate 有多可信、多有成本
- `viewer_admission_status` 回答這個 viewer 在此 profile 下是否目前被允許使用 selector-relevant powers
- `viewer_reputation_band` 回答這個 viewer 透過過去非濫用參與累積了多少 durable trust

建議的最小形狀是：

- `viewer_identity_tier`
  - `none`
  - `basic`
  - `strong`
- `viewer_admission_status`
  - `pending`
  - `admitted`
  - `restricted`
  - `revoked`
- `viewer_reputation_band`
  - `new`
  - `established`
  - `trusted`

建議解讀：

- `none`
  - 沒有可信的 anti-Sybil substrate
  - viewer 只能閱讀與留下 non-selector feedback
- `basic`
  - 已有某種有界的 anti-Sybil friction
  - 若 profile 允許，viewer 只能提供低影響、受限的 selector signals
- `strong`
  - 已有更強的 anti-Sybil substrate，可能來自 costly identity、成熟 reputation，或隱私保護型 biometric proof
  - 若另外通過 admission，viewer 才能使用較高影響、但仍受限的 participation

## 9.2 eligibility 與 weight 應如何推導

profile 應從 gating state 推導 `effective_signal_weight`，而不是依賴 raw viewer counts 或 viewer 自報聲明。

較安全的方向是：

`effective_signal_weight = f(viewer_identity_tier, viewer_admission_status, viewer_reputation_band, signal_type, confidence_level)`

並搭配以下基準規則：

- 若 `viewer_admission_status != admitted`，selector-relevant weight 應為 `0`
- 若 `viewer_identity_tier = none`，selector-relevant weight 應為 `0`
- 若 `viewer_reputation_band = new`，即使被允許參與 selector，其 contribution 也應維持在 profile 定義的最低 bounded cap 之下
- `approval` 與 `objection` 只有在 identity 與 admission gates 通過後，才可取得受限的非零權重
- `challenge` 的 gating 主要應用來開啟 escalation eligibility，而不是去覆寫主 maintainer score

依 tier 建議的 profile 行為：

- `none`
  - `bounded_viewer_bonus = 0`
  - `bounded_viewer_penalty = 0`
  - 不提供 `challenge`-grade escalation
- `basic`
  - 只給很小、受 cap 限制的 `bounded_viewer_bonus`
  - 只給很小、受 cap 限制的 `bounded_viewer_penalty`
  - 只有在附 evidence 時，`challenge` 才能打開 review
- `strong`
  - 可給較高、但仍受 cap 限制的 `bounded_viewer_bonus`
  - 可給較高、但仍受 cap 限制的 `bounded_viewer_penalty`
  - 若 profile 也要求 corroboration 或更高 evidence，`challenge` 才能打開更強的 review 或 freeze path

對 reputation 的建議角色：

- `new`
  - 只能以當前 profile 允許的最小 bounded weight 參與
- `established`
  - 可進入此 profile 的一般 bounded score channel
- `trusted`
  - 可使用此 profile 允許的最高 bounded participation，但仍永遠不能與 `view-maintainer` 等權

## 9.3 在弱 anti-Sybil 條件下應維持不可用的 viewer powers

若某個 profile 的 anti-Sybil protection 很弱或幾乎沒有，下列 viewer powers 應維持不可用：

- 任何無上限的正負 selector contribution
- 任何直接 override accepted-head ordering 的能力
- 任何單方面的 `temporary_freeze`
- 任何不要求 evidence 的 challenge path
- 任何把單一低成本 identity 當成 final rejection 充分條件的 challenge path

在弱 anti-Sybil 條件下，最安全的 viewer powers 應是：

- non-selector feedback
- 低權重、偏 advisory 的 `approval`
- 低權重、偏 advisory 的 `objection`
- 只能觸發 review、不能直接定案的 evidence-bearing challenge requests

## 9.4 目前這個 bounded model 的建議 anti-Sybil baseline

對目前這條 bounded viewer-in-selector 方向，比較合適的最低 baseline 是：

- 至少要求 `basic` 的 `viewer_identity_tier`
- 任何 selector-relevant viewer weight 要變成非零之前，必須先取得 `admitted` status
- `new` reputation viewers 只能使用最低 bounded caps
- 要讓 `challenge` 有資格觸發 review，必須附 evidence
- 要讓 `temporary_freeze` 可用，則必須再加更強的 corroboration 或更強 anti-Sybil 條件

這個 baseline 讓 viewer participation 有足夠真實性，不至於只剩裝飾，但仍明確維持它低於 view-maintainer ratification 的地位。

## 10. 建議方向

在這個專案方向裡，較穩的 bounded 版本是：

- 仍以 view-maintainer score channel 作為主 ratification 機制
- 新增 viewer `approval`、`objection`、`challenge`、`flag`
- 讓 `approval` / `objection` 只以 bounded bonus / bounded penalty channels 的方式進入 `selector_score`
- 讓 challenge 主要觸發 `review`
- 只有在更高門檻、且最好搭配更強 anti-Sybil 條件或 maintainer corroboration 時，才允許 `temporary_freeze`

這樣可以讓 viewer 進入 selector path，同時避免把整個治理壓成原始人氣計票。

若再講得更具體一點，建議的 bounded 方向是：

- `approval` 只能提供受 cap 限制的 `bounded_viewer_bonus`
- `objection` 只能提供受 cap 限制的 `bounded_viewer_penalty`
- `challenge` 不應直接加減主 selector score
- `flag` 應留在 score 計算之外，只支援較低嚴重度的 triage
- 若某個 profile 尚不具 anti-Sybil 或 admission gating 條件，viewer selector participation 應預設為 `disabled`
- viewer 的有效權重應由 identity tier、admission status 與 reputation band 推導，而不是由 raw counts 直接決定

## 11. 最小政策欄位形狀

未來 profile 可以定義像這樣的欄位：

- `viewer_selector_mode`
- `viewer_bonus_cap`
- `viewer_penalty_cap`
- `viewer_selector_participation_enabled`
- `viewer_eligibility_mode`
- `viewer_min_identity_tier`
- `viewer_min_reputation_band`
- `viewer_admission_required`
- `viewer_objection_delay_threshold`
- `viewer_challenge_review_threshold`
- `viewer_freeze_threshold`
- `viewer_signal_cost_model`
- `viewer_signal_weight_cap`
- `viewer_challenge_requires_evidence`

這些應維持為 profile-level rules，而不是臨時本地 client settings。

建議語意如下：

- `viewer_selector_mode`：`disabled` 或 `bounded_bonus_penalty`
- `viewer_bonus_cap`：viewer 正向加分的總上限
- `viewer_penalty_cap`：viewer 負向扣分的總上限
- `viewer_selector_participation_enabled`：明確控制某個 profile 是否開啟 viewer score participation
- `viewer_eligibility_mode`：viewer participation 是開放、admitted、reputation-gated，或其他受限模式
- `viewer_min_identity_tier`：viewer signal 要能影響 selector score 前，至少要達到的 identity tier
- `viewer_min_reputation_band`：profile 允許非零 selector weight 前，至少要求的 reputation band
- `viewer_admission_required`：selector-relevant viewer signals 是否必須先經過明確 admission
- `viewer_signal_weight_cap`：單一 viewer 或整體 viewer contribution 在 active profile 下的最大上限
- `viewer_challenge_requires_evidence`：較高影響的 escalation 是否必須帶 evidence 的 challenge signal

### 11.1 範例 `viewer` signal 形狀

既然這條 lane 預設 `viewer` 會直接影響 `selector_score`，最小可行設計就不應只有單一 `like` 計數，而應有一個可驗證、可限權、可分型的 signal 形狀。

建議最少欄位：

- `signal_id`
- `viewer_id`
- `candidate_revision_id`
- `signal_type`
- `reason_code`
- `confidence_level`
- `evidence_ref`
- `created_at`
- `expires_at`
- `signal_status`

其中：

- `signal_type` 至少應區分 `approval`、`objection`、`challenge`
- `reason_code` 讓 signal 可以被機器分類，而不必把整個說明都塞進自由文字
- `confidence_level` 用來區分低成本表態與高承諾表態
- `evidence_ref` 主要供 `challenge` 使用，避免它退化成較重的單純 dislike
- `expires_at` 用來限制過舊 signal 長期黏在 candidate 上
- `signal_status` 用來區分 active、expired、withdrawn、resolved，而不是把每個 challenge 都誤當成 final rejection

建議最小 enum 形狀：

- `signal_type`
  - `approval`
  - `objection`
  - `challenge`
- `confidence_level`
  - `low`
  - `medium`
  - `high`
- `signal_status`
  - `active`
  - `expired`
  - `withdrawn`
  - `resolved`

建議的各型別最低語意：

- `approval`
  - 只能貢獻到 `bounded_viewer_bonus`
  - 可使用 `low` 或 `medium` confidence
  - 不應強制要求 `evidence_ref`
- `objection`
  - 只能貢獻到 `bounded_viewer_penalty`
  - 可使用 `low`、`medium` 或 `high` confidence
  - 可以不附 `evidence_ref`，但仍應帶有 machine-readable 的 `reason_code`
- `challenge`
  - 不應直接改動主 selector score
  - 應要求 `medium` 或 `high` confidence
  - 應要求 `evidence_ref` 或其他可持續引用的 evidence handle
  - 應打開 review-oriented path，而不是自己直接編碼成 final rejection

建議的欄位規則：

- `signal_id` 應在相關 profile 或 application scope 內保持穩定且唯一
- `viewer_id` 用來識別發訊號者，但最終 eligibility 與 weight 仍由 profile 規則決定
- `candidate_revision_id` 讓 signal 綁定單一 candidate head，而不是永久綁在整份文件上
- `created_at` 記錄 signal 何時生效
- `expires_at` 限制它能影響 selector bonus、penalty 或 escalation path 的時間長度
- `signal_status=resolved` 代表 signal 已進入並完成某個 review path；它不代表 candidate 自動被 reject
- `signal_status=withdrawn` 代表 viewer 在 expiry 或 resolution 前主動撤回 signal

建議的 evidence 與 confidence 預期：

- `approval`
  - evidence 可選
  - confidence 主要表示 viewer 願意為這個 support signal 承擔多高程度的表態承諾
- `objection`
  - evidence 可選，但對重複或高 confidence objection 應鼓勵提供
  - confidence 表示這個 objection 在 bounded penalty 規則內應有多強的分量
- `challenge`
  - evidence 應為必填，因為 challenge 是 escalation request，不是比較重的 dislike
  - confidence 表示 challenge claim 的嚴重度與承諾度，但不應繞過 review

這些區分很重要，因為模型必須清楚分開：

- support 與 resistance
- resistance 與 formal review request
- formal review request 與 final rejection

若要安全落地，還需要與 signal 分開但可計算的 eligibility / weighting 欄位：

- `viewer_identity_tier`
- `viewer_reputation_band`
- `eligible_for_selector_bonus`
- `effective_signal_weight`

比較安全的方向是：

- 讓 `approval` 與 `objection` 只進有界 score channel
- 讓 `challenge` 主要影響 `review` / `freeze`，而不是直接大幅改寫主分數
- 讓最終 `effective_signal_weight` 由 profile 規則計算，而不是由 viewer 自報

## 11.2 這條 lane 的 non-goals

這份 bounded viewer-in-selector work 不應被理解成：

- 讓 `viewer` 與 `view-maintainer` 取得對等治理權
- 把 Mycel 變成一人一票式的 public popularity selection
- 允許無上限的 viewer 正負票數直接進 selector input
- 在 signal schema、anti-Sybil gating、trace surfaces 都還沒定清楚前，就強迫 `mycel-core` 立刻改 code
- 就算未來 biometric identity 成熟，也不把它直接視為治理正當性本身

## 12. Viewer 制衡力評估

按照目前這條有界方向，viewer 的制衡力是非對稱的。

它對 `editor-maintainer` overreach 相對有力，原因是：

- viewers 可以延緩 candidate activation
- viewers 可以把高爭議 candidate 升級進入 review
- editors 不能只靠 proposal power 就立即取得 accepted 狀態

但它對 `view-maintainer` 的協同行為仍然弱於對 `editor-maintainer` 的制衡，原因是：

- viewers 雖然進入 `selector_score`，但仍不掌握主導性的 ratification 權重
- viewers 不能單靠自己直接指定 accepted head
- 一旦 review 壓力被解除，形成協調共識的 view-maintainer 多數通常仍保有較大的定案能力

所以目前這條已採納 lane，比較準確的讀法是：

- 對 editors 有較強的程序性制衡
- 對 view maintainers 有中等、但仍屬次級的實質制衡
- 對公眾直接否決權則維持受限

## 13. 補強方案

如果我們希望 viewer 的制衡在有界 score channel 之外再更有力、但又不把系統直接改成人氣治理，最相容的補強方式有 3 種：

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

對這份提案而言，最平衡的 bounded 版本大概會是：

- 讓 viewers 保有有界、可審計的 selector contribution
- 讓 viewer challenge 能強制觸發 mandatory re-review
- 把 freeze 保留給高信任、高證據門檻的案例

## 14. `viewer` 進 / 不進 `selector_score` 的三角色比較

若 `viewer` 不進 `selector_score`：

- `editor-maintainer` 仍主要受 `view-maintainer` ratification 約束，再額外受 viewer 的 `delay` / `review` / `freeze` 約束
- `view-maintainer` 仍握有主裁決權，viewer 比較像程序性制衡者
- `viewer` 擁有煞車與挑戰權，但沒有直接定案權

這種結構的效果是：

- 對 `editor-maintainer` 的制衡較強
- 對 `view-maintainer` 的制衡屬中等，偏程序性
- 對 `viewer` 自身的約束較強，比較不容易讓系統滑向灌票式人氣治理

若 `viewer` 直接進 `selector_score`：

- `editor-maintainer` 不只要說服 `view-maintainer`，也要爭取 viewer score
- `view-maintainer` 會從主治理者，部分降格為與 viewer 共治 accepted-head selection 的角色
- `viewer` 會從煞車者升級成實質治理者

這種結構的效果是：

- 對 `editor-maintainer` 的制衡最強
- 對 `view-maintainer` 的制衡也最強
- 但對 `viewer` 的自我約束最弱，anti-Sybil、identity admission、signal quality control 都會變成更核心的安全邊界

所以三角色一起考慮時：

- `viewer` 不進 `selector_score`，比較像「editor proposal + maintainer ratification + viewer procedural check」
- `viewer` 進 `selector_score`，比較像「editor proposal + maintainer-viewer mixed governance」

既然讓 `viewer` 有界地進入 `selector_score` 就是這條 lane 的專案目標，較穩的路線會是：保留 view-maintainer 的主導 ratification 地位，同時只讓 viewer 以 capped score channel 與高門檻 challenge path 參與。

## 15. 取捨

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

## 16. 這個方向內部仍待收斂的設計問題

這些問題討論的是 `viewer` 進入 `selector_score` 後，bounded participation 應該長成什麼樣子，而不是在重談這條 lane 是否屬於專案方向。

- viewer 的有界 score channel 上限應該多高，才不會壓過 view-maintainer 的主導 ratification？
- viewer approvals 應只影響 tie-break，還是可提供受限的 score bonus？
- `temporary_freeze` 是否應要求 viewer challenge 與 view-maintainer concurrence 同時成立？
- viewer challenge identity 應該是 profile-local、network-global，還是 application-specific？
- 低信任 viewers 是否只能觸發 review，而不能觸發 freeze？
