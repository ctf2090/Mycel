# Mycel Roadmap

狀態：late partial progress，已在最近一批 wire-session reachability、store-backed bootstrap 與 transcript-backed sync-pull 後刷新；`M4` 現在已有 object-body verification 與 first-time / incremental transcript-backed pull coverage，但 peer-driven end-to-end sync 仍未完成

這份 roadmap 將目前 README 的優先順序、implementation checklist，以及 design-note 的 planning 指引，整理成 repo 層級的建置順序。

它刻意維持窄版範圍：

- 先做第一個可互通客戶端
- 對協定核心變更保持保守
- 在擴大範圍前，先把成熟想法落成 profiles、schemas 與 tests

## 目前位置

目前 repository 已具備：

- 持續成長中的 v0.1 protocol 與 wire-spec 文件集
- 適合做內部驗證與決定性模擬器工作流程的 Rust CLI
- `mycel-core` 對 object schema metadata、object-envelope parsing、replay-based revision verification、local object-store ingest/rebuild、persisted store indexes，以及 accepted-head inspection 的支援
- `mycel-core` 對早期 wire-envelope parsing、payload validation、通用 wire signature verification、sender mapping、minimal message set 的 inbound session sequencing/head-tracking、reachability gating，以及 store-backed session bootstrap 的支援
- transcript-backed sync-pull core 與 CLI entry point，已具備 first-time 與 incremental verify/store coverage
- 更集中化的 canonical hash 與 signed-payload helpers，已在 verification、replay、head/render 預先驗證、authoring，以及部分 CLI smoke 路徑之間重用
- 早期 reader-plus-governance surfaces，涵蓋 accepted-head rendering、具名 fixed-profile selection，以及具備 editor-admission 感知的 inspect/render workflows
- `document`、`block`、`patch`、`revision`、`view`、`snapshot` 在 parser / verify / CLI 路徑更廣的 strictness-surface coverage、更完整的 `object inspect` warning surface、對 merge 與 cross-document revision edge 更強的 signature-edge 與 replay/verification smoke coverage、更清楚的 multi-hop ancestry replay failure context，以及 isolate 過的 validate-peer fixtures
- 以 `assert_cmd`、`predicates`、`tempfile` 與小範圍 `rstest` 建立的較可維護 CLI test base
- simulator fixtures、topologies、tests 與 reports，作為 regression coverage

目前 repository 尚未具備：

- 完整可互通的節點實作
- 完成的 object-authoring 與 storage-write path
- 端到端 wire sync
- 正式可上線的公開 CLI 或 app

## Roadmap 摘要

### 現在

目前的 lane 是：

1. 完成窄版的第一個客戶端核心
2. 收掉 shared core 在 parsing 與 canonicalization 上剩餘的缺口
3. 一邊持續擴充 fixtures、模擬器 coverage 與負向測試，一邊開始 reader-plus-governance 的讀取路徑
4. 讓 `M4` 維持窄版範圍，把 transcript-backed first-time / incremental sync pull 往 minimal peer-driven sync 收斂

### 下一步

當窄版 core 穩定後，下一條 lane 是：

1. 面向 reader 的 accepted-head 與 governance 工作流程
2. fixed-profile accepted reading
3. reader-first 的 text reconstruction 與 inspection

### 之後

更後面的 lane 是：

1. canonical wire sync
2. 端到端 peer replication
3. 建立在穩定 protocol core 之上的選擇性 app-layer expansion

## Planning Levels

這份 roadmap 採用 design notes 已經暗示的 planning 分層：

1. `minimal`
2. `reader-plus-governance`
3. `full-stack`

後一層都假設前一層已經穩定。

## Milestones

這份 roadmap 以以下 milestones 追蹤：

1. `M1` Core Object and Validation Base
2. `M2` Replay, Storage, and Rebuild
3. `M3` Reader and Governance Surface
4. `M4` Wire Sync and Peer Interop
5. `M5` Selective App-Layer Expansion

