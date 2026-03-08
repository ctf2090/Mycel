# Governance History Security

狀態：design draft

這份筆記描述在 Mycel-based system 中，應如何保護 governance history，讓決策能長期保持可驗證、可重放、可重建。

核心原則是：

- governance history 必須可察覺竄改
- governance history 必須可追溯來源
- governance history 必須可重放
- governance history 必須可複製
- governance history 必須在節點故障或遺失後仍可審計

## 0. 目標

在不引入全域強制共識層的前提下，讓治理紀錄可以抵抗惡意修改、本地遺失、部分複製與事後審計需求。

## 1. 安全目標

一個安全的 governance-history 設計應保留五種性質。

1. Integrity：紀錄不能被靜默改寫。
2. Attribution：紀錄可回指到簽署者身分。
3. Ordering：proposal、approval、resolution、receipt 之間的關係可重建。
4. Rebuildability：accepted state 可從儲存的 objects 重新計算。
5. Availability：歷史不能只存在單一機器或單一操作者手上。

## 2. 威脅模型

governance history 至少應假設下列風險：

- 某個節點儲存了損毀或不完整的 objects
- 某個 maintainer key 簽了互相衝突的 records
- 某個本地操作者隱藏 superseded history
- 某個 client 只顯示 current state，卻不保留它為何成為 current
- 某個 runtime 發布的 receipts 與 accepted proposal 不一致
- 某些 replicas 消失或無法使用

## 3. 物件層保護

每個與治理有關的 object 都應在物件層受到保護。

建議規則：

- 使用 canonical serialization
- 在協議要求時導出 content-addressed IDs
- 對需簽章的治理物件強制驗簽
- 只有 canonical ID 檢查通過後才驗簽
- 在 indexing 前先拒絕格式錯誤或不完整的 objects

對 app-layer governance records，即使它們不是 core protocol primitive，實作上也應採取等價的驗證紀律。

## 4. 歷史層保護

安全系統不只要保存 current state，也要保存它如何變成 current。

建議規則：

- proposal、approval、resolution、receipt 應分成不同 records
- 後續 records 應明確引用前面的 records
- superseded 與 rejected records 要保留，不要刪除
- approvals 應保存 signer-set version references
- accepted outcomes 應保留 decision traces

治理鏈範例：

```text
proposal
-> signer approvals
-> accepted resolution
-> execution receipt
-> balance or state update
```

## 5. 採信層保護

client 不應自由地臨時重解 governance history。

建議規則：

- accepted state 只能從 fixed profiles 導出
- 將 signed governance signals 視為 selector inputs
- accepted results 要保留 decision-trace outputs
- 不允許 discretionary local policy 靜默改寫 accepted governance state

這能確保 current governance output 仍然綁在可重算的規則上，而不是本地偏好。

## 6. 複製與保存

如果 governance history 只存在一台機器上，就不算安全。

建議保存策略：

- 在多個彼此獨立的節點間複製 governance documents
- 至少保留一個 archival replica
- 同時保留 object-store copies 與可重建的 indexes
- snapshot-assisted recovery 只能當作最佳化，不能是唯一 truth source

建議角色分工：

- reader nodes 可檢查 current 與 historical governance state
- mirror 或 archivist nodes 保留長期副本
- governance-maintainer nodes 發布 signed decisions

## 7. 重建與審計程序

安全設計應定義固定的 rebuild 與 audit 行為。

最小 rebuild procedure：

1. 載入所有與治理有關的 objects
2. 驗證 signatures 與 references
3. 重放 object 與 revision history
4. 在 fixed profile 下重算 accepted outcomes
5. 比對重算結果與已存的 indexes / receipts

建議 audit outputs：

- missing-object report
- invalid-signature report
- conflicting-approval report
- unresolved-reference report
- accepted-state mismatch report

## 8. Fund 專用的治理歷史

對 fund 或 treasury workflows，系統至少應保留這些 record families：

- fund manifest
- inflow record
- allocation proposal
- signer approval 或 attestation
- accepted allocation resolution
- disbursement receipt
- balance snapshot 或可重放的 ledger state

關鍵要求：

- 鏈上 settlement evidence 必須能回指到授權它的 accepted governance proposal

否則 fund 只有 payment history，卻沒有安全的 governance history。

## 9. 故障與恢復情境

安全的 governance-history 模型應定義常見故障的處理方式。

### 9.1 Replica 遺失

- 從其他 replica 拉回 objects
- 本地重建 indexes
- 驗證 accepted state 未改變

### 9.2 Signer Key 遺失

- 保留先前 approvals 作為歷史事實
- 透過正常治理流程輪替到新的 signer-set version
- 不可重寫舊 signer identity history

### 9.3 衝突的 Approvals

- 保留所有互相衝突的 records
- 在 audit state 中明確標記衝突
- 在 fixed profile 下重算 accepted state

### 9.4 Execution Mismatch

- 若某 receipt 與 accepted proposal 不一致，應把它保留為 mismatch record
- 不可靜默把它合併成 accepted governance state

## 10. 最小第一版規則

對第一個可互通 client，我建議至少做到：

- 驗證所有 governance-object signatures
- 保留 `proposal -> approval -> resolution -> receipt` 連結
- 持久化 decision traces
- 能從 object storage 重建 governance state
- 至少複製到另一個節點
- 提供 accepted-state reasoning 的 audit view

## 11. Open Questions

- app-layer governance records 是否應在所有 apps 中共用一套 signer envelope？
- mismatch 與 conflict records 應成為 first-class app records，還是只當本地 audit artifacts？
- rebuild 與 audit output 有多少應該被複製，有多少只應保留在本地？
