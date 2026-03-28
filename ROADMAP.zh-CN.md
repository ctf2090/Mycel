# Mycel 路线图

状态：major progress，已在 implementation checklist 拆成已关闭的 `M1` minimal-client gate 与活跃的 post-`M1` follow-up checklist 之后刷新；当前主线已明确转到 `M3` / `M4`，其中 `M3` 最窄的下一步是剩余的最终独立 dual-role 收尾，之后才是更广的 governance persistence、更完整的 governance tooling、reader-facing profile ergonomics，以及仍处于 design draft 的 viewer / editor-maintainer / view-maintainer 三角色 checks-and-balances 方向；剩余的 peer interop session/capability/error-path proof 也仍未完成，而当前规划中的 production replication 子项已补齐

这份路线图把当前 README 的优先级、implementation checklist 和 design-note 的 planning 指引整理成一份仓库级的构建顺序。

它刻意保持窄范围：

- 先做第一个可互操作客户端
- 对协议核心改动保持保守
- 在扩大范围之前，先把成熟想法落成 profiles、schemas 和 tests

## 当前所处位置

当前仓库已经具备：

- 持续增长中的 v0.1 protocol 和 wire-spec 文档集
- 适合做内部验证和确定性模拟器工作流的 Rust CLI
- `mycel-core` 对 object schema metadata、object-envelope parsing、replay-based revision verification、local object-store ingest/rebuild、persisted store indexes 和 accepted-head inspection 的支持
- 面向 `document`、`block`、`patch`、`revision`、`view`、`snapshot` 的更广 parser / verify / CLI strictness-surface coverage、更完整的 `object inspect` warning surface、对 merge 和 cross-document revision edge 更强的 signature-edge 与 replay/verification smoke coverage，以及隔离后的 validate-peer fixtures
- 基于 `assert_cmd`、`predicates`、`tempfile` 和小范围 `rstest` 的更易维护 CLI test base
- simulator fixtures、topologies、tests 和 reports，作为 regression coverage

当前仓库还不具备：

- 完整的可互操作节点实现
- 完整的 object-authoring 和 storage-write path
- 端到端 wire sync
- 正式可上线的公开 CLI 或 app

## 路线图摘要

### 现在

当前这条 lane 是：

1. 在已经关闭的 `M1` gate 之上，收掉 `M2` replay、rebuild、merge-authoring 和 narrow write path 的剩余收尾；当前焦点已收窄到 richer mixed content/metadata competing-branch classification 落地后的 replay/store fixture closure 与 reporting/doc alignment follow-up
2. 扩展 `M3` reader-plus-governance workflows，但不要重新打开已经关闭的 minimal-client gate，先收掉剩余的最终独立 dual-role 收尾，再把更广的 governance persistence、更完整的 governance tooling、reader-facing profile ergonomics，以及仍属 design draft 的 viewer / editor-maintainer / view-maintainer 三角色 checks-and-balances 方向明确保留下来
3. 在当前规划中的 production replication 子项都已补齐之后，让 `M4` 从 peer-store proof 继续推进到剩余的 peer interop error/session coverage

### 下一步

当窄版 core 稳定之后，下一条 lane 是：

1. 在当前 `view inspect` / `view list` / `view publish`、persisted relationship summaries 与 per-document current-governance summaries 的 baseline 之上，于最终独立 dual-role 收尾之后，补上更广的 `M3` governance persistence、更完整的 governance tooling、reader-facing profile ergonomics，以及仍属 design draft 的三角色 checks-and-balances follow-up
2. 补上超出当前 positive-path 与 optional-message proof set 的剩余 `M4` session、capability 与 error-path interop proof
3. 等当前 governance 与 interop baseline 更稳定后，再补 reader-facing 的 text reconstruction 与 presentation 打磨

### 之后

更后面的 lane 是：

1. canonical wire sync
2. 端到端 peer replication
3. 建立在稳定 protocol core 之上的选择性 app-layer expansion

## Planning Levels

这份路线图采用 design notes 已经暗示的 planning 分层：

1. `minimal`
2. `reader-plus-governance`
3. `full-stack`

后一层都假定前一层已经稳定。

## Milestones

这份路线图通过以下 milestones 追踪：

1. `M1` Core Object and Validation Base
2. `M2` Replay, Storage, and Rebuild
3. `M3` Reader and Governance Surface
4. `M4` Wire Sync and Peer Interop
5. `M5` Selective App-Layer Expansion

