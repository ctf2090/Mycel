# Auto-signer Consent Model

狀態：design draft

這份文件描述在 Mycel 系統中，automatic threshold signers（自動門檻簽署者）的 consent（同意）應如何建模。

核心設計原則是：

- signer 先對 enrollment（加入）與 policy scope（政策範圍）做事前同意
- signer 不需要逐筆手動批准後續交易
- 自動簽章只有在 accepted 且受邊界約束的政策條件內才有效
- consent 邊界必須保持可見、可撤回、可審計

## 0. Goal

在允許自動門檻簽章的同時，不把「沒有逐筆人工批准」誤當成「沒有人的同意」。

這個模型會明確分開：

- enrollment consent（加入同意）
- policy-scope consent（政策範圍同意）
- operational state（操作狀態）
- execution events（執行事件）

它不把後續交易執行當作無上限的隱性授權。

## 1. Consent Boundary

signer 的同意發生在加入 signer pool 並接受某個明確 policy scope 的時點。

signer 不需要：

- 在每筆交易簽出前先看到該筆交易
- 每筆交易都點一次 approve
- 即時知道某次執行是否用了自己的 share

但 signer 仍然必須知道：

- 自己已經被 enrollment
- 自己屬於哪個 fund 或 signer pool
- 哪些 trigger classes（觸發類型）可能啟動自動簽章
- 目前有哪些 amount、rate 與 destination constraints（目標限制）
- 要如何 pause、revoke 或 rotate 自己的參與

## 2. Core Definitions

### 2.1 Enrollment Consent

Enrollment consent 表示 signer 明確知道自己加入了 signer pool。

最低語義：

- 「我的 key 或 key share 屬於這個 custody system」
- 「當 policy 條件滿足時，我的 signer node 可能自動簽章」

### 2.2 Policy-scope Consent

Policy-scope consent 表示 signer 同意某個受邊界限制的執行範圍。

最低邊界：

- allowed trigger types
- maximum amount per execution
- maximum amount per time window
- allowed assets
- allowed destination classes 或 allowlists
- effective start and end time

### 2.3 Operational State

Operational state 表示目前是否允許自動簽章。

典型值：

- `active`
- `paused`
- `revoked`
- `expired`
- `rotating`

### 2.4 Execution Event

Execution event 是一個之後發生的 spend attempt（支出嘗試），系統會拿它去比對 accepted policy scope。

Execution event 本身不是新的 consent grant（同意授予）。

## 3. What Counts as Valid Consent

有效同意至少要同時滿足以下條件：

1. signer 被明確 enrollment
2. signer 可取得適用的 policy scope
3. 在評估當下，signer 不是 paused、revoked 或 expired
4. 執行事件符合 accepted policy scope
5. 系統保留了 signer 狀態與執行結果之間的 audit trail

若任何一項不成立，自動簽章就應視為無效或超出政策範圍。

## 4. What Does Not Count as Consent

以下情況不應被視為有效同意：

- 靜默地把某人加入 signer pool
- 從一般 app 使用行為推定 consent
- 沒有邊界的無上限自動簽章
- 在 accepted 的 fund 或 policy scope 之外重用某 signer
- 在 revoke、expiry 或 explicit pause 之後仍然繼續簽章
- 本地 runtime 靜默擴大政策範圍

## 5. Consent Lifecycle

### 5.1 Join

signer 被加入一個 signer pool。

必要輸出：

- signer enrollment record
- signer key reference
- signer pool 或 fund reference
- initial operational state

### 5.2 Activate

signer 在某個 accepted policy scope 下變成可自動簽章。

必要輸出：

- effective policy reference
- effective time window
- operational state 設成 `active`

### 5.3 Pause

signer 仍然保留 enrollment，但暫時停用自動簽章。

預期行為：

- 不再產生新的簽章
- blocked 或 skipped 結果仍可審計

### 5.4 Revoke

signer 被移出未來的執行資格。

預期行為：

- 新 intent 不可再使用該 signer 作為 active authority
- 舊歷史仍然保留

### 5.5 Rotate

signer 轉移到新的 key、新的 signer-set version，或新的 policy scope。

預期行為：

- 舊新 identity link 仍然可追溯
- 未來執行只綁定新的有效版本

## 6. Recommended Records

### 6.1 Signer Enrollment Record

建議欄位：

- `enrollment_id`
- `signer_id`
- `signer_key_ref`
- `fund_id`
- `signer_set_id`
- `status`
- `joined_at`

### 6.2 Consent Scope Record

建議欄位：

- `consent_scope_id`
- `enrollment_id`
- `policy_id`
- `max_amount_per_execution`
- `max_amount_per_day`
- `allowed_trigger_types`
- `allowed_assets`
- `destination_allowlist_ref`
- `effective_from`
- `effective_until`

### 6.3 Consent State Record

建議欄位：

- `state_id`
- `enrollment_id`
- `state`
- `reason`
- `created_at`

### 6.4 Consent Evidence Record

建議欄位：

- `evidence_id`
- `enrollment_id`
- `consent_scope_id`
- `accepted_at`
- `source_ref`

這部分可以依 deployment（部署）而定，但系統應至少保留某種證據，證明 signer 確實知道並接受了 enrollment 與 scope。

### 6.5 Auto-sign Outcome Record

建議欄位：

- `outcome_id`
- `intent_id`
- `signer_id`
- `consent_scope_id`
- `result`
- `reason`
- `created_at`

典型 `result` 值：

- `signed`
- `blocked-paused`
- `blocked-revoked`
- `blocked-expired`
- `blocked-policy-mismatch`

## 7. Client Responsibilities

合規 client 應讓 signer 看得到：

- signer enrollment status
- current consent scope
- current operational state
- recent automatic-sign outcomes
- pause 與 revoke controls
- pending rotations

合規 client 不應呈現：

- 隱藏的自動簽章
- 模糊或無上限的 policy scope
- 「自動化就不需要 consent」的錯誤印象

## 8. Runtime Responsibilities

signer runtime 應該：

- 若缺少 enrollment，就拒絕簽章
- 若缺少 consent scope 或 scope 已過期，就拒絕簽章
- 若 state 是 paused 或 revoked，就拒絕簽章
- 為被阻擋的簽章記錄明確原因
- 把每次簽章結果綁到一個 signer-set version 與一個 policy scope

signer runtime 不應：

- 在本地擴大 policy
- 靜默忽略 consent-state changes（同意狀態變更）
- 在 accepted state 同步失敗後仍繼續簽章

## 9. Failure Cases

### 9.1 Signer never knowingly enrolled

對未來操作審查來說，該 identity 的所有自動簽章都應視為無效。

### 9.2 Scope mismatch

不簽章，並記錄一筆 policy mismatch outcome。

### 9.3 Pause not propagated

不可靜默繼續簽章，應將此事件標記為 state-synchronization failure（狀態同步失敗）。

### 9.4 Rotated signer still signing under old scope

阻擋舊有效狀態下的後續簽章，並保留 mismatch trail（不匹配軌跡）。

## 10. Minimal First-client Rules

對第一個可互通 client，我建議至少做到：

- explicit signer enrollment UI
- explicit policy-scope display
- 可見的 `active / paused / revoked / expired` 狀態
- 可見的 auto-sign outcomes 歷史
- 明確的 revoke 與 pause controls
- 不允許 hidden 或 implicit enrollment path

## 11. Open Questions

- 在最小部署中，consent evidence 應該做到多強？
- pause 與 revoke 應只允許 signer-local、只允許 governance-driven，還是兩者都支援？
- 對同一個 fund，是否允許同一 signer 同時持有多個 consent scopes？
