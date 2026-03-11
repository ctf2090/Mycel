# Mycel 术语表（简体中文）

状态：working glossary

这份文档提供 Mycel 仓库范围内的简体中文术语对照和推荐写法。

目标：

- 统一协议核心、profile、设计说明和 README 的简体中文表达
- 降低 `document`、`accepted head`、`profile` 等术语的误读
- 让 `zh-CN` 文档采用中国大陆习惯的技术写法，而不是只把繁体改成简体

这份术语表不是规范来源。规范性定义仍以 [`PROTOCOL.en.md`](../PROTOCOL.en.md) 为准。

## 1. 使用原则

- 涉及协议核心的正式术语，优先沿用规格文件已经使用的表达。
- 编写或修订 `zh-CN` 文档时，应优先使用中国大陆常见的技术文档写法，而不是机械转换字形。
- 如果中文直译容易引起误解，可以保留英文并补一小段中文说明。
- 字段名、enum 值、对象类型名在数据结构里保留英文；正文可以用中文解释。
- `accepted` 不应直接翻成“共识”，因为 Mycel 不要求全网单一共识。
- `zh-CN` 和 `zh-TW` 是两个本地化文档面，不互相视为机械转换版本。

## 2. Core Protocol 术语

| English term | 建议中文 | 补充说明 |
| --- | --- | --- |
| protocol core | 协议核心 | 指 Mycel 最底层的可验证对象和规则，不包含应用层语义。 |
| document | Document / 文档 | 在 Mycel 里是“一条可长期演化、可重放的对象历史”，不一定是传统意义上的文档文件。 |
| block | 区块 / 段落区块 | v0.1 的主要操作单元。 |
| patch | 修改 | 一次对 document state 的修改。 |
| revision | 修订 | 表示某个可验证状态，不只是编辑记录。 |
| view | 治理 View | 带签名的治理信号，用来导出 accepted head。 |
| snapshot | 快照包 | 某个时间点的打包状态。 |
| logical ID | 逻辑 ID | 例如 `doc_id`、`block_id`，属于状态内部稳定引用，不是内容哈希。 |
| canonical object ID | canonical object ID / 内容寻址对象 ID | 例如 `patch_id`、`revision_id`、`view_id`、`snapshot_id`。 |
| replay | replay / 重放验证 | 根据历史对象重建状态，并检查其正确性。 |
| state hash | `state_hash` / 状态哈希 | 从 canonical state 导出的可复现哈希。 |
| head | head | 建议保留英文；表示某条 document 历史当前没有子孙的修订端点。 |
| accepted head | accepted head / 已采用 head | 在固定 profile 下被导出的默认 head。 |
| accepted reading | 默认阅读结果 | 指 reader 在固定 profile 下默认采用的阅读结果。 |
| eligible head | eligible head / 合格 head | 满足 selector 前置条件、可以继续参与 accepted-head 选择的 head。 |
| selector | selector / 选择规则 | 用来在合法 heads 之间导出 accepted head 的规则。 |
| selector epoch | `selector_epoch` / 选择 epoch | selector 计算上下文的一部分。 |
| view maintainer | View 维护者 | 发布 View 治理信号的维护角色。 |
| reader client | 阅读客户端 | 展示 document family 并导出 accepted reading 的客户端。 |

## 3. Profile 与治理术语

| English term | 建议中文 | 补充说明 |
| --- | --- | --- |
| profile | profile / 规则集 | 在正式技术语境里建议保留 `profile`。 |
| fixed profile | 固定 profile | 指不能被本地临时偏好随意改动的规则集。 |
| profile-governed | 由 profile 决定 | 强调结果来自固定规则，而不是自由裁量。 |
| policy | policy / 策略约束 | 可以是 profile 的一部分，也可以是一组更具体的执行条件。 |
| policy bundle | policy bundle / 策略包 | 一组共同生效的策略条件。 |
| accepted-state derivation | accepted-state 推导 | 从已验证对象和固定规则导出可采用状态。 |
| governance signal | 治理信号 | 例如 View 里的签名声明。 |
| non-discretionary | 非自由裁量 | 指客户端不应按本地偏好随意选择 accepted head。 |

## 4. Replication 与实现术语

| English term | 建议中文 | 补充说明 |
| --- | --- | --- |
| replication | 复制 | 指对象在 peers 之间被传递和保存。 |
| peer | peer / 对等节点 | 建议保留 `peer`，必要时补中文。 |
| ingest | 导入 | 对象进入本地存储。 |
| rebuild | 重建 | 根据已知对象重新构建状态或索引。 |
| fixture | fixture / 测试样例 | 仓库里用于确定性验证的固定数据。 |
| simulator | simulator / 模拟器 | 用于测试 peer / topology / reports 的模拟层。 |
| negative validation | 负向验证 | 确认错误样例会被正确拒绝。 |
| deterministic | 确定性 | 输入相同时结果可复现。 |

## 5. App-layer 常用术语

| English term | 建议中文 | 补充说明 |
| --- | --- | --- |
| app layer | 应用层 | 位于协议核心之上，承载领域语义。 |
| record family | 记录族 | 一组相关 document families 或 object streams。 |
| runtime | runtime / 运行时环境 | 指执行外部效果或本地判断的运行环境。 |
| effect layer | 外部效果层 | 明确表示外部观察、支付、通知等副作用。 |
| consent profile | consent profile / 同意规则集 | 用户事前授权条件。 |
| session record | 会话记录 | 一次有边界的执行或观测摘要。 |
| derived event | 导出事件 | 从 session 或其他 evidence 摘要出来的高层事件。 |
| intent | 意图 | 系统准备执行某个动作前的可验证中间状态。 |
| pledge | 承诺 | 尚未实际结算的承诺或待确认状态。 |
| receipt | 回执 / 收据 | 外部效果完成或失败后的可审计记录。 |
| dispute | 争议 | 针对某个意图、结算或状态提出的异议。 |
| revoke | 撤销 | 取消已有授权。 |
| pause | 暂停 | 暂时停用，但不表示永久撤销。 |

## 6. 建议避免的说法

下面这些表达容易造成误解，建议避免：

- 把 `accepted head` 写成“全网共识版本”
- 把 `document` 直接理解成“一篇文章”或“一个文件”
- 把 `profile` 写成“用户个人偏好”
- 把 `View` 写成“界面”或普通 UI view
- 把 `replay` 写成单纯的“动画回放”

## 7. 推荐短句

如果要用简体中文快速介绍 Mycel，建议优先使用这几句：

- Mycel 是一个用于可验证文本历史、按规则导出的默认阅读结果和去中心化复制的协议。
- 所谓“默认采用版本”不是全网共识，而是在固定 profile 规则下，从已验证对象推导出的结果。
- `Document` 在 Mycel 里是一条可长期重放的对象历史，不一定等于传统文档文件。
- 应用层语义应该放在 profile 和 applications 里，不应该写死进协议核心。
