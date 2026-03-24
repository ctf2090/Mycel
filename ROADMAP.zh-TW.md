# Mycel Roadmap

狀態：整體進度已有明顯推進；implementation checklist 已拆成已關閉的 `M1` minimal-client gate 與持續追蹤中的 post-`M1` 後續清單。`M2` 在目前窄版 replay/storage/rebuild 範圍內已完成收口，因此現在的主線更明確地轉到 `M3` / `M4`；目前仍未完成的重點是更完整的治理狀態持久化、更完整的治理工具面、reader-facing profile ergonomics、最後的獨立 dual-role 收尾，以及 `M4` 尚未補齊的 peer interop session/capability/error-path proof；原先規劃的 production replication 子項則都已補上，也已新增常設的 messages-after-BYE session proof、`HEADS` 先於 `MANIFEST` 的 sync-root setup、`HEADS replace=true` 後 stale root/dependency rejection、unknown-sender 與 HELLO sender-identity mismatch rejection、explicit `ERROR`-only 與 unreachable `WANT` fault proofs，同時也把 per-document current-governance summaries 納入目前的 `M3` 基線

這份 roadmap 將目前 README 的優先順序、implementation checklist，以及 design-note 的 planning 指引，整理成 repo 層級的實作推進順序。

這份規劃刻意維持在窄版範圍：

- 先做第一個可互通客戶端
- 對協定核心變更保持保守
- 在擴大範圍前，先把成熟想法落成 profiles、schemas 與測試

## 目前位置

目前 repo 已具備：

- 持續成長中的 v0.1 protocol 與 wire-spec 文件集
- 適合做內部驗證與決定性模擬器工作流程的 Rust CLI
- `mycel-core` 對 object schema metadata、object-envelope parsing、replay-based revision verification、local object-store ingest/rebuild、persisted store indexes，以及 accepted-head inspection 的支援
- `mycel-core` 對早期 wire-envelope parsing、payload validation、通用 wire signature verification、sender mapping、minimal message set 的 inbound session sequencing/head-tracking、reachability gating，以及 store-backed session bootstrap 的支援
- transcript-backed sync-pull core、peer-store sync driver 與 CLI entry points，已具備 first-time 與 incremental verify/store coverage，並包含 capability-gated `SNAPSHOT_OFFER` 與 `VIEW_ANNOUNCE` flows
- 更集中化的 canonical hash 與 signed-payload helpers，已在 verification、replay `state_hash`、head/render 預先驗證、authoring，以及 wire-object identity checks 之間重用
- 早期 reader-plus-governance surfaces，涵蓋 accepted-head rendering、具名 fixed-profile selection，以及具備 editor-admission 感知的 inspect/render workflows
- `document`、`block`、`patch`、`revision`、`view`、`snapshot` 在 parser / verify / CLI 路徑更廣的 strictness-surface coverage、更完整的 `object inspect` warning surface、對 merge 與 cross-document revision edge 更強的 signature-edge 與 replay/verification smoke coverage、更清楚的 multi-hop ancestry replay failure context，以及 isolate 過的 validate-peer fixtures
- 以 `assert_cmd`、`predicates`、`tempfile` 與小範圍 `rstest` 建立的較可維護 CLI test base
- simulator fixtures、topologies、tests 與 reports，作為 regression coverage

目前 repo 尚未具備：

- 完整可互通的節點實作
- 完成的 object-authoring 與 storage-write path
- 端到端 wire sync
- 正式可上線的公開 CLI 或 app

## Roadmap 摘要

### 現在

目前主線是：

1. 維持 `M2` 在目前窄版 replay/storage/rebuild 範圍內的已關閉狀態，並以已落地的 richer mixed content/metadata competing-branch rebuild/reporting proof 作為基線
2. 擴展 `M3` 的 reader-plus-governance 工作流程，但不要重新打開已經關閉的 minimal-client gate，同時把更廣的 governance persistence、更完整的 governance tooling、reader-facing profile ergonomics，以及最後的獨立 dual-role 收尾明確保留下來
3. 在目前規劃中的 production replication 子項都已補齊，且目前負向 proof 基線已包含常設的 messages-after-BYE rejection、`HEADS` 先於 `MANIFEST` 的 sync-root setup、`HEADS replace=true` 後的 stale root/dependency rejection、sender-validation faults、explicit `ERROR`-only failure，以及 unreachable `WANT` rejection 後，讓 `M4` 從 peer-store proof 繼續往剩餘的 peer interop session/capability/error-path coverage 推進

### 下一步

等窄版 core 穩定後，下一條主線會是：

1. 在目前 `view inspect` / `view list` / `view publish`、persisted relationship summaries，以及 per-document current-governance summaries 的 baseline 之上，補上更廣的 `M3` governance persistence、更完整的 governance tooling、reader-facing profile ergonomics，以及最後的獨立 dual-role 收尾
2. 補上超出目前 positive-path 與 optional-message proof set 的剩餘 `M4` session、capability 與 error-path interop proof
3. 等目前的 governance 與 interop baseline 更穩定後，再補 reader-facing 的 text reconstruction 與 presentation 打磨