## Phase 1: Minimal

目标：达到一个窄版 first client，能够以确定性的方式 parse、verify、store、replay，并 inspect Mycel objects。

### Deliverables

1. 所有 v0.1 object families 的 shared protocol object model
2. canonical serialization、derived ID recomputation 和 signature verification
3. replay-based revision verification 和 `state_hash` 检查
4. 本地对象存储与可重建索引
5. 稳定的内部 CLI/API，可用于 validation、object verification、object inspection 和 accepted-head inspection
6. object 和 simulator validation 的 interop fixtures 及 negative tests

### Exit Criteria

1. 必需 object types 能稳定 parse 和 validate
2. canonical IDs 和 signatures 具有确定性
3. revision replay 能只依赖 stored objects 通过
4. fixed profile 下的 accepted-head selection 具有确定性
5. 本地存储可以只依赖 canonical objects 重建

### Current Status

Phase 1 exit criteria 现在已经完全满足。`IMPLEMENTATION-CHECKLIST.en.md` 中的 Ready-to-Build Gate 仍然保持全绿（7/7），而 checklist 现在把这个 gate 作为已关闭的历史区块保留，并另外追踪活跃的 post-`M1` follow-up work。

已完成：

1. Shared object schema metadata
2. Shared object-envelope parsing
3. Object inspection 和 verification
4. Replay-based revision verification 和 `state_hash` 检查
5. Local object-store ingest、rebuild、persisted manifest indexing 和 query surfaces
6. Accepted-head inspection，包括 store-backed selector object loading
7. Internal validation 和 simulator harness CLI

8. malformed field-shape depth 与 semantic-edge strictness closure
9. Canonical JSON reuse 已在所有 wire-validation 路径确认完成

### 本 phase 的 milestones

#### M1: Core Object and Validation Base

重点：

1. shared object schema 和 parsing
2. canonical object validation rules
3. object inspection 和 verification tooling
4. interop fixtures 和 negative validation coverage

完成门槛：

1. 所有必需的 v0.1 object families 都能 parse 到 shared protocol layer
2. derived IDs 能稳定地重新计算
3. 必需的 signature rules 能一致执行
4. CLI 和 tests 能稳定暴露 validation 与 verification surfaces，供内部 workflows 使用

当前判断：

Complete。shared parsing、更收敛的 canonical helper module、top-level core-version equality checks、保留路径信息的 nested parser field errors、更广的 parser / verify / CLI strictness-surface coverage、更完整的 inspect-surface parity、更强的 replay dependency verification 与 sibling declared-ID determinism、直接覆盖无效 sibling/parent dependency ID 与 signature 的 CLI smoke coverage、更清楚的 multi-hop ancestry failure context、隔离后的 validate-peer fixtures、canonical reproducibility coverage、field-shape depth 与 semantic-edge closure、dual-role key closure，以及 canonical JSON reuse across wire-validation paths 现在都已经存在。

当前仓库里已经可以看到：

1. shared schema metadata
2. shared object-envelope parsing
3. shared canonical JSON、derived-ID recomputation 和 signed-payload helpers
4. object inspection 和 verification
5. 面向已支持 object families 的 protocol-level typed parsing，包括 `document`、`block`、`patch`、`revision`、`view`、`snapshot`
6. shared JSON loading 中的 duplicate-key rejection 和 unsupported-value rejection
7. IDs、signed payloads 和 signatures 的 canonical round-trip 及 reproducibility coverage
8. internal validation 和 simulator harness coverage

主要剩余缺口：

没有会阻挡 `M1` exit 的缺口。接下来的活跃工作重点已经转到 `M2` / `M3`。

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

建议的 build order：

1. 先在 `crates/mycel-core/src/protocol.rs` 里完成所有必需 object families 的 shared protocol parsing coverage
2. 把 canonical object mechanics 移到 shared protocol-level helpers，而不是只放在 `crates/mycel-core/src/verify.rs`
3. 扩展 `crates/mycel-core/src/verify.rs`，让它对每个已支持 object family 都消费这些 shared helpers
4. 在扩展 CLI surface 之前，先加深 `mycel-core` tests，让 object-rule regressions 在 CLI 层以下就被抓到
5. 只有在 shared core 稳定后，才扩展 CLI 和 simulator-facing validation coverage

第一批 implementation batch：

在当前仓库状态中已经完成：

