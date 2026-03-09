# Mycel Blind-address Threat Model

狀態：design draft

這份文件描述在 Mycel deployment 採用 blind-address custody model 時，主要的威脅面、信任邊界與必要控制。

在這份文件中，blind-address model 指的是：

- signer 知道已接受的 `fund_id`、`policy_id`、`signer_set_id` 與簽章條件
- signer 不一定知道對外可見的 settlement address（結算地址）或完整 address mapping（地址映射）
- coordinator 與 executor 角色可能比 signer 知道更多地址層資訊

這份設計的目標不是完美祕密性。

它的目標是在保留明確 consent（同意）、可執行 policy（政策）與可審計 execution（執行）的前提下，降低不必要的地址知情。

## 0. 範圍

這份文件適用於以下部署：

- Mycel 承載 accepted 的治理與 custody state
- m-of-n signer layer 產生簽章或 signature shares（簽章分片）
- coordinator 或 execution layer 負責地址映射與 settlement

這份文件不定義：

- 單一強制的 custody 架構
- 單一強制的 blockchain 或 payment rail
- 完整匿名系統

## 1. 安全目標

blind-address deployment 應盡量同時保住以下目標：

- 降低知道完整 address mapping 的角色數量
- 防止 signer 取得超出必要範圍的地址層資訊
- 在 fund 與 policy scope 上保留 signer consent
- 即使部分角色 metadata 洩漏，也要避免未授權執行
- 保留 dispute resolution（爭議處理）與事後 audit（審計）能力

## 2. 受保護資產

系統至少應把以下內容視為受保護資產：

- signer key shares 或簽章控制權
- `fund_id -> address` mappings
- signer-set membership 與 rotation history
- execution intents 與其 settlement targets
- runtime logs、receipts 與 monitoring output
- 可能間接暴露地址身分的 timing 與行為 metadata

## 3. 角色與知情邊界

### 3.1 Signer

signer 應知道：

- accepted 的 fund 與 policy scope
- 簽章條件
- 系統是否處於 paused、revoked 或 rotated 狀態

signer 不應自動知道：

- 完整 settlement address map
- 所有其他 signer 的身分
- executor 內部的 wallet topology（錢包拓撲）

### 3.2 Coordinator

coordinator 負責組裝一次簽章流程。

coordinator 可能知道：

- 需要哪一個 signer set
- 哪一個 intent 正待處理
- 足以路由請求的 metadata

coordinator 不應被預設授予以下不受限制的存取：

- 原始 signer secrets
- 不必要的長期地址清單

### 3.3 Executor

executor 負責真正執行外部 settlement。

executor 可能知道：

- 真實 address 或 settlement target
- 最終組裝完成的簽章或 settlement authorization

因此 executor 是高價值目標。

### 3.4 Peer 與 Governance Nodes

peer nodes 與 governance state 維護 accepted records。

它們應保留：

- policy history
- signer-set history
- receipts 與 disputes

它們不應複製：

- 原始 custody secrets
- 不必要的、會暴露地址的 runtime internals

## 4. 信任邊界

設計至少應把以下視為不同的信任邊界：

- governance state 與 signer runtime
- signer runtime 與 coordinator
- coordinator 與 executor
- accepted records 與本地 runtime logs
- fund identity 與 settlement address identity

如果這些邊界在操作上全部坍縮，blind-address model 就會急速弱化。

## 5. 主要威脅

### 5.1 Address-mapping 洩漏

如果 `fund_id -> address` mapping 透過 API、logs、dashboard、receipt 或支援流程外洩，blind-address 的特性就大致失效。

常見來源：

- 過度詳細的 runtime logs
- support ticket 或 operator chat
- monitoring labels
- payment processor callbacks
- 同時暴露 fund 與 address reference 的靜態報表

### 5.2 Coordinator 集中風險

如果 coordinator 看到過多 fund、signer 與 intent metadata，它就會變成一個關聯中心。

風險包括：

- 內部濫用
- 被定向入侵
- 法律或營運強制
- 重建跨 fund 關聯圖

### 5.3 Executor 集中風險

executor 可能是唯一知道真實 settlement address 的層。

如果 executor 被攻破，攻擊者可能取得：

- 地址知識
- settlement timing knowledge
- target selection knowledge
- 甚至 execution capability

signer 看不到地址，無法保護 executor 被攻破的情況。

### 5.4 Signer 端反推

