# Mycel v0.1 实现检查清单

状态：late partial progress，M1 parsing、parser / verify / CLI strictness coverage、更广的 inspect-surface parity、signature-edge 与 replay/verify smoke coverage、fixture isolation、test-foundation cleanup 和 canonical reproducibility core 已接近完成

这份清单把 v0.1 规格转成偏实现导向的构建计划，目标是一个最小但可互操作的客户端。

## 0. 构建目标

先锁定受限版 v0.1 客户端：

- 一个本地对象存储
- 一个固定的 network hash algorithm
- 只支持 canonical JSON
- 支持 `patch` / `revision` / `view` / `snapshot`
- 实现 `HELLO`、`MANIFEST`、`HEADS`、`WANT`、`OBJECT`、`BYE`、`ERROR`
- 只有在声明对应 capability 时才实现 `SNAPSHOT_OFFER` 和 `VIEW_ANNOUNCE`
- 基于 replay 的 revision 验证
- 确定性且 profile-locked 的 head selection
- 保守版 merge generation profile

可以延后：

- 丰富的 editor UX
- 高级 policy UI
- 自动 key discovery
- 非 JSON 编码
- 自定义 merge plugin

## 1. 仓库与构建设置

- [x] 选定一种实现语言和 package layout。
- [x] 为 network profile 固定一个 canonical hash algorithm。
- [x] 为 client profile 固定一组 signature algorithms。
- [ ] 加入一个可被 hash、signature 和 wire 共用的 canonical JSON 工具。
- [x] 加入 protocol examples 和 regression tests 的 fixture 加载机制。

## 2. 对象类型与 ID

- [x] 实现 `document` 解析，并把 `doc_id` 视为 logical ID。
- [x] 实现 `block` 解析，并把 `block_id` 视为 logical ID。
- [x] 实现带导出 `patch_id` 的 `patch` 解析。
- [x] 实现带导出 `revision_id` 的 `revision` 解析。
- [x] 实现带导出 `view_id` 的 `view` 解析。
- [x] 实现带导出 `snapshot_id` 的 `snapshot` 解析。
- [x] 拒绝任何内嵌 derived ID 与重算 canonical ID 不一致的内容寻址对象。
- [x] 在 shared parsing 和 verification 中，拒绝 typed object 的未知顶层字段和非法必需字段类型。
- [ ] 在最近 strictness-surface 与 replay/verify smoke 扩展后，完成剩余 malformed field-shape depth 和 semantic edge-case 的收尾。
- [ ] 把 editor-maintainer 和 view-maintainer 的角色分配拆开建模。

## 3. Canonical Serialization 与 Hashing

- [x] 把所有协议对象 canonicalize 成 UTF-8 JSON，并且不包含多余空白。
- [x] 强制 object key 按字典序排序。
- [x] 完整保留 array 顺序。
- [x] 拒绝 duplicate keys。
- [x] 拒绝 `null`、浮点数等不支持的值类型。
- [x] 重算 object ID 时省略 derived ID 字段和 `signature`。
- [ ] 对 `state_hash` 和 wire envelope signature 复用同一套 canonicalization 规则。

## 4. Signature 验证

- [x] 实现 object signature matrix。
- [x] 禁止 `document` 和 `block` 带 signature。
- [x] 要求 `patch`、`revision`、`view` 和 `snapshot` 必须带 signature。
- [x] 只有在 canonical ID 检查通过后才验签。
- [ ] 为所有 v0.1 wire message type 实现 envelope signature 验证。
- [ ] 拒绝任何未通过 profile 必需签名检查的对象或消息。

## 5. Patch 与 Revision Engine

- [ ] 实现 v0.1 patch operations：
- [x] `insert_block`
- [x] `insert_block_after`
- [x] `delete_block`
- [x] `replace_block`
- [x] `move_block`
- [x] `annotate_block`
- [x] `set_metadata`
- [x] 强制非 genesis patch 的 `base_revision` 等于 execution-base revision。
- [x] 支持 genesis sentinel `rev:genesis-null`。
- [x] 按数组顺序应用 revision 的 `patches`。
- [x] 把 `parents[0]` 视为唯一 execution base state。
- [x] 把 `parents[1..]` 视为 ancestry-only，除非内容被显式 patch operation 实体化。
- [x] 对每个接收的 revision 重新计算并验证 `state_hash`。
- [x] 保持 revision 发布权与 accepted-head governance weight 分离。

## 6. 本地状态与存储

- [x] 按 canonical `object_id` 存储所有接收的对象。
- [x] 维护 `doc_id -> revisions` 索引。
- [x] 维护 `revision_id -> parents` 索引。
- [x] 维护 `author -> patches` 索引。
- [x] 维护 `view_id -> governance signal contents` 索引。
- [x] 维护 `profile_id -> selected document heads` 索引。
- [ ] 将本地 transport 和 safety policy 与可复制的协议对象分开持久化。
- [x] 不让自由裁量的本地 policy 进入 active accepted-head 路径。
- [x] 支持只依赖 object store 就能重建 indexes。

## 7. Wire Protocol