## Phase 1: Minimal

目標：達到一個窄版的第一個客戶端，能夠以決定性的方式 parse、verify、store、replay，並 inspect Mycel objects。

### Deliverables

1. 所有 v0.1 object families 的 shared protocol object model
2. canonical serialization、derived ID recomputation 與 signature verification
3. replay-based revision verification 與 `state_hash` 檢查
4. 本地物件儲存與可重建的索引
5. 穩定的內部 CLI/API，可做 validation、object verification、object inspection 與 accepted-head inspection
6. object 與 simulator validation 的 interop fixtures 與 negative tests

### Exit Criteria

1. 必要 object types 可以穩定地 parse 與 validate
2. canonical IDs 與 signatures 是決定性的
3. revision replay 可只靠 stored objects 通過
4. fixed profile 下的 accepted-head selection 是決定性的
5. 本地儲存可以只靠 canonical objects 重建

### Current Status

仍屬 late partial progress，已接近 phase 尾端，但還不能宣告 complete。

已在進行中或部分完成：

1. Shared object schema metadata
2. Shared object-envelope parsing
3. Object inspection 與 verification
4. Replay-based revision verification 與 `state_hash` 檢查
5. Local object-store ingest、rebuild、persisted manifest indexing 與 query surfaces
6. Accepted-head inspection，包括 store-backed selector object loading
7. Internal validation 與 simulator harness CLI

仍缺少或未完成：

1. malformed field-shape depth、剩餘 inspect-surface parity polish 與剩餘 semantic-edge strictness 的最後收尾工作
2. verified ingest 之外的窄版 object-authoring 與 write path
3. 建立在 accepted-head selector 之上的更乾淨 reader-facing profile surface
4. 將 shared canonicalization reuse 擴展到未來 wire-envelope work
5. 足以宣告 Phase 1 exit criteria 完成的最後收尾工作

### 本 phase 的 milestones

#### M1: Core Object and Validation Base

重點：

1. shared object schema 與 parsing
2. canonical object validation rules
3. object inspection 與 verification tooling
4. interop fixtures 與 negative validation coverage

完成門檻：

1. 所有必要的 v0.1 object families 都能 parse 進 shared protocol layer
2. derived IDs 可以 deterministic 地重新計算
3. 必要 signature rules 可以被一致執行
4. CLI 與 tests 能穩定暴露 validation 與 verification surfaces，供內部 workflows 使用

目前判讀：

接近完成。shared parsing、更收斂的 canonical helper module、top-level core-version equality checks、保留路徑資訊的 nested parser field errors、更廣的 parser / verify / CLI strictness-surface coverage、更完整的 inspect-surface parity、更嚴格的 replay dependency verification 與 sibling declared-ID determinism、直接涵蓋無效 sibling/parent dependency ID 與 signature 的 CLI smoke coverage、更清楚的 multi-hop ancestry failure context、isolate 過的 validate-peer fixtures，以及 canonical reproducibility coverage 都已存在；剩餘工作大多是最後的 malformed-field depth 與 semantic-edge 收尾，加上一些 milestone-close proof points。

目前 repo 已可見：

1. shared schema metadata
2. shared object-envelope parsing
3. shared canonical JSON、derived-ID recomputation 與 signed-payload helpers
4. object inspection 與 verification
5. protocol-level typed parsing for supported object families，包括 `document`、`block`、`patch`、`revision`、`view`、`snapshot`
6. shared JSON loading 中的 duplicate-key rejection 與 unsupported-value rejection
7. IDs、signed payloads 與 signatures 的 canonical round-trip 與 reproducibility coverage
8. internal validation 與 simulator harness coverage

主要剩餘缺口：