1. `document` 和 `block` logical-ID handling 的 typed parsing coverage，位于 `crates/mycel-core/src/protocol.rs`
2. `patch`、`revision`、`view`、`snapshot` derived-ID fields 的 typed parsing coverage，位于 `crates/mycel-core/src/protocol.rs`
3. 从 verification-only ownership 中抽出的 shared protocol-level canonical JSON、derived-ID recomputation 和 signed-payload helpers
4. `crates/mycel-core/src/verify.rs` 已经对所有已支持 object families 消费 shared typed parsing 和 canonical helpers
5. `mycel-core` tests 已覆盖 malformed object type、missing signer fields、wrong derived-ID fields、duplicate keys、unsupported values 和 malformed field-shape cases，然后才继续扩展 CLI behavior

这一批的具体 completion check：

已完成：

1. `protocol.rs` 通过单一 shared parsing layer 理解所有当前已支持的 object families。
2. `verify.rs` 不再持有 canonical object mechanics 的唯一实现。
3. `cargo test -p mycel-core` 直接覆盖 shared protocol helpers 和 object-family edge cases。
4. 现有 `object inspect` 和 `object verify` CLI contract 仍然通过，不需要 CLI-only fallback logic。

#### M2: Replay, Storage, and Rebuild

重点：

1. replay-based revision verification
2. `state_hash` recomputation
3. local object-store indexing
4. store rebuild 和 recovery workflows
5. 初始的 object-authoring 与 storage-write path

完成门槛：

1. revisions 能从 stored objects 中确定性地 replay
2. replay 过程中会重新计算并验证 `state_hash`
3. indexes 可以只依赖 canonical objects 重建
4. 第一个 client 至少具备窄版 object creation 和 write path

当前判断：

已经大幅展开，但仍未完成。replay-based verification、store rebuild、persisted indexes、窄版 store write path、初始的保守型 merge-authoring workflow、author 与 merge workflow 中更广的 document-level index reuse、persisted `doc_heads` index for sync，以及能保留 ancestry context 的 render/store verification 都已经存在，但这个 milestone 仍未达到可关闭状态。

主要剩余缺口：

1. 保守型 merge authoring 现在已覆盖基本 move/reorder、insert/delete 组合、reparent 到新引入 parent 的 case、简单的 composed parent-chain reparenting、更广的初步 nested structural matrix、manual-curation-required 的 nested parent-choice / sibling-choice / composed-branch placement smoke proof、更细的 direct/anchor-based placement reasons，以及 richer mixed content/metadata competing-branch classification 与对应 CLI smoke coverage；这个 milestone 现在主要剩下 final replay/store closure 与 reporting/doc alignment

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
   replay 和 `state_hash` logic 很可能会先落在 `crates/mycel-core`
   storage-write 和 rebuild entry points 很可能需要新文件或新模块，而不是更多 CLI-only glue
4. Useful commands:
   `cargo test -p mycel-core`
   `cargo run -p mycel-cli -- validate fixtures/object-sets/minimal-valid/fixture.json --json`

建议的 build order：

1. 先在 `crates/mycel-core` 落 replay primitives，再开始任何新的 storage-writing CLI flow
2. 将 deterministic `state_hash` recomputation 建立在 replay 之上，而不是做成独立的孤立工具
3. 等 replay output 稳定后，再定义 minimal local store 和 rebuild model
4. 只有在 replay 和 rebuild semantics 稳定后，才加入窄版 object builder 和 storage-write path
5. 最后再暴露 CLI 或 API entry points，让它们建立在 shared replay 和 storage logic 之上，而不是发明平行行为

## Phase 2: Reader-Plus-Governance

目标：在 deterministic accepted-head behavior 和 governance-aware reading state 的基础上，加入可用的 reader-oriented client layer。

### Deliverables

1. 作为 governance signal input 的 verified View ingestion
2. fixed reader profiles 的稳定 accepted-head selection
3. 从 replayed revision state 进行 reader-first text rendering
4. reader workflows 和 governance publication workflows 的清晰分离
5. 可 inspect accepted heads、views 和 governance decision detail 的 CLI/API 支持

### Exit Criteria

1. 固定 reader profile 在重复运行下能产出稳定 accepted heads
2. governance inputs 与 discretionary local policy 清晰分离
3. reader 可以从 stored objects reconstruct 并 inspect accepted text state
4. decision summaries 和 typed arrays 已经稳定到足以供 tooling 和 tests 使用

