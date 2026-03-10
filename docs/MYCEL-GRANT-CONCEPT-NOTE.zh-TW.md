# Mycel Grant Concept Note

## 專案標題

**Mycel：可驗證文本歷史、依規則導出的預設閱讀版本與去中心化複製**

## 摘要

Mycel 是一個面向文本系統的開放協定，適用於同時需要下列能力的場景：

- 可驗證的 revision 歷史
- 依規則導出的預設閱讀版本
- 不依賴強制全域共識的去中心化複製

它特別適合長期文本、評註傳統、受治理的參考文本集合、典藏與檔案環境，以及其他要求歷史、詮釋與審計能力必須長期保留，且允許多個有效分支並存的情境。

Mycel 要補的是一個現有工具沒有很好覆蓋的空缺。中央化協作系統雖然方便，但審計性與可攜性薄弱。偏向程式碼工作流的分散式工具雖然保留歷史能力強，卻不是為受治理的預設閱讀而設計。區塊鏈式系統提供強共識，但代價高，且其前提對很多文本治理工作流而言不是必要，甚至並不適合。

Mycel 的主張是：歷史、accepted reading 與 replication 應該互通，但不應被壓進同一個共識機制。

## 問題陳述

許多重要文本系統需要的能力，介於一般文件編輯與 blockchain 共識之間。

典型例子包括：

- 受治理的法律或政策評註
- 機構級參考文本與典藏
- 學術註解系統
- 長期維護的技術、規範或文化文本集合
- 依 accepted textual state 運作的政策驅動執行系統

在這些情境中，利害關係人需要回答：

- 內容改了什麼
- 這段歷史是否能被獨立驗證
- 目前預設應閱讀哪個版本
- 為什麼是這個版本
- 其他有效分支如何繼續保留且可審計

現有系統通常只對其中一部分最佳化，卻會削弱另一部分。Mycel 的目標是讓這些要求可以同時成立。

## 方法

Mycel 把系統拆成三層：

1. **可驗證歷史**  
   Revisions 應該可以從 canonical objects replay、檢查與重建。

2. **受治理的閱讀狀態**  
   預設 accepted reading 應由固定 profile 規則與已驗證治理訊號推導，而不是來自任意本地偏好，也不是宣稱有一個全域真相。

3. **去中心化複製**  
   Objects 應能在 peers 之間複製，而不需要每個讀者都接受同一個全域共識結果。

這個架構的目標是在避免不必要共識成本的同時，保留彈性與審計性。

## 目前狀態

目前專案已具備：

- 持續成長中的 v0.1 protocol 與 wire-spec 文件
- Rust 驅動的內部驗證與 simulator 工具鏈
- 基於 replay 的 revision verification
- deterministic 的 accepted-head inspection
- local object store ingest 與 rebuild
- fixtures、simulator topologies 與 negative validation coverage

目前專案尚未提供：

- 完成的公開 interoperable client
- 完整的 end-to-end wire sync
- production-ready 的 node 或終端使用者應用

## Grant 支持能加速什麼

Grant 支持將加速最關鍵的公共基礎設施工作：

- shared protocol parsing 與 canonicalization 收斂
- replay 與 `state_hash` verification 強化
- 可重建的 local storage 與 accepted-head selection
- deterministic negative testing 與 interop fixtures
- 更清楚的 profile、schema 與文件邊界
- 一個更窄但可互通的 first client

這一層具有最高槓桿。只要 shared core 變穩，後續 profiles、applications 與 deployment models 都會更安全、也更可重用。

## 預期成果

在支持下，Mycel 目標交付：

- 更強的 open protocol core，支撐可驗證文本系統與文化參考文本集合
- 更完整的 first-client implementation path
- 可重用的 fixtures 與 negative tests，支援 interoperability
- 更清楚的公共文件，說明 governed reading 與 accepted-state derivation
- 作為未來 text-governance、commentary 與文化保存應用的 reference base

## 為什麼這個專案重要

Mycel 正在補一塊尚未被充分建設的數位基礎設施：那些必須同時保留歷史、治理與詮釋，但又不應被中央平台控制或強制全域共識綁死的系統。

這項工作具有公共價值，因為它支持：

- 可長期保存且可審計的知識系統
- 文本治理與文化保存基礎設施
- 可重現的詮釋與評註工作流
- 相對於封閉平台的開放協議替代方案

## 適合的資助方向

Mycel 特別適合下列資助類型：

- 開放數位基礎設施
- 公共利益導向的協議開發
- 可驗證知識系統
- 可互通的 open-source foundations
- 值得信任的資料與治理工具

## 結語

Mycel 雖然仍在早期，但已經從純概念前進到具體的 spec、implementation 與 validation 路徑。此階段的支持，將有助於把一個有潛力的協議方向，推進成為可用於受治理文本系統與文化治理工作的公共基礎設施。