1. 在廣泛 unknown-field 與 invalid-type rejection 之後，final malformed-field depth 與 semantic-edge strictness closure
2. 目前 revision / patch、replay 與 view / snapshot batches 之外，其餘 semantic edge cases 的更深 `mycel-core` coverage
3. 把剩餘 replay 衍生的 `state_hash` 與未來 wire-validation canonicalization 路徑收斂到 shared helper module 上
4. 在擴大更多表面前，先釐清 milestone-close criteria

Implementation anchors：

1. Crates:
   `crates/mycel-core`
   `crates/mycel-sim`
   `apps/mycel-cli`
2. Key files:
   `crates/mycel-core/src/protocol.rs`
   `crates/mycel-core/src/verify.rs`
   `crates/mycel-core/src/lib.rs`
   `crates/mycel-sim/src/validate.rs`
   `apps/mycel-cli/src/object.rs`
   `apps/mycel-cli/tests/object_verify_smoke.rs`
   `apps/mycel-cli/tests/object_inspect_smoke.rs`
   `apps/mycel-cli/tests/validate_smoke.rs`
3. Useful commands:
   `cargo test -p mycel-core`
   `cargo test -p mycel-cli`
   `cargo run -p mycel-cli -- object inspect <path> --json`
   `cargo run -p mycel-cli -- object verify <path> --json`
   `cargo run -p mycel-cli -- validate <path> --json`

建議 build order：

1. 先在 `crates/mycel-core/src/protocol.rs` 完成所有必要 object families 的 shared protocol parsing coverage
2. 把 canonical object mechanics 移到 shared protocol-level helpers，而不是只留在 `crates/mycel-core/src/verify.rs`
3. 擴充 `crates/mycel-core/src/verify.rs`，讓它對每個支援 object family 都消費這些 shared helpers
4. 在擴張 CLI surface 前，先加深 `mycel-core` tests，讓 object-rule regressions 在 CLI 層以下就被抓到
5. 只有在 shared core 穩定後，才擴大 CLI 與 simulator-facing validation coverage

第一批 implementation batch：

在目前 repo 狀態中已完成：

1. `document` 與 `block` logical-ID handling 的 typed parsing coverage，位於 `crates/mycel-core/src/protocol.rs`
2. `patch`、`revision`、`view`、`snapshot` derived-ID fields 的 typed parsing coverage，位於 `crates/mycel-core/src/protocol.rs`
3. 從 verification-only ownership 中抽出 shared protocol-level canonical JSON、derived-ID recomputation 與 signed-payload helpers
4. `crates/mycel-core/src/verify.rs` 已對所有支援 object families 消費 shared typed parsing 與 canonical helpers
5. `mycel-core` tests 已覆蓋 malformed object type、missing signer fields、wrong derived-ID fields、duplicate keys、unsupported values，以及 malformed field-shape cases，然後才繼續擴張 CLI behavior

這批的具體 completion check：

已完成：

1. `protocol.rs` 透過單一 shared parsing layer 理解所有目前支援的 object families。
2. `verify.rs` 不再持有 canonical object mechanics 的唯一實作。
3. `cargo test -p mycel-core` 直接覆蓋 shared protocol helpers 與 object-family edge cases。
4. 現有的 `object inspect` 與 `object verify` CLI contract 仍可通過，不需要 CLI-only fallback logic。

#### M2: Replay, Storage, and Rebuild

重點：

1. replay-based revision verification
2. `state_hash` recomputation
3. local object-store indexing
4. store rebuild 與 recovery workflows
5. 初始的 object-authoring 與 storage-write path

完成門檻：

1. revisions 可從 stored objects deterministic 地 replay
2. replay 過程中會重新計算並驗證 `state_hash`
3. indexes 可只靠 canonical objects 重建
4. 第一個 client 至少具備窄版 object creation 與 write path

目前判讀：

已大幅展開，但尚未完成。replay-based verification、store rebuild、persisted indexes、窄版 store write path、初始的保守型 merge-authoring workflow，以及能保留 ancestry context 的 render/store verification 都已存在，但這個 milestone 仍未到可關閉狀態。

