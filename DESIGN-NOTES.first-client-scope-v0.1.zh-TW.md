# First-client Scope v0.1

狀態：design draft

這份文件定義 Mycel 第一版 client 最窄、最實際的範圍。

核心設計原則是：

- 第一版 client 應先證明 protocol 可行，而不是一次把整個平台做滿
- reader behavior 應優先於豐富 authoring 或 app execution
- 一開始就要明確選定支援哪些 profiles
- 凡是不在目前範圍內的能力，都應刻意延後

## 0. Goal

設定一個現實可做的 first-client 目標，使其能：

- 驗證 core protocol
- 驗證 wire sync 路徑
- 驗證 accepted-head selection
- 讓讀者閱讀 accepted text

這份文件刻意比整套文件集的總範圍窄很多。

## 1. Build Shape

建議的第一版 client 應是：

- 一個 reader-first client
- 一個本地 node process
- 一個本地 object store
- 一組受限的 wire implementation
- 一組狹窄的 profile set

它不應是：

- 完整 editor
- 通用 app runtime
- fund-execution node
- signer node
- 廣域 public mesh node

## 2. In-Scope Layers

第一版 client 應只包含以下層次：

1. protocol core
2. object verification 與 local state
3. 最小同步與 transport
4. governance 與 accepted-state selection
5. reader-facing client surface

可選地包含：

- 一個狹窄的 anonymous transport profile
- 一個狹窄的 canonical-text reading profile

## 3. Required Protocol Features

第一版 client 應實作：

- `document`
- `block`
- `patch`
- `revision`
- `view`
- `snapshot`
- canonical serialization
- derived-ID verification
- signature verification
- 以 replay 為基礎的 `state_hash` verification

必要 wire messages：

- `HELLO`
- `MANIFEST`
- `HEADS`
- `WANT`
- `OBJECT`
- `BYE`
- `ERROR`

第一版可延後：

- `SNAPSHOT_OFFER`
- `VIEW_ANNOUNCE`

## 4. Required Local Capabilities

第一版 client 應支援：

- 一個持久化的本地 object store
- 可重建的本地 indexes
- 針對一個固定 reader profile 計算 accepted head
- 僅在 object 驗證通過後才進行 indexing
- 僅靠 canonical objects 就能重建本地 state

它不應依賴：

- 伺服器端資料庫基礎設施
- 外部搜尋服務
- 背景 execution runtimes

## 5. Required Reader Behavior

第一版 client 應能：

- 透過 logical ID 開啟文件
- 計算並顯示 accepted head
- render accepted text
- 顯示基本 history context
- 顯示 alternative heads
- 顯示目前是哪個 profile 選出了 accepted head

建議但仍屬最小範圍的功能：

- 若存在 citations 或 source references，則可顯示
- 提供簡短的 `Why this text` 面板

## 6. Chosen Profiles

第一版 client 應明確選定它支援哪些 profiles。

建議第一組：

- 一個固定 reader profile，用於 accepted-head selection
- 可選的 `mycel-over-tor-v0.1`

延後的 profiles：

- `fund-auto-disbursement-v0.1`
- signer-oriented custody profiles
- runtime-heavy app profiles

第一版 client 不應宣稱支援那些它無法端到端實作的 profile。

## 7. Explicit Non-goals

第一版 client 應延後以下所有能力：

- 豐富的 editing UX
- editor-maintainer 的 authoring workflows
- view-maintainer 的 publication workflows
- donation execution
- threshold custody
- automatic effect execution
- sensor-triggered flows
- 廣泛的 Q&A authoring workflows
- 超出 bounded configuration 的 public anonymous mesh discovery

延後不是失敗，而是刻意收窄。

## 8. Networking Posture

第一版 client 應採 bounded network posture。

建議模式：

- restricted peer list
- explicit bootstrap peers
- 可選的 Tor-routed transport

第一版應避免：

- uncontrolled public discovery
- transport fallback ambiguity
- reader 與 signer/runtime 行為混在同一個 client 內

## 9. UI Surface

第一版 UI 應維持 reader-first。

主要畫面：

- library 或 document list
- accepted text reader
- history 與 branch inspection
- profile 或 selection status

延後的 UI：

- 完整 curator console
- fund operations console
- signer controls
- runtime control panels

## 10. Testing Gate

若未通過以下測試，第一版 client 不應視為完成：

- canonical object parsing tests
- derived-ID verification tests
- wire sync 端到端測試
- 以 replay 為基礎的 state reconstruction tests
- deterministic accepted-head tests
- rebuild-from-store tests

## 11. Minimal Success Criteria

若一位新使用者可以做到以下六件事，第一版 client 就算成功：

1. 啟動本地 node
2. 連到 bounded peer set
3. 同步 objects
4. 在本地驗證 objects
5. 計算某份文件的 accepted head
6. 帶著基本 trace context 閱讀 accepted text

只要這六件事能運作，Mycel 就擁有一個真正的 first client。

## 12. Recommended Next Step After This Scope

當第一版 client 可用後，下一步的擴張方向應只先選一個：

- 更完整的 canonical-text reading
- 基本 authoring workflows
- 某個 profile-specific app support

即使往前擴，也應一次只增加一個 major layer。