### Current Status

属于早期 partial progress，但已经不再局限于 fixture-only 的 head inspection。

已经在进行中或部分完成：

1. Accepted-head inspection
2. 以 typed arrays 表示的 structured decision detail
3. accepted-head inspection 的 store-backed selector object loading
4. 围绕 peer、topology、test 和 report 范围的 simulator 与 validation workflows

仍然缺少或不完整：

1. 完整 reader rendering path
2. View publication workflow
3. 稳定 reader-facing profile selection surface
4. 独立的 governance retrieval 与 inspection surfaces，不再局限于 head inspection

### 本 phase 的 milestones

#### M3: Reader and Governance Surface

重点：

1. verified View ingestion
2. fixed-profile accepted-head selection
3. reader-first text reconstruction
4. reader inspection workflows 与 governance publication workflows 的清晰分离

完成门槛：

1. 固定 reader profile 在重复运行下能产出确定性的 accepted heads
2. governance data 的存储和消费与本地 discretionary policy 分离
3. 可以从 stored objects 中渲染或检查重建后的 accepted text
4. reader-facing CLI 或 API surfaces 已经稳定到足以重复用于内部工作

当前判断：

处于早期 partial progress，现在已经有了从 persisted store state 到 reader inspection 的初始桥接，并且已经出现 bounded viewer score surfaces；不过 `M3` 仍然还缺更广的治理持久化、更完整的治理工具面、reader-facing profile ergonomics、最终独立 dual-role 角色指派收尾，以及把仍属 design draft 的 viewer / editor-maintainer / view-maintainer 三角色 checks-and-balances 方向更清楚地反映到路线图上。

当前仓库里已经可以看到：

1. accepted-head inspection
2. typed arrays 中的 structured decision detail
3. accepted-head inspection 的 store-backed selector object loading
4. accepted-head rendering、具名 fixed-profile selection，以及更清楚的 profile 发现与错误反馈
5. 具备 editor-admission 感知的 inspect/render 行为
6. `view inspect` / `view list` / `view publish` 这些与 reader-facing `head` commands 分离的治理工作流
7. persisted governance reverse indexes、relationship summaries，以及 `view current` 中的 per-document current-governance summaries
8. head inspection 中的 bounded viewer score channels，包括 typed signal summaries、anti-Sybil gating 与 challenge review/freeze pressure
9. 围绕 peer、topology、test 和 report 范围的 simulator 与 validation workflows

主要剩余缺口：

1. 更广泛的 governance-state persistence，超出当前 reverse governance indexes、selector 与 replay inputs
2. 超出当前 `view` inspection / listing / publication 基线的更完整治理检索与发布 surfaces
3. 超出当前最小具名 fixed-profile surface 的 reader-facing profile ergonomics
4. 可在后续与 wire / sync 对齐的 governance-state tooling
5. mixed-role 与 shared-key case 的最终 editor-maintainer / view-maintainer 独立角色指派收尾
6. 在路线图层级更清楚表达仍属 design draft 的 viewer / editor-maintainer / view-maintainer 三角色 checks-and-balances 方向，包括 bounded viewer participation 与 proposal / ratification / public objection 的明确分离，但暂不把所有 draft 中的 escalation 或 anti-Sybil 细节直接提升为路线图承诺

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
   `cargo test -p mycel-cli head_inspect`

## Phase 3: Full-Stack

目标：从 local verification 与 governed reading 扩展到 interoperable replication、更丰富的 profiles，以及选择性的 app-layer support。

### Deliverables

1. Canonical wire envelope implementation
2. `HELLO`、`MANIFEST`、`HEADS`、`WANT`、`OBJECT`、`BYE`、`ERROR`
3. 对已支持 profiles 的可选 `SNAPSHOT_OFFER` 和 `VIEW_ANNOUNCE`
4. peers 之间的 end-to-end sync workflow
5. 面向 local authoring tools 的 merge-generation profile support
6. 建立在稳定 protocol core 之上的选择性 app-layer profiles

### Exit Criteria

1. peers 之间的 minimal sync 能端到端成功
2. 收到的 objects 会在 indexing 和 exposure 之前先完成验证
3. merge generation 能产出可 replay 的 patch operations
4. profile-specific extensions 除非理由明确，否则应继续留在 protocol core 之外

### Current Status

大部分尚未开始。

已经在进行中或部分完成：