主要剩餘缺口：

1. 持久化 store indexes 在 reader workflows 中的更廣 reuse
2. 在目前直接 store-backed replay proof point 之外，進一步補強與更真實 fixture sets 綁定的 replay 與 store reconstruction coverage
3. 保守型 merge authoring 現在已覆蓋基本 move/reorder、insert/delete 組合、reparent 到新引入 parent 的 case、簡單的 composed parent-chain reparenting，以及更廣的初步 nested structural matrix，但更豐富的 nested/reparenting conflict cases 仍需 manual curation
4. 擴大 shared core reuse，避免 authoring 與 replay helpers 過度停留在 CLI-driven glue

Implementation anchors：

1. Crates:
   `crates/mycel-core`
   `apps/mycel-cli`
2. Key files:
   `crates/mycel-core/src/verify.rs`
   `crates/mycel-core/src/protocol.rs`
   `IMPLEMENTATION-CHECKLIST.en.md`
   `fixtures/README.md`
3. Expected next code areas:
   replay 與 `state_hash` logic 很可能會先落在 `crates/mycel-core`
   storage-write 與 rebuild entry points 很可能需要新 files 或 modules，而不是更多 CLI-only glue
4. Useful commands:
   `cargo test -p mycel-core`
   `cargo run -p mycel-cli -- validate fixtures/object-sets/minimal-valid/fixture.json --json`

建議 build order：

1. 先在 `crates/mycel-core` 落 replay primitives，再建立任何新的 storage-writing CLI flow
2. 將 deterministic `state_hash` recomputation 建立在 replay 之上，而不是做成分離的孤立 utility
3. 等 replay output 穩定後，再定義 minimal local store 與 rebuild model
4. 只有在 replay 與 rebuild semantics 固定後，才加入窄版 object builder 與 storage-write path
5. 最後才暴露 CLI 或 API entry points，讓它們建立在 shared replay 與 storage logic 之上，而不是發明平行行為

## Phase 2: Reader-Plus-Governance

目標：在 deterministic accepted-head behavior 與 governance-aware reading state 基礎上，加入可用的 reader-oriented client layer。

### Deliverables

1. 作為 governance signal input 的 verified View ingestion
2. fixed reader profiles 的穩定 accepted-head selection
3. 從 replayed revision state 進行 reader-first text rendering
4. reader workflows 與 governance publication workflows 的清楚分離
5. 可 inspect accepted heads、views 與 governance decision detail 的 CLI/API 支援

### Exit Criteria

1. 固定 reader profile 在重複執行下能產生穩定 accepted heads
2. governance inputs 與 discretionary local policy 清楚分離
3. reader 可以從 stored objects reconstruct 並 inspect accepted text state
4. decision summaries 與 typed arrays 已穩定到足以供 tooling 與 tests 使用

### Current Status

屬早期 partial progress，現在已在 deterministic selector path 之上具備 accepted-head rendering、具名 fixed-profile selection，以及具備 editor-admission 感知的 inspect/render behavior。

已在進行中或部分完成：

1. Accepted-head inspection
2. 以 typed arrays 呈現的 structured decision detail
3. accepted-head inspection 的 store-backed selector object loading
4. 可從 persisted store state 或 explicit bundle objects 產生 accepted-head render output
5. 為 accepted-head inspection 與 render workflows 提供具名 fixed-profile selection
6. 在具名 profile 與 store-backed 路徑中，提供具備 editor-admission 感知的 accepted-head inspect/render behavior
7. 提供獨立於 reader-facing `head` commands 的 `view inspect` / `view list` / `view publish` governance workflows，並具備 listing filter、sort、time window、grouped summary 與 projection modes
8. persisted governance reverse indexes，支援依 maintainer、profile 與 document 反查 view
9. simulator 與 validation workflows，涵蓋 peer、topology、test 與 report 範圍

主要剩餘缺口：

