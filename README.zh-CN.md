# Mycel

语言：简体中文 | [English](./README.md) | [繁體中文](./README.zh-TW.md)

Mycel 是一个以 Rust 实现为主的协议栈，用于可验证的文本历史、按规则导出的默认阅读结果，以及去中心化复制。

它面向的是这类以文本为核心的系统：

- 需要可重放验证的历史
- 需要带签名的治理信号
- 允许多个有效分支并存，而不是要求全网强制单一共识
- 默认阅读结果应由固定规则集（profile）导出，而不是来自临时的本地偏好

## 为什么是 Mycel

大多数协作工具通常会落在两种形态里：

- 由中心化平台维护可变状态
- 为代码协作或全局共识优化的分布式系统

Mycel 走的是另一条路。它把文本历史、默认阅读结果和去中心化复制看成彼此分离但可以互相配合的层。

因此，它更适合长期文本、评注系统、受治理的参考文本集合，以及其他以文字为核心的分布式工作流。

它想补上的，是中心化协作平台和全局共识区块链之间的那层空白：让历史可验证、默认阅读结果可由规则导出、对象可去中心化复制，同时不要求整个网络收敛成一个唯一真版本。

## 它的不同点

- 可验证历史：修订不是“看起来像有历史”，而是应该能被重放、检查和验证。
- 受规则约束的默认阅读结果：默认采用哪个版本，来自固定 profile 规则和已验证的 View 对象。
- 允许分叉：多个 head 可以同时有效，不需要伪装成全网只有一个真相。
- 中立的协议核心：领域语义应该放在 profile 和应用层，而不是写死进协议核心。

换句话说，所谓“默认采用版本”不是网络级共识，而是在某个固定 profile 下，从已验证对象推导出的结果。

## 它不是什么

Mycel 不是：

- 要求全局强制共识的区块链
- Git 的复制品
- 通用文件传输层

## 60 秒可以试什么

当前的 Rust CLI 还是内部验证和模拟器工具链，还不是正式可上线的 Mycel 客户端或节点。

如果你是从全新环境开始，先看 [`docs/DEV-SETUP.zh-CN.md`](./docs/DEV-SETUP.zh-CN.md)。

在仓库根目录执行：

```bash
cargo run -p mycel-cli -- info
cargo run -p mycel-cli -- validate fixtures/object-sets/minimal-valid/fixture.json --json
cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json --json
```

这三个命令分别会看到：

- `info`：仓库内部工作区和脚手架目录路径
- `validate`：对已提交测试样例的稳定验证输出
- `sim run`：模拟器测试流程的执行摘要，以及生成的报告路径

## 当前状态

- 协议阶段：`v0.1` 概念规格，并持续补充 profile 和 design-note 层
- 当前实现重点：收敛第一个客户端的边界、加强 replay 和 verification、稳定确定性的模拟器工作流
- 当前 CLI 边界：适合在本仓库内做验证、对象检查、对象校验、accepted-head 检查、报告检查和模拟器运行
- 尚未交付：正式可上线的节点行为、公开网络 wire sync，或完整的终端用户客户端

## 按目标阅读

当前 `zh-CN` 文档支持还是第一批入口覆盖，不是全量三语同步。

如果你想用简体中文先建立整体理解，建议按这个顺序读：

- 先理解 Mycel 想补上的空白：[docs/MYCEL-GAP.zh-CN.md](./docs/MYCEL-GAP.zh-CN.md)
- 再看本地化术语约定：[docs/TERMINOLOGY.zh-CN.md](./docs/TERMINOLOGY.zh-CN.md)
- 从全新环境开始时看：[docs/DEV-SETUP.zh-CN.md](./docs/DEV-SETUP.zh-CN.md)

如果你接下来要深入协议和实现，当前建议回到英文或繁体中文版：

- 协议核心：[PROTOCOL.en.md](./PROTOCOL.en.md)
- 传输规则：[WIRE-PROTOCOL.en.md](./WIRE-PROTOCOL.en.md)
- 实现顺序：[ROADMAP.md](./ROADMAP.md)
- 实现检查清单：[IMPLEMENTATION-CHECKLIST.en.md](./IMPLEMENTATION-CHECKLIST.en.md)

## 从这里开始参与贡献

如果你想先接一个范围窄的任务，可以先看这几个 issue：

- [#1 Reject duplicate JSON object keys in shared object parsing](https://github.com/ctf2090/Mycel/issues/1)
- [#3 Add malformed logical-ID coverage for document and block objects](https://github.com/ctf2090/Mycel/issues/3)
- [#4 Add snapshot derived-ID verification smoke coverage](https://github.com/ctf2090/Mycel/issues/4)

如果你想看更结构化的任务入口，可以直接浏览带有 `ai-ready` 和 `well-scoped` 标签的 issues。

## 第一批简体中文支持包含什么

当前先覆盖这些入口：

- [README.zh-CN.md](./README.zh-CN.md)
- [docs/MYCEL-GAP.zh-CN.md](./docs/MYCEL-GAP.zh-CN.md)
- [docs/DEV-SETUP.zh-CN.md](./docs/DEV-SETUP.zh-CN.md)
- [docs/TERMINOLOGY.zh-CN.md](./docs/TERMINOLOGY.zh-CN.md)

后续如果继续扩展，优先顺序应该是：

1. `ROADMAP.zh-CN.md`
2. `IMPLEMENTATION-CHECKLIST.zh-CN.md`
3. `PROTOCOL.zh-CN.md`
4. `WIRE-PROTOCOL.zh-CN.md`

## 许可证

本仓库采用 [MIT License](./LICENSE)，除非将来某个文件或目录另有说明。

关于贡献和许可证预期，请参考 [CONTRIBUTING.md](./CONTRIBUTING.md)。