### 之後

更後面的階段則會是：

1. 超出目前 peer-store-driven proof surface 的 canonical wire sync
2. 建立在已穩定 interop core 之上的端到端 peer replication
3. 建立在穩定 protocol core 與 sync baseline 之上的選擇性 app-layer expansion

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

目標：做出一個窄版的第一個客戶端，能夠以可重現的方式 parse、verify、store、replay，並 inspect Mycel objects。

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

Phase 1 exit criteria 現在已完全滿足。`IMPLEMENTATION-CHECKLIST.zh-TW.md` 中的 Ready-to-Build Gate 仍維持全綠（7/7），而 checklist 現在把這個 gate 保留為已關閉的歷史區塊，並另外追蹤仍在進行中的 post-`M1` 後續工作。

已完成：

1. Shared object schema metadata
2. Shared object-envelope parsing
3. Object inspection 與 verification
4. Replay-based revision verification 與 `state_hash` 檢查
5. Local object-store ingest、rebuild、persisted manifest indexing 與 query surfaces
6. Accepted-head inspection，包括 store-backed selector object loading
7. Internal validation 與 simulator harness CLI

8. malformed field-shape depth 與 semantic-edge strictness closure
9. Canonical JSON reuse 已在所有 wire-validation 路徑確認完成

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

Complete。shared parsing、更收斂的 canonical helper module、top-level core-version equality checks、保留路徑資訊的 nested parser field errors、更廣的 parser / verify / CLI strictness-surface coverage、更完整的 inspect-surface parity、更強的 replay dependency verification 與 sibling declared-ID determinism、直接涵蓋無效 sibling/parent dependency ID 與 signature 的 CLI smoke coverage、更清楚的 multi-hop ancestry failure context、isolate 過的 validate-peer fixtures、canonical reproducibility coverage、field-shape depth 與 semantic-edge closure、dual-role key closure，以及 canonical JSON reuse across wire-validation paths 現在都已存在。

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

沒有會阻擋 `M1` exit 的缺口。接下來的活躍工作重點已經轉到 `M2` / `M3`。

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

在目前窄版範圍內已關閉。replay-based verification、store rebuild、persisted indexes、可直接證明多文件 indexes 能在 index loss 後只靠 stored canonical objects 重建的 CLI smoke proof、窄版 store write path、初始的保守型 merge-authoring 工作流程、author 與 merge 工作流程中更廣的 document-level index reuse、供 sync 使用的 persisted `doc_heads` index、能保留 ancestry context 的 render/store verification，以及 richer mixed content/metadata competing-branch classification 搭配對應 CLI smoke coverage，現在都已具備；此外，較完整 metadata multi-variant merge case 的 rebuild-after-index-loss proof 也已落地。

主要剩餘缺口：

1. 目前沒有會阻擋這個窄版 `M2` milestone 的缺口。後續若要擴大 merge-authoring 能力，可作為更晚的 follow-up，而不是繼續算在 `M2` 收尾債務內。

Implementation anchors：

1. Crates:
   `crates/mycel-core`
   `apps/mycel-cli`
2. Key files:
   `crates/mycel-core/src/verify.rs`
   `crates/mycel-core/src/protocol.rs`
   `IMPLEMENTATION-CHECKLIST.zh-TW.md`
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

屬早期 partial progress，現在已在 deterministic selector path 之上具備 accepted-head rendering、具名 fixed-profile selection、更清楚的可用 profile 探索與 profile 錯誤回饋、具備 editor-admission 感知的 inspect/render behavior、`head inspect` / `head render` 的 `human` / `debug` 文字輸出模式、head inspection 裡的 bounded viewer score surfaces、透過 `view inspect` 與 `view list` 曝露的 persisted governance relationship summaries，以及透過 `view current` 提供的 per-document current-governance summaries；`M3` 仍未完成，主要剩下更廣泛的 governance persistence、超出目前 inspect/list/publish base 的 governance tooling、超出這一輪初步打磨的 reader-facing profile ergonomics，以及最後的獨立 dual-role 角色指派收尾。

已在進行中或部分完成：

