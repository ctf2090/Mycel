# Policy-driven m-of-n Custody

狀態：design draft

這份文件描述一種保管模型：由 Mycel 承載 fund movement（資金移動）的 policy（政策）與 governance history（治理歷史），再由 m-of-n signer network（m-of-n 簽章網路）依固定 policy 約束自動執行交易。

在這份文件中，`m-of-n = members + threshold`。

核心設計原則是：

- Mycel 承載 signer enrollment state（簽署者加入狀態）、signer-set versions（簽署者集合版本）、policy bundles（政策包）、trigger records（觸發紀錄）、execution intent（執行意圖）與 audit history（審計歷史）
- signer pool 成員會明確知道自己已加入，但不需要逐筆手動批准交易
- 交易執行先由已接受的 policy 做事前授權，再由 m-of-n signers 自動執行
- 核心協議維持中立且純技術化

## 0. Goal

在不要求每筆交易都有人類逐筆批准的前提下，提供去中心化保管能力。

留在 Mycel 內的內容：

- signer enrollment 與 revocation（撤銷）狀態
- signer-set 定義與 rotation（輪替）歷史
- policy bundles 與 trigger conditions（觸發條件）
- execution intents 與 receipts（收據）
- audit 與 dispute（爭議）歷史

留在 Mycel 核心之外的內容：

- 原始私鑰材料
- partial-signature assembly（部分簽章組裝）內部細節
- hardware wallet（硬體錢包）或 HSM（硬體安全模組）特定邏輯
- 不可逆的 settlement（結算）副作用

## 1. Model Summary

這個模型不是傳統的人類審閱式 multisig（多重簽章）。

它的特性是：

- policy-authorized（政策授權）
- m-of-n-executed（m-of-n 執行）
- audit-preserving（保留審計鏈）
- signer-aware but not per-transaction interactive（簽署者知道自己參與，但不逐筆互動）

實際規則是：

1. signer 明確加入 signer pool
2. signer 接受固定的 custody policy scope（保管政策範圍）
3. 當某個 accepted trigger（已接受觸發）符合該 policy scope 時，signer node 可自動簽章
4. 一旦達到 threshold，就可廣播交易

## 2. Four Layers

### 2.1 Client Layer

client 是面向使用者的層。

職責：

- 顯示 fund policy state（資金池政策狀態）
- 顯示 signer-pool membership（簽署者池成員）與 signer-set versions
- 顯示 execution intents、receipts、pauses、revocations 與 disputes
- 解釋某次執行為什麼被允許或被阻擋

非職責：

- 預設不持有可複製的私鑰
- 不重新定義 accepted custody policy
- 不繞過 accepted trigger state

### 2.2 Governance Layer

governance layer 由 Mycel 承載。

職責：

- 定義 signer-set 成員
- 定義 policy bundles
- 定義 execution eligibility rules（執行資格規則）
- 定義 pause、revoke 與 rotation records
- 保留完整 audit trail（審計軌跡）

非職責：

- 不直接產生原始簽章
- 不把保管秘密作為可複製狀態保存

### 2.3 Threshold Signer Layer

threshold signer layer 是 signer network 或 signer runtime。

職責：

- 驗證 accepted policy state
- 驗證 trigger bundles
- 驗證 amount caps（金額上限）、cooldowns（冷卻）、allowlists（允許清單）與 signer-set version
- 在滿足 policy 條件時產生 partial signatures

非職責：

- 不在本地重新解讀 governance rules
- 不在 accepted policy scope 之外簽章
- 當系統處於 paused 或 revoked 狀態時，不可靜默繼續執行

### 2.4 Execution Layer

execution layer 負責組裝 threshold signatures 並執行結算。

職責：

- 組裝有效 partial signatures
- 廣播交易或提交至 settlement rail（結算通道）
- 發布 execution receipts
- 顯示 mismatch（不匹配）或 failure（失敗）紀錄

非職責：

- 不捏造 approval state（批准狀態）
- 不對未通過 policy 驗證的交易做結算

## 3. Signer Enrollment and Consent

即使之後執行是自動的，這個模型仍然要求明確的 enrollment（加入）。

建議規則：

- signer MUST 知道自己的 key 或 key share 屬於 signer pool
- signer MUST 知道哪些 policy scope 下可能發生自動簽章
- signer MUST 能夠 pause、revoke 或 rotate 自己的參與
- signer SHOULD NOT 被要求逐筆手動批准交易

這裡要分清楚：

- enrollment consent（加入同意）
- 與 per-transaction manual approval（逐筆手動批准）

signer 同意的是 policy envelope（政策邊界），不是未來每一筆執行事件本身。

## 4. Core Custody Objects

### 4.1 Signer Enrollment Record

定義某個 signer 已加入 custody system（保管系統）。

建議欄位：

- `enrollment_id`
- `signer_id`
- `signer_key_ref`
- `role`
- `status`
- `joined_at`
- `policy_scope_ref`

典型 `status` 值：

- `active`
- `paused`
- `revoked`
- `retired`

### 4.2 Signer Set

定義一個 m-of-n signer group（m-of-n 簽署群組）。

建議欄位：

- `signer_set_id`
- `fund_id`
- `members`
- `threshold`
- `version`
- `status`
- `created_at`

### 4.3 Policy Bundle

定義自動執行允許做什麼。

建議欄位：

