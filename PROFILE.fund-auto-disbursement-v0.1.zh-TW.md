# Fund Auto-disbursement Profile v0.1

狀態：profile draft

這份 profile 定義一個收斂且可互通的 automatic fund disbursement（資金池自動撥款）模型。它建立在 Mycel app-layer records、accepted-state selection，以及 policy-driven threshold custody 之上。

這份 profile 刻意採取保守設計。

它限制：

- accepted trigger 如何變成 disbursement candidate（撥款候選）
- policy checks（政策檢查）如何套用
- automatic threshold signing（自動門檻簽章）如何進行
- 哪些 records 必須存在，才能支撐 audit 與 rebuild

它不重新定義核心協議。

## 0. Scope

這份 profile 假設底層實作已支援：

- Mycel core protocol
- accepted-head selection
- app-layer records
- policy-driven threshold custody
- signer enrollment 與 consent tracking

這份 profile 適用於：

- 一個 `fund_id`
- 每個 execution intent 僅綁定一個 active signer-set version
- 每條 execution path 僅對應一個 accepted policy bundle
- 一次只處理一個具體的 disbursement intent

## 1. Profile Goals

目標如下：

1. 讓 automatic disbursement 可預期
2. 保持批准邊界清楚
3. 保留可 rebuild 的 governance history
4. 把第一版 client 收斂到安全可做的範圍

## 2. Required Record Families

合規實作至少必須保存以下 record families：

- `fund_manifest`
- `signer_enrollment`
- `signer_set`
- `policy_bundle`
- `consent_scope`
- `trigger_record`
- `execution_intent`
- `signer_attestation`
- `execution_receipt`
- `pause_or_revoke_record`

可以有額外 records，但不可用來取代這些最低需求 records。

## 3. Accepted Trigger Sources

這份 profile 只允許從 accepted trigger record 開始一條撥款路徑。

允許的 trigger classes：

- `allocation-approved`
- `sensor-qualified`
- `pledge-matured`

部署可以支援更少的 trigger class，但在這個 profile 版本中不可再增加更多類別。

每筆 `trigger_record` 必須包含：

- `trigger_id`
- `trigger_type`
- `trigger_ref`
- `fund_id`
- `policy_id`
- `amount_requested`
- `asset`
- `created_at`

## 4. Policy Constraints

每次撥款嘗試都必須綁定一份 accepted `policy_bundle`。

active policy bundle 必須定義：

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

若缺少任何必要 policy 欄位，合規實作必須拒絕執行。

## 5. Execution Eligibility Rules

只有在以下條件全部成立時，execution intent 才算 eligible：

1. trigger record 已在 active profile 下被接受
2. trigger type 在 active policy bundle 的允許範圍內
3. requested amount 不超過 `max_amount_per_execution`
4. requested amount 不會讓 fund 超過 `max_amount_per_day`
5. cooldown window 已經過去
6. destination 在 active allowlist 內
7. active signer-set version 與 policy bundle 相符
8. signer set 並未 paused 或 revoked
9. fund 有足夠 available balance

若任何規則失敗，系統必須產生 blocked 或 rejected execution outcome，不可靜默繼續。

## 6. Execution Intent

每條 eligible 的撥款路徑都必須產生一筆 `execution_intent`。

必要欄位：

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

這份 profile 允許的 `status` 值：

- `pending`
- `eligible`
- `blocked`
- `signed`
- `broadcast`
- `failed`

`intent_hash` 必須能穩定對應到當次要簽出的 outputs 與 amount。

## 7. Automatic Threshold Signing

這份 profile 只在以下規則下允許 automatic signing：

1. 所有參與 signer 都必須有 active enrollment
2. 所有參與 signer 都必須有有效的 consent scope
3. 每個 signer runtime 都必須驗證同一個 `intent_hash`
4. 每個 signer runtime 都必須驗證同一個 `policy_id`
5. 每個 signer runtime 都必須把結果綁到同一個 `signer_set_id` 與 version

合規 signer runtime 絕不可在以下情況簽章：

- 缺少 enrollment
- 缺少 consent scope 或 consent 已過期
- state 是 paused 或 revoked
- policy 欄位不完整
- 本地 runtime 狀態與 accepted state 不同步

## 8. Signer Attestations

每個 signer-side result 都必須保存為一筆 `signer_attestation`。

必要欄位：

- `attestation_id`
- `intent_id`
- `signer_id`
- `signer_set_version`
- `intent_hash`
- `outcome`
- `created_at`

這份 profile 允許的 `outcome` 值：

- `signed`
- `rejected`
- `skipped-paused`
- `skipped-revoked`
- `skipped-policy-mismatch`
- `skipped-insufficient-sync`

實作必須同時保留成功與失敗的結果。

## 9. Threshold Rule

這份 profile 假設每個 active signer-set version 只有一個固定 threshold。

必要規則：

- `required_signatures = threshold(signer_set_id, version)`

只有在同一個 `intent_hash` 收到至少 `required_signatures` 個有效結果之後，execution layer 才能廣播。

## 10. Receipt Requirements

每次 broadcast 或 settlement 嘗試都必須產生一筆 `execution_receipt`。

必要欄位：

- `receipt_id`
- `intent_id`
- `executor`
- `settlement_ref`
- `status`
- `submitted_at`
- `confirmed_at` 或 `null`
- `error_summary`

這份 profile 允許的 `status` 值：

- `submitted`
- `confirmed`
- `failed`
- `rejected-by-rail`

receipt 必須可回連到：

- 一筆 `execution_intent`
- 一筆 `trigger_record`
- 一份 `policy_bundle`
- 一個 signer-set version

## 11. Pause, Revoke, and Rotation

這份 profile 要求支援：

- signer pause
- signer revoke
- signer-set rotation
- policy pause

必要行為：

- 新 execution intents 只能綁定 current active signer-set version
- 舊 intents 保留舊 signer-set reference
- pause 或 revoke 只阻擋未來簽章，不重寫舊歷史

## 12. Minimal Flow

最小合規流程如下：

1. 出現 accepted trigger record
2. 實作檢查 active policy bundle
3. 實作檢查 balance 與 rate limits
4. 實作建立 `execution_intent`
5. signer runtimes 驗證資格並發出 `signer_attestation`
6. execution layer 達到 threshold 並廣播
7. 實作寫入 `execution_receipt`

## 13. Non-goals

這份 profile 不定義：

- raw payment processor integration
- raw sensor interpretation
- oracle trust models
- cross-fund aggregation
- dynamic weighted signer math
- 超出單一 active signer set 的 committee derivation

## 14. Minimal First-client Requirements

對第一個可互通 client，我建議：

- 一次只支援一個 active `fund_id`
- 一次只支援一個 active signer-set version
- 一次只支援一個 active policy bundle
- 不做 dynamic committee derivation
- 不做 parallel partial-intent merging
- 明確顯示 blocked-intent 與 failed-receipt 檢視

## 15. Open Questions

- 後續版本是否應允許每個 fund 同時存在多個 active policy bundles？
- 後續版本是否應允許 weighted signer math，而不是固定 threshold？
- `allocation-approved` 與 `sensor-qualified` 應繼續共用同一個 profile，還是未來拆成更窄的兩個 profiles？