即使 signer 從未看到原始 address，仍可能從以下資訊反推出來：

- 重複的交易時間
- 穩定的金額模式
- 固定 allowlists
- 已知 counterparties
- 重複出現的 settlement 行為

所以 blind 並不是絕對，而是機率性的。

### 5.5 弱 consent

如果 blind-address 設計隱藏得太多，signer 可能不再真正理解自己參與的是什麼風險範圍。

這會導致：

- 弱 informed consent（知情同意）
- 治理爭議
- 事故後 accountability（可追責性）不足

signer 應該對不必要的地址細節保持 blindness，而不是對 policy scope 失明。

### 5.6 Metadata 偽造與 phishing

如果 signer 依賴的是摘要化 metadata，而不是強驗證上下文，攻擊者可能偽造：

- intent summaries
- policy references
- signer-set references
- pause 或 revoke state

blind-address system 會提高對可信 metadata 驗證的依賴。

### 5.7 Rotation 失配

如果 signer rotation、policy rotation 與 address rotation 不同步，系統可能漂移到權限不清楚的狀態。

例子：

- signer 以為某個 policy 是 active，但 executor 使用另一個 mapping
- signer set 已輪替，但流程仍導向舊 settlement address
- address 已變更，但治理層沒有對應可見性

### 5.8 Audit 失敗

如果系統隱藏過多地址資訊，卻沒有保留足夠的密封 audit 材料，事後重建就可能失敗。

這會在以下場景形成風險：

- disputes
- incident response
- 外部 compliance review
- 內部 governance review

## 6. 威脅行為者

設計至少應假設以下行為者存在：

- 針對 signer、coordinator 或 executor 的外部攻擊者
- 惡意或粗心的 operators
- 被入侵的 runtime hosts
- 跨層串通的 insiders
- 透過 timing 與 settlement 行為做關聯分析的 observers
- 事後對授權範圍提出爭議的治理參與者

## 7. 必要控制

### 7.1 Mapping 隔離

把 address mapping 與一般 signer 可見狀態隔離。

建議控制：

- 把 mapping 存取限制在最小 executor 邊界
- 預設不把原始 mapping 複製進 operator dashboards
- 預設避免產生帶 address 的 logs

### 7.2 強 signer 驗證

signer 應驗 accepted state，而不是只看人類可讀摘要。

建議控制：

- 驗證 `fund_id`、`policy_id`、`signer_set_id` 與 intent digest
- 驗證 pause、revoke 與 rotation state
- 拒絕未簽名或不可驗證的 coordinator prompts

### 7.3 受限 execution authority

blindness 不能替代 policy 限權。

建議控制：

- 每次 intent 限額
- 每日限額
- allowlists
- cooldowns
- timelocks
- 強制 mismatch receipts

### 7.4 可審計的保密

系統應保留可密封揭露的 audit 材料，而不是直接對所有人公開。

建議控制：

- 保留之後能證明實際使用哪個 mapping 的 receipts
- 把公開 audit records 與受限 forensic records 分開
- 爭議時可揭露足夠證據，但預設不暴露全部 mappings

### 7.5 Rotation 紀律

signer、policy 與 address rotation 應明確且可關聯。

建議控制：

- 發布 rotation records
- 讓過期 mapping 自動失效
- 拒絕對已被取代的 signer-set version 執行交易

### 7.6 角色分離

不要預設 peer、coordinator 與 executor 可以安全地合併。

建議控制：

- 高風險場景下分離部署角色
- 盡量縮小常駐權限
- 檢查每個角色即使拿不到原始地址，也還能反推出多少資訊

## 8. 殘餘限制

blind-address 設計無法保證：

- signer 的完全無知
- 完美匿名
- executor 被攻破時的安全
- 對強 metadata 關聯分析的完全防護

它主要能做的是降低不必要的地址暴露。

它不能取代：

- 狹窄的 policy scope
- signer independence（簽署者獨立性）
- monitoring
- incident response
- 清楚的治理邊界

## 9. 實務判準

只有在以下條件都成立時，blind-address deployment 才算站得住腳：

1. signer 對 policy scope 仍有清楚理解
2. 地址知情被最小化，但不是不可審計
3. coordinator 與 executor 的權力被明確限制
4. 系統能在事後重建爭議執行過程

如果這些條件缺一，這個設計就容易變成不安全，或只是形式上看似私密、實際上不夠可靠。