- `policy_id`
- `fund_id`
- `signer_set_id`
- `allowed_trigger_types`
- `max_amount_per_execution`
- `max_amount_per_day`
- `cooldown_seconds`
- `destination_allowlist_ref`
- `asset_scope`
- `pause_state`
- `effective_from`
- `effective_until`

### 4.4 Trigger Record

代表一個可能啟用執行的 accepted trigger。

建議欄位：

- `trigger_id`
- `trigger_type`
- `trigger_ref`
- `fund_id`
- `policy_id`
- `amount_requested`
- `asset`
- `created_at`

### 4.5 Execution Intent

代表從 accepted trigger 導出的具體 spend attempt（支出嘗試）。

建議欄位：

- `intent_id`
- `fund_id`
- `policy_id`
- `signer_set_id`
- `trigger_id`
- `outputs`
- `total_amount`
- `intent_hash`
- `status`
- `created_at`

典型 `status` 值：

- `pending`
- `eligible`
- `blocked`
- `signed`
- `broadcast`
- `failed`

### 4.6 Signer Attestation

代表某個 signer 端確認 policy 檢查通過，並已產生 signature share（簽章分片）。

建議欄位：

- `attestation_id`
- `intent_id`
- `signer_id`
- `signer_set_version`
- `intent_hash`
- `outcome`
- `created_at`

典型 `outcome` 值：

- `signed`
- `rejected`
- `skipped-paused`
- `skipped-revoked`
- `skipped-policy-mismatch`

### 4.7 Execution Receipt

代表最終結算結果。

建議欄位：

- `receipt_id`
- `intent_id`
- `executor`
- `settlement_ref`
- `status`
- `submitted_at`
- `confirmed_at`
- `error_summary`

## 5. Automatic Approval Flow

建議流程：

1. 某個 fund 存在一份 accepted policy bundle
2. 建立一筆 accepted trigger record
3. 系統導出一筆 execution intent
4. signer nodes 驗證：
   - signer enrollment state
   - signer-set version
   - policy bundle validity（有效性）
   - pause 與 revoke 狀態
   - amount、rate 與 destination constraints（目標限制）
5. 合格的 signer nodes 自動產生 partial signatures
6. 一旦達到 threshold，executor 廣播交易
7. 系統寫入 execution receipt

這會把批准邊界放在 policy 被接受的時點，而不是交易當下的點擊動作。

## 6. Committee Selection Options

有兩種實際可行的 committee（委員會）模型。

### 6.1 Fixed Signer Set

一個 signer set 中所有 active 成員都可參與。

取捨：最容易實作，但 signer 暴露面較固定。

### 6.2 Large Pool with Derived Signing Committee

先有較大的 signer pool，再為每個 intent 或 epoch 導出較小的 signing committee。

可用的導出輸入：

- `intent_hash`
- `epoch_id`
- `signer_set_version`
- deterministic random beacon（決定性隨機信標）或 VRF output

取捨：更去中心化、較難預測，但複雜度明顯更高。

第一版實作我建議先採 fixed signer-set model。

## 7. Guardrails

自動門檻保管需要硬限制。

最低建議 guardrails：

1. 每個 signer MUST 明確 enrollment
2. 每次執行 MUST 對應一份 accepted policy bundle
3. 每份 policy bundle MUST 有金額與頻率限制
4. 系統 MUST 支援 `pause`
5. 系統 MUST 支援 `revoke`
6. 系統 MUST 支援 signer-set rotation
7. 系統 MUST 保留 failed 與 blocked intents 作為 audit records
8. 本地 runtime policy MUST NOT 靜默擴大 accepted policy scope

## 8. Pause, Revoke, and Rotation

這個 custody model 應定義三種操作控制。

### 8.1 Pause

暫時停止自動簽章，但不移除 signer 或 policy。

### 8.2 Revoke

把某個 signer、policy bundle 或 trigger class 從未來的執行資格中移除。

### 8.3 Rotation

建立新的 signer-set version，並把未來執行遷移到該版本。

rotation 應保留：

- 舊 signer identity history（簽署者身分歷史）
- 舊 intent 與 receipt references
- 舊 policy applicability windows（適用時間窗）

## 9. Failure Cases

### 9.1 Intent-policy mismatch

- 不簽章
- 保留 blocked intent record

### 9.2 Threshold not reached

- 保留已收集的 signer attestations
- 把 intent 標記為 incomplete 或 expired

### 9.3 Paused signer pool

- 產生明確的 `skipped-paused` 結果
- 不可靜默忽略 pause state

### 9.4 Rotated signer set

- 新 intents MUST 綁定新 signer-set version
- 舊 intents 保留舊 signer-set reference

## 10. Minimal First-client Rules

對第一個可互通 client，我建議至少做到：

- 顯示每個 fund 的 active policy bundle
- 顯示 active signer-set version
- 顯示 automatic signing 是否為 enabled、paused 或 revoked
- 保留 trigger -> intent -> attestation -> receipt 連結
- 當 accepted policy state 不完整時拒絕執行
- 保持 signer enrollment 與 signer-set history 可見

## 11. Open Questions

- automatic signing 應該維持 fund-specific，還是允許同一 signer 被多個 funds 重用？
- committee derivation 應先採 fixed-set，還是早期就加入 VRF-based committee selection？
- 哪些 app-layer record families 應強制在 reader nodes 與 signer nodes 之間複製？
