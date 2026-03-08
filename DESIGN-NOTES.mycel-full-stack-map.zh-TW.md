# Mycel Full-stack Map

狀態：design draft

這份文件把目前的 Mycel 文件集整理成一張 full-stack system view（全棧系統視圖）。

核心設計原則是：

- Mycel 應被理解為分層系統，而不是單一文件格式
- protocol core 只是最終整體中的一層
- profiles、app-layer models、governance 與 deployment 都各自承擔不同責任
- 實作規劃應依這些層次來切，而不是把所有文件都視為同一個立即範圍

## 0. Goal

給 repo 一張系統層級的地圖，用來回答：

- Mycel 目前實際暗示了哪些主要層次
- 哪些文件屬於哪一層
- 每一層承擔什麼責任
- 如果把目前所有層都實做出來，Mycel 會變成什麼級別的專案

## 1. Stack Overview

目前的 Mycel 文件集至少暗示了七個主要層次：

1. protocol core
2. object verification 與 local state
3. synchronization 與 transport
4. governance 與 accepted-state selection
5. canonical text 與 application models
6. fund、custody 與 execution systems
7. deployment、privacy 與 operational layers

這些層在分析上可以分開，但在實作上會高度互動。

## 2. Protocol Core

這是最窄、也最強調互通性的層。

責任：

- 定義 core object families
- 定義 canonical serialization
- 定義 hashing 與 derived IDs
- 定義 signature expectations
- 定義 revision replay
- 定義 wire message structure

主要文件：

- `PROTOCOL.*`
- `WIRE-PROTOCOL.*`

若這一層不穩，其他層就無法可靠互通。

## 3. Object Verification and Local State

這一層把 canonical objects 轉成本地可運作的節點狀態。

責任：

- object parsing
- derived-ID verification
- signature verification
- revision replay
- state reconstruction
- index rebuild
- local object store management

主要文件：

- `IMPLEMENTATION-CHECKLIST.*`
- `PROTOCOL.*` 的部分內容

這是第一個真正落地成 client 或 node 的實作層。

## 4. Synchronization and Transport

這一層讓 peers 可以交換 objects 與目前狀態。

責任：

- session setup
- manifest 與 heads 交換
- object fetch 與 validation
- transport constraints
- bounded peer communication

主要文件：

- `WIRE-PROTOCOL.*`
- `PROFILE.mycel-over-tor-v0.1.*`
- `DESIGN-NOTES.peer-discovery-model.*`

這一層代表 Mycel 不再只是文本模型，而是實際的網路系統。

## 5. Governance and Accepted-state Selection

這一層決定 conforming reader 或 deployment 應把哪個狀態視為 active。

責任：

- 儲存 governance signals
- 定義 accepted-head 或 accepted-reading selection
- 區分不同 publication roles
- 保存 governance history
- 讓 decision traces 保持可審計

主要文件：

- `PROTOCOL.*`
- `DESIGN-NOTES.client-non-discretionary-multi-view.*`
- `DESIGN-NOTES.two-maintainer-role.*`
- `DESIGN-NOTES.governance-history-security.*`

這一層是 Mycel 與一般 replicated document system 的重要差異。

## 6. Canonical Text Layer

這一層負責長期 reference corpora 的建模。

責任：

- stable citation anchors
- witness handling
- accepted reading profiles
- commentary separation
- witnesses 之間的 alignment

主要文件：

- `DESIGN-NOTES.canonical-text-profile.*`

如果 Mycel 要用來承載高度引用的長文本，而不只是短文協作，這層就很重要。

## 7. General App Layer

這一層讓 Mycel 可以承載 app definitions，而不把 protocol core 直接變成副作用執行器。

責任：

- app manifest modeling
- app state modeling
- intent modeling
- effect request 與 receipt modeling
- runtime separation

主要文件：

- `DESIGN-NOTES.mycel-app-layer.*`
- `DESIGN-NOTES.qa-app-layer.*`
- `DESIGN-NOTES.qa-minimal-schema.*`
- `DESIGN-NOTES.donation-app-layer.*`

