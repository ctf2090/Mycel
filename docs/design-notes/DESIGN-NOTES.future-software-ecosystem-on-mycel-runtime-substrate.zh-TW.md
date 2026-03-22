# Future Software Ecosystem on Mycel Runtime Substrate

狀態：design draft

這份筆記想像一個未來：Mycel 式的 signed、on-demand runtime loading 不再只是小眾架構選項，而成為主流軟體環境的一部分。

這份文件不是要預測單一、精確的市場結局。

它的目的是描述：如果軟體從 install-first packaging 轉向 trusted host 在 runtime 按需抓取、驗證並執行 signed modules，那整個生態會有哪些結構性改變。

相關文件：

- `DESIGN-NOTES.dynamic-module-loading.*`：module-level execution 模型
- `DESIGN-NOTES.signed-on-demand-runtime-substrate.*`：較大的 runtime-substrate framing
- `DESIGN-NOTES.minimal-mycel-host-bootstrap.*`：可承載這個模型的最小 trusted local host

## 0. 目標

描述如果 Mycel Runtime Substrate 成為主流，軟體文化、分發、信任與產品結構可能出現的變化。

這份文件聚焦於：

- 軟體分發
- trust 與 policy 市場
- application structure
- 使用者期待
- 生態級風險

## 1. 最根本的位移

最深的變化不會只是：

-「大家改用 Mycel apps，而不是普通 apps」

而會是：

- 軟體不再以已安裝套件為中心
- 軟體改以已簽章、可抓取、經 policy gate 的 capabilities 為中心

在這個世界裡，軟體的日常單位不再主要是：

- 一個永久安裝在本機的 app bundle

而更可能是：

- 一組 signed modules
- 一份 host policy
- 一個 state model
- 一組在 runtime 被授權的 capabilities

## 2. Installation 退居次要

在主流的 Mycel Runtime Substrate 世界裡，「安裝軟體」的重要性會下降，而「授權 host 載入某些 capabilities」會變得更中心。

典型使用流程會從：

- 下載 app
- 安裝 app
- 更新 app

轉成：

- 打開某份 state、workflow 或 document
- 讓 host 解析並載入缺少的 modules
- 對 requested capabilities 做允許或拒絕
- 只把值得保留的 modules 留在本地 cache

這不代表 installation 會完全消失。

但它會不再是軟體體驗的絕對中心。

## 3. App Store 會變成 Trust 與 Policy 市場

今天的 app store 主要是：

- 下載目錄
- 付款通道
- 審核閘門

在 Mycel 主流的世界裡，它們最重要的角色會變成：

- trust distribution
- signer reputation
- capability policy packaging
- module-family admission
- compatibility guarantees

真正重要的問題不再只是：

- 我去哪裡下載這個 app？

還會包括：

- 我該信任哪個 signer？
- 我該採用哪套 host policy？
- 哪些 module families 在這個 profile 或組織裡被接受？

## 4. 軟體產品會被拆解

主流軟體產品不再主要以一個封閉 binary artifact 呈現。

它更常會以 artifact graph 的形式出現：

- state schemas
- module metadata
- module blobs
- UI renderers
- policy helpers
- execution modules
- audit 與 receipt modules

這會提高：

- reuse
- auditability
- portability

但也會提高：

- policy complexity
- compatibility management
- signer 與 dependency governance 的負擔

## 5. State 會比 App Shell 更重要

一個重要的文化變化會是：application state 的價值高於目前的 app wrapper。

使用者真正關心的不只是：

- 自己裝了哪個 app

而是：

- 狀態能不能跨 host 攜帶
- 狀態能不能在不同 trusted module sets 下仍被讀取
- governance 與 execution history 是否保持可驗證

這會讓軟體更不像：

- 擁有一個封裝好的產品

而更像：

- 維持一份受信任 state 的連續性

## 6. Frontend、Backend 與 Plugin 邊界會模糊

今天的軟體生態很習慣把東西硬切成：

- frontend
- backend
- plugin
- local app
- cloud service

在 Mycel Runtime Substrate 模型裡，這些更會變成 execution context 的差異，而不是硬性的軟體分類。

同一個邏輯功能可能以不同形式存在：

- local renderer
- server-side policy worker
- CLI transformer
- browser-hosted presentation module

重要的邊界會變成：

- 它在哪裡執行
- 它擁有哪些 capabilities
- 它可以解讀或變動哪些 state surfaces

## 7. 安全會從 App Trust 轉成 Capability Trust

今天很多使用者其實只回答一個粗粒度問題：

- 我信不信這個 app？

在 substrate 世界裡，更重要的問題會變成：

