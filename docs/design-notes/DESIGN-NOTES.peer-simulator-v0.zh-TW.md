# Peer Simulator v0

Status: design draft

本文件定義一個狹窄的多 peer simulator，供 Mycel 早期實作測試使用。

主要設計原則是：

- 在建立完整客戶端之前，先證明核心同步與驗證行為
- 讓 simulator 保持決定性且容易重置
- 優先採用有界的本地拓樸，而不是廣域 discovery
- 把 wire 行為測試與豐富產品功能分開

## 0. 目標

提供一個實際可用的第一版測試框架，能夠：

- 在本地執行多個 Mycel peer 身分
- 交換最小 v0.1 wire messages
- 驗證 canonical objects 並重播 state
- 提早暴露 sync 與 acceptance 問題

這個 simulator 是給實作測試用的。

它不是正式部署模型。

## 1. 建議形式

我建議分成兩個階段。

### 1.1 Phase A: 單程序多 peer simulator

在同一個程序內執行多個 peer state。

優點：

- 最快可以做出來
- 排程具決定性
- 容易注入 fixture
- 容易收集 trace

限制：

- 無法測真實 socket 行為
- 無法測程序層級隔離
- 無法真實測試 restart 行為

### 1.2 Phase B: Localhost 多程序 peer harness

在 `127.0.0.1` 上執行多個相同 node 實作。

優點：

- 能測到真實 session setup 與 transport framing
- 能測到各 peer 的儲存隔離
- 能測到 bootstrap 設定與 restart 行為

限制：

- 比 Phase A 慢
- 需要更多 harness code 與程序控制

建議路徑是：

1. 先做 Phase A
2. 把同一套 peer logic 移到 Phase B
3. 如果可以，兩種 harness 都保留

## 2. 納入範圍的能力

這個 simulator 應只涵蓋最狹窄的互通路徑。

必要：

- 多個 peer identities
- 每個 peer 一個本地 object store
- 具決定性的 fixture 載入
- `HELLO`
- `MANIFEST`
- `HEADS`
- `WANT`
- `OBJECT`
- `BYE`
- `ERROR`
- object ID 驗證
- object signature 驗證
- replay-based `state_hash` 驗證

Simulator v0 可選：

- `SNAPSHOT_OFFER`
- `VIEW_ANNOUNCE`
- peer 之間 accepted-head 比對

延後：

- rich reader UI
- editor 工作流程
- public discovery
- Tor transport
- signer 或 runtime roles

## 3. Peer 角色

第一版 simulator 應只用非常小的角色集合。

### 3.1 Seed Peer

seed peer 一開始持有已知的 fixture objects。

職責：

- 提供初始 `MANIFEST` 與 `HEADS`
- 回應 `WANT` 請求
- 在測試拓樸中維持最穩定的 peer

### 3.2 Reader Peer

reader peer 一開始是空的，或只有部分資料。

職責：

- 從有界 peer list 啟動
- 同步缺失 objects
- 在 indexing 之前先驗證所有收到的 objects
- 若啟用該層，則計算本地 accepted state

### 3.3 Fault Peer

fault peer 是可選的，但很有用。

職責：

- 傳送 malformed 或不一致的 messages
- 宣告自己有其實不存在的 objects
- 傳送錯誤 hash、錯誤 signature 或重複 announcements

這個角色很適合做負面測試，而不用污染正常的 seed peer。

## 4. 最小拓樸

simulator 一開始只需要支援三種拓樸。

### 4.1 兩個 peer 的同步

- `peer-seed`
- `peer-reader-a`

用來做第一個端對端 sync 測試。

### 4.2 三個 peer 的一致性

- `peer-seed`
- `peer-reader-a`
- `peer-reader-b`

用來比較：

- 收到的 object sets
- replay 結果
- accepted-head 結果

### 4.3 含 fault injection 的拓樸

- `peer-seed`
- `peer-reader-a`
- `peer-fault`

用來測 rejection 與 recovery 行為。

## 5. 每個 peer 的狀態

每個 simulated peer 都應有隔離狀態。

最小狀態：

- `node_id`
- transport endpoint 或 logical bus address
- peer keypair
- peer capabilities
- bootstrap peer list
- object store
- derived indexes
- sync session history
- local transport policy

建議再加：

- event log
- decision trace log
- fault-injection flags

即使在單程序 simulator 中，這些狀態也必須邏輯上保持分離。