這一層代表 Mycel 已開始像 platform，而不只是 document protocol。

## 8. Fund, Custody, and Execution Layer

這一層處理受治理的經濟流程與委派執行。

責任：

- donation modeling
- fund disbursement policy
- execution-intent generation
- signer enrollment 與 consent boundaries
- threshold custody
- execution receipts

主要文件：

- `PROFILE.fund-auto-disbursement-v0.1.*`
- `DESIGN-NOTES.policy-driven-threshold-custody.*`
- `DESIGN-NOTES.auto-signer-consent-model.*`
- `DESIGN-NOTES.sensor-triggered-donation.*`

這是整個 full stack 裡最敏感、也最需要操作謹慎的一層。

## 9. Anonymity and Privacy Layer

這一層處理身份洩漏與 deployment privacy posture（部署隱私姿態）。

責任：

- transport-anonymity posture
- metadata minimization
- role separation
- local hardening
- runtime hardening
- deployment-tier boundaries

主要文件：

- `DESIGN-NOTES.mycel-anonymity-model.*`
- `PROFILE.mycel-over-tor-v0.1.*`

這一層之所以重要，是因為 Mycel 的可驗證歷史很容易在時間上變得高度可連結。

## 10. Client Surface Layer

這一層是使用者真正接觸到的部分。

責任：

- accepted-text reading
- history inspection
- branch visibility
- source 與 citation browsing
- Q&A 與 commentary navigation
- sync state display
- policy/profile visibility

主要文件：

- `IMPLEMENTATION-CHECKLIST.*`
- 各種 design notes 中偏 reader-oriented 的部分

這一層目前還沒有被收成單一 UI note，但現有文件已經暗示一個 reader-first client 模型。

## 11. Meta and Direction Layer

這一層不直接定義 interoperability。

它定義的是：

- project direction
- document boundaries
- upgrade philosophy
- repository working rules

主要文件：

- `PROJECT-INTENT.*`
- `DESIGN-NOTES.mycel-protocol-upgrade-philosophy.*`
- `AGENTS.md`

這一層的重要性在於，它能防止 project intent 的語言滲進技術層。

## 12. Dependency Shape

大致上的依賴方向是：

`protocol core`
-> `verification and local state`
-> `sync and transport`
-> `governance and accepted-state selection`
-> `canonical text and app-layer models`
-> `fund / custody / execution`
-> `client and deployment behavior`

不是每個 deployment 都需要把上層全部做完。

例如：

- 最小 reader 可以停在 accepted text 與 sync
- 較完整的 deployment 可以加 canonical text、Q&A 與 commentary
- 高度雄心的 deployment 才會再加 fund automation 與 threshold custody

## 13. Three Realistic Build Shapes

### 13.1 Minimal Mycel

包含：

- protocol core
- local object store
- revision replay
- wire sync
- accepted-head rendering

這已經是一個真正的 protocol client，但還不是完整平台。

### 13.2 Reader-plus-governance Mycel

包含：

- minimal Mycel
- canonical text handling
- citations
- Q&A
- commentary
- accepted-reading profiles

這是一個嚴肅的知識或 reference-text 系統。

### 13.3 Full-stack Mycel

包含：

- reader-plus-governance Mycel
- app layer
- donation 與 fund systems
- threshold custody
- automatic execution paths
- anonymity-aware deployment profiles

這時它就不再只是狹義的 protocol project，而會變成完整的 distributed text、governance 與 application ecosystem。

## 14. What This Means for Planning

整套文件不應被誤解成一個單一、立即要全部實作的目標。

更合理的規劃方式是：

- 決定哪些 layers 現在在 scope 內
- 哪些 layers 暫時只停在 profile
- 哪些 layers 先維持在 design-note 形態

最實際的結論是：

- Mycel 已經描述了一個很大的最終系統
- 但第一版實作仍然應刻意保持狹窄

## 15. Recommended Next Step

在這張 full-stack map 之後，下一步最適合做的是把工作再分成：

- `minimal`
- `reader-plus-governance`
- `full-stack`

這樣才能把概念上的整體地圖，轉成真正可執行的 build roadmap。