- [ ] 实现 canonical wire envelope。
- [ ] 验证 `type`、`version`、`msg_id`、`timestamp`、`from`、`payload` 和 `sig`。
- [ ] 对 wire messages 强制 RFC 3339 时间格式。
- [ ] 实现 `HELLO`。
- [ ] 实现 `MANIFEST`。
- [ ] 实现 `HEADS`。
- [ ] 实现 `WANT`。
- [ ] 实现 `OBJECT`。
- [ ] 实现 `BYE`。
- [ ] 实现 `ERROR`。
- [ ] 只有在声明 `snapshot-sync` 时才实现 `SNAPSHOT_OFFER`。
- [ ] 只有在声明 `view-sync` 时才实现 `VIEW_ANNOUNCE`。
- [ ] 对每个 `OBJECT` 重新计算 `hash(body)`。
- [ ] 根据 `object_type` 和 `hash` 重建预期的 `object_id`。
- [ ] 拒绝任何内嵌 derived ID 与 envelope `object_id` 不一致的 `OBJECT`。

## 8. 同步流程

- [ ] 支持首次同步：`HELLO` -> `MANIFEST` / `HEADS` -> `WANT` -> `OBJECT`。
- [ ] 支持从更新后的 `HEADS` 做增量同步。
- [ ] 只按 canonical object ID 获取缺失对象。
- [ ] 先验证对象，再建立索引或暴露给 reader。
- [ ] 如果对方声明 snapshot，则支持 snapshot-assisted catch-up。
- [ ] 如果启用 `view-sync`，则支持抓取已公告的 views。
- [ ] 将抓回的 View objects 视为 governance signals，而不是用户偏好状态。

## 9. Views 与 Head Selection

- [x] 把已验证 `view` 对象作为 governance signals 存储，并与本地 transport/safety policy state 分开。
- [x] 按 `profile_id`、`doc_id` 和 `effective_selection_time` 分组 selector inputs。
- [x] 将 `profile_id` 解析为 active reader profile 的固定 `policy_hash`。
- [x] 精确实现 eligible heads 判定。
- [x] 只使用 `policy_hash` 相同且已验证的 View 对象作为 view-maintainer signals。
- [x] 精确实现 selector epoch 计算。
- [x] 实现规范中的 `selector_score`。
- [x] 实现规范中的 tie-break 顺序。
- [x] 输出或持久化最小 decision trace schema。
- [x] 不提供会改变 active accepted head 的自由裁量本地 policy controls。
- [ ] 如果支持多个固定 profiles，就必须显式列举，而不是允许 ad hoc local profiles。
- [x] 确保仅有 editor-maintainer 身份不会自动获得 selector weight。
- [ ] 如果支持 dual-role keys，必须分别验证 editor-maintainer 和 view-maintainer 的准入。

## 10. Merge Generation

- [x] 保持 revision 验证为 replay-based；不要要求接收端重新运行 merge generation。
- [ ] 为本地作者工具实现保守版 merge generation profile。
- [ ] 区分 `Auto-merged`、`Multi-variant` 和 `Manual-curation-required`。
- [ ] 把 merge 结果实体化成普通 patch operations。
- [ ] 拒绝用隐藏 merge metadata 代替显式状态变更。

## 11. CLI 或 API 入口

- [ ] 提供本地 init command 或 API。
- [x] 提供 object verification 工具。
- [ ] 提供 document creation 和 patch authoring 入口。
- [ ] 提供 revision commit 入口。
- [ ] 提供 sync pull 入口。
- [x] 提供 view inspection 或 head inspection 入口。
- [ ] 把 reader-facing accepted-head inspection 与 curator-facing View publication workflow 分开。
- [x] 让 head inspection 的 `decision_trace` 只保留高层摘要。
- [x] 把 maintainer、weight 和 violation 的机器可消费细节放进 `effective_weights[]`、`maintainer_support[]`、`critical_violations[]` 这类 typed arrays，而不是塞进 `decision_trace`。
- [x] 把 `decision_trace` 视为给人看的解释输出；把 typed arrays 视为供工具和测试依赖的稳定细节接口。
- [ ] 把 editor-maintainer revision publication 和 view-maintainer governance publication workflow 分开。
- [x] 提供 store-rebuild 或 reindex 入口，用于恢复。

## 12. Interop Test 最低门槛

- [ ] 加载所有规范性 example objects，并确认可以解析。
- [ ] 对 example `patch`、`revision`、`view` 和 `snapshot` 重新计算 derived IDs。
- [x] 至少对一个 single-parent revision 和一个 merge revision 重新计算 `state_hash`。
- [ ] 验证 example wire envelopes 和 `OBJECT` 验证行为。
- [x] 加入 hash mismatch、signature mismatch 和 invalid parent ordering 的 negative tests。
- [x] 加入 canonical serialization 的 round-trip test。
- [x] 加入只依赖存储对象重建 document state 的 replay test。

## 13. 可开始构建客户端的门槛

当下面这些条件都成立时，才可以把客户端视为 ready for first interoperable build：

- [ ] 所有必需 object types 都能解析并验证
- [x] canonical IDs 和 signatures 可重现
- [x] revision replay 和 `state_hash` 验证通过
- [ ] minimal wire sync 能端到端跑通
- [x] 确定性 head selection 能产出稳定结果
- [ ] merge generation 能产出有效且可 replay 的 patch operations
- [x] 本地 store 能只依赖 canonical objects 完整重建