1. 超出 selector、reverse view indexes 與 replay inputs 的更廣泛 governance-state persistence
2. 超出目前初始 filtered / sorted / projected `view` inspection / listing / publication workflow 的專用 governance surfaces
3. 超出最小具名 fixed-profile surface 的 reader-facing profile ergonomics
4. 後續可與 wire / sync 對齊的 governance-state tooling

Implementation anchors：

1. Crates:
   `crates/mycel-core`
   `apps/mycel-cli`
2. Key files:
   `crates/mycel-core/src/head.rs`
   `apps/mycel-cli/src/head.rs`
   `apps/mycel-cli/tests/head_inspect_smoke.rs`
   `fixtures/head-inspect/README.md`
3. Useful commands:
   `cargo run -p mycel-cli -- head inspect <doc-id> --input <path-or-fixture> --json`
   `cargo run -p mycel-cli -- head render <doc-id> --input <path-or-fixture> --json`
   `cargo run -p mycel-cli -- view inspect <view-id> --store-root <store> --json`
   `cargo run -p mycel-cli -- view list --store-root <store> --latest-per-profile --limit 10 --summary-only --group-by profile-id --json`
   `cargo run -p mycel-cli -- view publish <path> --into <store> --json`
   `cargo run -p mycel-cli -- store index <store> --governance-only --maintainer <maintainer> --json`
   `cargo test -p mycel-cli head_inspect`

## Phase 3: Full-Stack

目標：從 local verification 與 governed reading，擴展到 interoperable replication、更豐富的 profiles，以及選擇性的 app-layer support。

### Deliverables

1. Canonical wire envelope implementation
2. `HELLO`、`MANIFEST`、`HEADS`、`WANT`、`OBJECT`、`BYE`、`ERROR`
3. 對支援 profiles 可選的 `SNAPSHOT_OFFER` 與 `VIEW_ANNOUNCE`
4. Peers 之間的 end-to-end sync workflow
5. 面向 local authoring tools 的 merge-generation profile support
6. 建立在穩定 protocol core 之上的選擇性 app-layer profiles

### Exit Criteria

1. peers 之間的 minimal sync 可端到端成功
2. 收到的 objects 會先驗證，再進行 indexing 與 exposure
3. merge generation 能產生可 replay 的 patch operations
4. 除非理由明確，profile-specific extensions 應維持在 protocol core 之外

### Current Status

早期部分完成。

已在進行中或部分完成：

1. Simulator topology 與 report scaffolding
2. 用於 report inspection、listing、stats 與 diffing 的 CLI workflows
3. 可為窄版 resolved-state merges 產出可 replay patch operations 的保守型 local merge-authoring workflow
4. `mycel-core` 的 wire-envelope parsing、payload validation、RFC 3339 timestamp checks、signature verification、sender identity checks，以及 minimal message set 的 inbound session sequencing

仍缺少或未完成：

1. 將 `OBJECT` body 衍生的 hash 與 object-ID 重算真正接進主要 incoming verification path
2. Object fetch 與 sync state machine
3. Snapshot-assisted catch-up 與 capability-gated optional message handling
4. Production replication behavior
5. App-layer runtime support

### 本 phase 的 milestones

#### M4: Wire Sync and Peer Interop

重點：

1. canonical wire envelope
2. minimal message set
3. peers 間的 end-to-end sync
4. 在 indexing 前先做 verified object ingestion

完成門檻：

1. `HELLO`、`MANIFEST`、`HEADS`、`WANT`、`OBJECT`、`BYE`、`ERROR` 可端到端運作
2. peers 可以完成 minimal 的 first-time 與 incremental sync flow
3. fetched objects 在 storage 與 exposure 前先完成驗證
4. interop fixtures 與 simulator coverage 包含 sync success 與 negative sync cases

目前判讀：