## 6. Transport 模型

transport layer 應可替換。

### 6.1 Phase A Transport

使用 in-memory message bus。

要求：

- 保留 sender 與 receiver identity
- 保留單一 session 內的 message order
- 支援具決定性的 delay 或 drop injection
- 能完整捕捉 wire envelopes 供檢查

### 6.2 Phase B Transport

使用真實 localhost sockets。

要求：

- 每個 peer 一個 listen address
- 可設定 bootstrap peers
- session lifecycle events
- 支援乾淨 shutdown 與 restart

peer logic 不應依賴 transport 是 in-memory 還是 socket-based。

## 7. Fixture 策略

simulator 一開始應使用明確 fixtures，而不是臨時生成的內容。

建議 fixture sets：

1. 一份合法文件，只有一條 revision chain
2. 一份文件，含兩個合法 heads
3. 一組含 hash mismatch 的非法 objects
4. 一組含 signature mismatch 的非法 objects
5. 一個需要靠 `WANT` 補抓的 partial store

fixtures 應該：

- 具決定性
- 受版本控制
- 能載入到任何 peer role

## 8. 最小流程

baseline simulator flow 應該是：

1. 初始化 peer identities 與 stores
2. 把 fixture objects 載入 `peer-seed`
3. 啟動 peer sessions
4. 交換 `HELLO`
5. 交換 `MANIFEST` 或 `HEADS`
6. 計算缺少的 canonical object IDs
7. 用 `WANT` 請求它們
8. 用 `OBJECT` 回傳它們
9. 驗證 object ID、hash、signature 與 replayed state
10. 只索引已驗證通過的 objects
11. 視需要計算 accepted head
12. 輸出 session result report

## 9. 測試案例

Simulator v0 至少應支援以下測試。

### 9.1 正向案例

- 從空的 reader 做第一次 sync
- 新的 heads 出現後做 incremental sync
- 只靠 stored objects 做 replay rebuild
- 兩個 readers 從同一組 verified objects 算出相同 accepted result

### 9.2 負向案例

- 拒絕 derived ID 不符的 `OBJECT`
- 拒絕 body hash 不符的 `OBJECT`
- 拒絕 signature 無效的 object
- 拒絕 wire envelope signature 無效的 message
- 拒絕無效 parent ordering 或無效 replay 結果

### 9.3 Recovery 案例

- partial object delivery 後重試
- peer restart 後重新連線
- 忽略 faulty peer 之後繼續 sync

## 10. Harness 輸出

simulator 應輸出容易 diff 的結果。

建議輸出：

- 每個 peer 收到的 object IDs
- 每個 peer 的 verification results
- 每個 peer 依 `doc_id` 的最終 heads
- 啟用時的 accepted-head 結果
- wire trace log
- failure summary

這些輸出應為 machine-readable。

之上可以再加 human-readable summaries。

## 11. 非目標

Simulator v0 應明確避免：

- 宣稱已證明完整 protocol completeness
- 模擬 public-mesh 規模
- 建立最終 reader UI
- 建模 fund execution flows
- 建模 signer consent flows
- 建模 anonymous deployment behavior
- 把 peer-discovery 草案當成強制 runtime 行為

這個 simulator 是 build aid，不是整個平台。

## 12. 建議的 Repository 形狀

一個實際可行的 repo 形狀可以是：

- `fixtures/`
- `sim/`
- `sim/peers/`
- `sim/topologies/`
- `sim/tests/`
- `sim/reports/`

建議的 simulator 頂層元件：

- peer state model
- transport adapter
- fixture loader
- wire session driver
- verification engine wrapper
- report generator

## 13. 成功條件

若我們能做到以下六點，Peer Simulator v0 就算成功：

1. 在本地啟動至少三個隔離的 peer identities
2. 將具決定性的 fixtures 載入一個 peer
3. 將這些 fixtures 同步到一個或多個其他 peers
4. 正確拒絕 malformed 或不一致的 objects
5. 只靠 canonical objects 重建本地狀態
6. 比對各 peer 輸出，並在預期情況下確認決定性一致

如果這六點都成立，這個 simulator 就已經有用。

## 14. 建議下一步

當 simulator v0 可用後，下一個擴充應只選以下其中一項：

- 把 accepted-head comparison 變成一級 report
- 加入 localhost 多程序模式
- 加入 snapshot-assisted catch-up tests

不要三項一起擴。