1. Simulator topology 和 report scaffolding
2. 用于 report inspection、listing、stats 和 diffing 的 CLI workflows

仍然缺少或不完整：

1. 真正的 wire implementation
2. Object fetch 和 sync state machine
3. Snapshot-assisted catch-up
4. Production replication behavior
5. App-layer runtime support

### 本 phase 的 milestones

#### M4: Wire Sync and Peer Interop

重点：

1. canonical wire envelope
2. minimal message set
3. peers 之间的 end-to-end sync
4. 在 indexing 前先完成 verified object ingestion

完成门槛：

1. `HELLO`、`MANIFEST`、`HEADS`、`WANT`、`OBJECT`、`BYE`、`ERROR` 能端到端工作
2. peers 能完成 minimal 的 first-time 和 incremental sync flow
3. fetched objects 在 storage 和 exposure 之前先完成验证
4. interop fixtures 和 simulator coverage 包含 sync success 和 negative sync cases

当前判断：

实现尚未开始，但文档和 simulator 结构已经搭好 scaffold。

Implementation anchors：

1. Crates:
   `crates/mycel-sim`
   `apps/mycel-cli`
2. Key files:
   `crates/mycel-sim/src/run.rs`
   `crates/mycel-sim/src/model.rs`
   `crates/mycel-sim/src/manifest.rs`
   `sim/README.md`
   `WIRE-PROTOCOL.en.md`
   `PROTOCOL.en.md`
3. Useful commands:
   `cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json --json`
   `cargo run -p mycel-cli -- report inspect sim/reports/out/three-peer-consistency.report.json --events --json`
   `cargo run -p mycel-cli -- report diff <left> <right> --events --json`

#### M5: Selective App-Layer Expansion

重点：

1. protocol core 之上的保守 profile growth
2. 只有在 first client 稳定后才加入 selective app-layer support
3. 在 protocol 已经支持的前提下，加入 authoring 和 merge-generation workflows

完成门槛：

1. app-layer additions 建立在稳定 core protocol behavior 之上
2. merge generation 能产出可 replay 的 patch operations
3. profile-specific logic 除非理由明确，否则应留在 protocol core 之外

当前判断：

大体上是有意延后。

Implementation anchors：

1. Design 和 spec files:
   `docs/design-notes/`
   `PROFILE.fund-auto-disbursement-v0.1.en.md`
   `PROFILE.mycel-over-tor-v0.1.en.md`
   `PROJECT-INTENT.md`
2. 这个 milestone 的关键规则：
   成熟功能应先成为 profiles 或 schemas，然后再成为 protocol-core work

## Cross-Cutting Priorities

以下优先级适用于所有 phases：

1. 刻意保持第一个 client 的窄范围
2. 优先使用 profiles 和 schemas，而不是频繁扩大 protocol-core
3. 只要 tests 依赖，就保持 machine-readable CLI output 稳定
4. 每次引入新的 protocol rule 或 CLI contract，都补上 regression coverage
5. 保持 protocol state、governance state 和 local discretionary policy 的分离

## Immediate Priorities

近期最高价值的工作是：

1. 先收掉剩余的最终独立 dual-role 切片，再以窄切片继续扩展 `M3` 的 governance persistence、governance tooling、reader-facing profile ergonomics，以及仍属 design draft 的三角色 checks-and-balances follow-up，同时不要重新打开已关闭的 minimal-client gate
2. 继续为 `M4` 补上更多 deterministic 的 session、capability 与 error-path interop proofs
3. 每当剩余规则或 follow-up slice 落地，就持续补强 interop fixtures 与 negative tests
4. 在后续工作继续落地时，保持当前已收口的 `M2` proof surface 不被回归破坏

## 什么会推动一个 Milestone 前进

通常只有在以下条件都成立时，milestone 才应该前进：

1. core behavior 已存在于 `mycel-core` 或其他 shared implementation layer，而不只是 CLI glue
2. CLI 或 simulator surfaces 已经以足够稳定的形式暴露该行为，可供内部使用
3. fixtures 或 negative tests 已覆盖这条新 rule 或 behavior
4. 这个改动是在收窄 first-client path，而不是过早扩大 protocol scope

## 当前还不是目标的东西

当前路线图不把下面这些视为近期目标：

1. rich editor UX
2. production network deployment
3. generalized app runtime
4. broad plugin systems
5. 由推测性 design notes 驱动的快速 protocol-core 扩张