`mycel-core` 已有 early groundwork：canonical envelope parsing、payload shape validation、RFC 3339 timestamp enforcement、通用 wire signature verification、sender checks、對 `HELLO`、`MANIFEST`、`HEADS`、`WANT`、`OBJECT`、`BYE`、`ERROR` 的 inbound sequencing/head-tracking、reachability gating、store-backed session bootstrap，以及 `OBJECT` body 衍生的 hash / `object_id` 驗證。現在也已有 transcript-backed `sync pull` path 與 CLI entry point，可證明 first-time 與 incremental 的 verify/store flow。仍缺的是 peer-driven 的端到端 sync driver、capability-gated optional flows，以及 production replication behavior。

Implementation anchors：

1. Crates:
   `crates/mycel-core`
   `crates/mycel-sim`
   `apps/mycel-cli`
2. Key files:
   `crates/mycel-core/src/wire.rs`
   `crates/mycel-core/src/signature.rs`
   `crates/mycel-sim/src/run.rs`
   `crates/mycel-sim/src/model.rs`
   `crates/mycel-sim/src/manifest.rs`
   `sim/README.md`
   `WIRE-PROTOCOL.en.md`
   `PROTOCOL.en.md`
3. Useful commands:
   `cargo test -p mycel-core wire::`
   `cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json --json`
   `cargo run -p mycel-cli -- report inspect sim/reports/out/three-peer-consistency.report.json --events --json`
   `cargo run -p mycel-cli -- report diff <left> <right> --events --json`

#### M5: Selective App-Layer Expansion

重點：

1. 在 protocol core 之上的保守 profile growth
2. 只有在 first client 穩定之後才加入 selective app-layer support
3. 在 protocol 已支援的前提下，加入 authoring 與 merge-generation workflows

完成門檻：

1. app-layer additions 建立在穩定 core protocol behavior 之上
2. merge generation 能產出可 replay 的 patch operations
3. 除非理由明確，profile-specific logic 應維持在 protocol core 之外

目前判讀：

大多刻意延後。

Implementation anchors：

1. Design 與 spec files:
   `docs/design-notes/`
   `PROFILE.fund-auto-disbursement-v0.1.en.md`
   `PROFILE.mycel-over-tor-v0.1.en.md`
   `PROJECT-INTENT.md`
2. 這個 milestone 的關鍵規則：
   成熟功能應先成為 profiles 或 schemas，再變成 protocol-core work

## Cross-Cutting Priorities

以下優先事項適用於所有 phases：

1. 刻意維持第一個 client 的窄版範圍
2. 優先用 profiles 與 schemas，而不是頻繁擴大 protocol-core
3. 只要 tests 依賴，就維持 machine-readable CLI output 的穩定
4. 每當引入新 protocol rule 或 CLI contract，都補上 regression coverage
5. 維持 protocol state、governance state、與 local discretionary policy 的分離

## Immediate Priorities

近期最高價值的工作是：

1. 完成 `M1`，補齊 shared object-family coverage 與 shared canonical object mechanics
2. 以 replay、`state_hash` 與 store-rebuild foundations 開始 `M2`
3. 每落一條 protocol rule，就持續強化 interop fixtures 與 negative tests
4. 只有在 minimal core 穩定後，才把成熟 governance behavior 轉成 fixed reader-profile workflows

## 什麼會讓一個 Milestone 前進

通常只有在以下條件都成立時，milestone 才應前進：

1. core behavior 已存在於 `mycel-core` 或其他 shared implementation layer，而不只是 CLI glue
2. CLI 或 simulator surface 已以足夠穩定的形式暴露該行為，可供內部使用
3. fixtures 或 negative tests 已覆蓋這條新 rule 或 behavior
4. 這個變更是在縮窄 first-client path，而不是過早擴大 protocol scope

## 目前還不是目標的東西

目前 roadmap 不把以下項目視為近期目標：

1. rich editor UX
2. production network deployment
3. generalized app runtime
4. broad plugin systems
5. 由推測性 design notes 驅動的快速 protocol-core 擴張