1. Accepted-head inspection
2. 以 typed arrays 呈現的 structured decision detail
3. accepted-head inspection 的 store-backed selector object loading
4. 可從 persisted store state 或 explicit bundle objects 產生 accepted-head render output
5. 為 accepted-head inspection 與 render workflows 提供具名 fixed-profile selection，並補上更清楚的可用 profile 摘要與對稱的 profile 錯誤回饋
6. 在具名 profile 與 store-backed 路徑中，提供具備 editor-admission 感知的 accepted-head inspect/render behavior
7. 為 `head inspect` / `head render` 提供獨立的 `human` 與 `debug` 文字輸出模式，讓高階決策摘要與 debug trace 細節分層呈現
8. 提供獨立於 reader-facing `head` commands 的 `view inspect` / `view list` / `view publish` governance workflows，並具備 listing filter、sort、time window、grouped summary 與 projection modes
9. persisted governance reverse indexes，支援依 maintainer、profile 與 document 反查 view
10. 透過 `view inspect` 與 `view list` 呈現的 persisted governance relationship summaries
11. 透過 `view current` 呈現的 per-document current-governance summaries
12. simulator 與 validation workflows，涵蓋 peer、topology、test 與 report 範圍
13. head inspection 中的 bounded viewer score channels，包括 typed signal summaries、anti-Sybil gating、challenge review/freeze pressure，以及 fixture-backed coverage

主要剩餘缺口：

1. 超出 selector、reverse view indexes 與 replay inputs 的更廣泛 governance-state persistence
2. 超出目前初始 filtered / sorted / projected `view` inspection / listing / publication workflow 的專用 governance surfaces
3. 超出這一輪初步打磨的最小具名 fixed-profile surface 的 reader-facing profile ergonomics
4. 後續可與 wire / sync 對齊的 governance-state tooling
5. mixed-role 與 shared-key case 的最終 editor-maintainer / view-maintainer 獨立角色指派收尾，以及之後若要超出目前 head-inspect-local bundle surface，還需要哪些更廣泛的 governance persistence 或 governance-tooling 決策

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
3. 對支援 profiles 提供 capability-gated 的 `SNAPSHOT_OFFER` 與 `VIEW_ANNOUNCE` 支援
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

`mycel-core` 已有 early groundwork：canonical envelope parsing、payload shape validation、RFC 3339 timestamp enforcement、通用 wire signature verification、sender checks、對 `HELLO`、`MANIFEST`、`HEADS`、`WANT`、`OBJECT`、`BYE`、`ERROR` 的 inbound sequencing/head-tracking、reachability gating、store-backed session bootstrap，以及 `OBJECT` body 衍生的 hash / `object_id` 驗證。現在也已有 `mycel-core` 內的 peer-store sync path、CLI entry points，以及 9 個 simulator scenarios 的 positive-path coverage，可在不把手寫 transcript 當成唯一整合表面的前提下，證明 first-time 與 incremental 的 verify/store flow；同時 capability-gated 的 `SNAPSHOT_OFFER` / `VIEW_ANNOUNCE` handling 也已透過 peer-store generation、fetch/store behavior 與 simulator proof 落地。`localhost-multi-process` 也已透過 `mycel sync stream | mycel sync pull --transcript -` 的 stdin/stdout pipe proof，確認目前 wire flow 可以跨真實 process boundary 運作。Re-sync 冪等性也已經補上 proof：reader 已是最新狀態時，再跑一次 sync 會得到零次新寫入。Depth-N incremental catchup 也已經補上 proof：位於 revision depth 2 的 reader 透過一次 HEADS/WANT pass 追上 depth-3 的 seed，且只抓取差異部分。Partial-doc selective sync 也已補上 proof：reader 只請求 seed 的部分文件時，仍可維持穩定 partial store，並只對所請求子集計算 accepted heads，與 PROTOCOL §8 的 partial replication 支援一致。現在已落地的負向 proof set 也明顯更廣：已涵蓋缺少 capability 時對 `SNAPSHOT_OFFER` / `VIEW_ANNOUNCE` 的拒收、在 `HELLO` 前對 `MANIFEST`、`HEADS`、`WANT`、`BYE`、`SNAPSHOT_OFFER`、`VIEW_ANNOUNCE` 的拒收、duplicate-`HELLO`、unknown-sender rejection、HELLO sender-identity mismatch rejection、explicit `ERROR`-only transcript failure、在 accepted sync roots 建立前對一般 `WANT`、snapshot `WANT`、announced-view `WANT` 的拒收、對 accepted sync roots 外的 unreachable `WANT` revision/object 拒收、在 accepted sync roots 建立前立刻送出 `OBJECT` 的拒收，以及常設的 messages-after-`BYE` rejection。剩餘缺口因此不再是「任何 session negative case 都沒補」，而是下一批更廣的 session/capability/error-path interop faults，例如 advertised-root / root-set 違規，或其他 post-`HELLO` protocol-state errors。剩下的則是 peer interop 的 session/capability/error-path coverage。

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

1. 以窄版切片持續擴張 `M3`，補上 governance persistence 與 reader-plus-governance 後續工作，同時不要重新打開已關閉的 minimal-client gate
2. 在目前追蹤的 production replication 子項已落地後，持續為 `M4` 補上更多 deterministic 的 session、capability 與 error-path interop proofs
3. 每當剩餘的規則或 follow-up slice 落地，就持續補強 interop fixtures 與 negative tests
4. 在後續工作持續落地時，維持目前已關閉的 `M2` proof surface 不被回歸破壞

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