- 我信不信這個 signer？
- 我信不信這個 module family？
- 我願不願意授予這個 capability？
- 它能不能影響 accepted-state derivation？
- 它能不能觸發外部 side effects？

這會讓安全粒度細很多。

但也會帶來新的生態問題：

- capability fatigue

## 8. 軟體公司會改變形狀

如果這個模型成為主流，軟體公司會越來越像：

- signer maintainers
- policy maintainers
- schema maintainers
- compatibility maintainers
- audit 與 trust maintainers

競爭優勢會逐漸從：

- 控制一個封閉 client bundle

轉向：

- 維護可信任的 module families
- 提供穩定、治理良好的 schemas
- 提供高品質 policy defaults
- 累積長期 signer reputation

## 9. 開源會更強，也更制度化

開放生態在這種世界裡可能會更強，因為：

- modules 更容易重用
- state formats 更容易保持可攜
- trust decisions 更明確

但開放生態也會變得更制度化，圍繞：

- signer governance
- capability review
- artifact retention
- compatibility policy

開源社群最核心的問題會從：

- 我能不能 build 這個 package？

轉成：

- 這個 artifact family 是否值得信任、可被審查、也能安全地被主流 hosts 接納？

## 10. Offline 會變成品質紀律

在 fetch-on-demand 的世界裡，online resolution 會變得很自然。

所以真正做得好的產品，會在這些地方拉開差距：

- pin 住 critical modules
- 保持 offline continuity
- warm-cache execution
- safe cold-cache failure

offline support 不再只是模糊的 marketing checkbox。

它會變成一種具體紀律，回答：

- 哪些被 pin 住了
- 哪些可以被重建
- 哪些在沒有網路 resolve 時不能安全執行

## 11. 作業系統會變得更像 Host

傳統作業系統不會消失，但它的可見角色會改變。

它會越來越像：

- host runtime
- verifier shell
- trust 與 capability mediator
- module cache manager

而不再是：

- 軟體 identity 的主要所在地

這表示整個技術棧的可見中心會往上移到：

- trusted host policy
- state model
- runtime substrate

## 12. 新的權力結構會出現

這個未來不會消除權力集中。

它只會改變權力落在哪裡。

很可能的新權力中心包括：

- trust-anchor maintainers
- major host vendors
- 大型 policy registries
- artifact retention providers
- compatibility-profile authorities

也就是說，這個未來可能會在某個意義上更開放，但在另一個意義上變得更制度化、也更政治化。

## 13. 可能的主要失敗模式

這個生態很可能會出現幾種新的系統級失敗模式。

### 13.1 Trust-List Monopoly

如果少數 hosts 控制預設 accepted signer sets，它們就會變成新的平台 gatekeepers。

### 13.2 Capability Fatigue

如果 capability 與 policy prompts 太頻繁、太難懂，使用者就無法再做有意義的信任決策。

### 13.3 Artifact Availability Politics

如果關鍵 module blobs 沒有人長期鏡像與保存，那實質上的軟體自由就會收斂到「誰還保有 artifacts」。

### 13.4 Governance Overload

如果每一個有用的 module family 都需要很重的治理負擔，整個生態就會變得過於難用。

### 13.5 Host Vendor Overreach

如果 hosts 太過強勢與主觀，表面上的 open substrate 最終仍可能退化成另一種 platform lock-in。

## 14. 這個世界裡的日常運算

對一般使用者來說，主流軟體體驗可能會變成：

- identity 與 state 可以跨 host 延續
- interfaces 的變化會比 underlying state 更流動
-「打開一個 workspace」比「啟動一個 app」更重要
- execution 更像是在授權 capabilities，而不是啟動固定 installation

整體感受會更不像：

- 啟動一個盒裝軟體產品

而更像：

- 進入一個由 trusted host 管理、工具按需組裝的計算空間

## 15. 實務總結

如果 Mycel Runtime Substrate 成為主流，軟體生態很可能會從：

- install-first
- app-bundle-centric
- platform-siloed

轉向：

- state-first
- signer- 與 policy-mediated
- capability-gated
- fetch-on-demand

整個生態會更可組裝，也更可審計。

但它同時也會更依賴好的 trust governance 與更好的 user-facing policy design。

## 16. 開放問題

- 主流 hosts 的 default trust anchors 應該由誰控制？
- capability UX 要怎麼設計，才不會變得不可用？
- 哪些 module classes 應該被普遍鏡像，以保證長期 software continuity？
- 主流生態應如何區分 portable state 與 signer-specific execution logic？
- host 到什麼程度會停止中立，轉而變成平台治理者？
