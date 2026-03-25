# 规划同步计划

状态：当前有效的 planning surfaces 同步工作约定

这份文档定义 Mycel 如何同步以下几类表面：

- 仓库级 planning Markdown，尤其是 `ROADMAP.*` 与 `IMPLEMENTATION-CHECKLIST.*`
- GitHub Issues
- GitHub Pages 上的 planning 摘要页面，包括各语言入口页

它的目的，是避免权威构建顺序、可执行任务池和公开进度页之间发生漂移。

## 0. Sync 名词定义

请固定使用以下名词：

- `sync doc`：只同步 Markdown。包括 `ROADMAP.*`、`IMPLEMENTATION-CHECKLIST.*`、`docs/PROGRESS.md`，以及相关 README 文案。
- `sync web`：只同步 GitHub Pages。包括 `pages/progress.html` 与各语言 landing pages 中不属于 issue 入口的 HTML 摘要文案。
- `sync issue`：只同步 GitHub Issues。
- `sync plan`：完整同步，也就是 `sync doc` + `sync web` + `sync issue`。

## 1. 适用范围

这份计划适用于：

- [`ROADMAP.md`](../ROADMAP.md)
- [`ROADMAP.zh-CN.md`](../ROADMAP.zh-CN.md)
- [`ROADMAP.zh-TW.md`](../ROADMAP.zh-TW.md)
- [`IMPLEMENTATION-CHECKLIST.en.md`](../IMPLEMENTATION-CHECKLIST.en.md)
- [`IMPLEMENTATION-CHECKLIST.zh-CN.md`](../IMPLEMENTATION-CHECKLIST.zh-CN.md)
- [`IMPLEMENTATION-CHECKLIST.zh-TW.md`](../IMPLEMENTATION-CHECKLIST.zh-TW.md)
- [`docs/PROGRESS.md`](./PROGRESS.md)
- [`pages/progress.html`](../pages/progress.html)
- GitHub Issues，尤其是 `ai-ready` 类型任务

这份计划不适用于：

- 不会改变实现顺序或 closure 状态的 protocol wording 微调
- 纯视觉性的首页或 support page 调整
- 不影响项目 planning 状态的 issue triage

## 2. Source of Truth 顺序

当不同表面彼此不一致时，采用以下权威顺序：

1. `ROADMAP.md`、`ROADMAP.zh-CN.md` 和 `ROADMAP.zh-TW.md`
2. `IMPLEMENTATION-CHECKLIST.*`
3. GitHub Issues
4. `docs/PROGRESS.md`
5. `pages/progress.html`
6. landing page 或 support page 上的摘要说法
解释方式：

- `ROADMAP.md`、`ROADMAP.zh-CN.md` 和 `ROADMAP.zh-TW.md` 共同拥有 milestone 顺序、phase 边界和 build sequence 的权威性；如果三者暂时不同步，以英文版为最终校对基准，并应尽快补齐其余语言。
- `IMPLEMENTATION-CHECKLIST.*` 拥有 section-level closure 状态和 readiness gate 的权威性。
- GitHub Issues 代表剩余缺口的可执行切片。
- `docs/PROGRESS.md` 和 `pages/progress.html` 是派生摘要，不得自行发明项目状态。

## 3. 各表面的角色

### 3.1 `ROADMAP.*`

用 `ROADMAP.md`、`ROADMAP.zh-CN.md` 和 `ROADMAP.zh-TW.md` 回答：

- 我们现在在哪个 phase
- 下一条 lane 是什么
- milestone 顺序是什么
- 当前 lane 刻意排除了哪些东西

在以下情况应更新 `ROADMAP.*`：

- milestone 重心发生变化
- 仓库从一个 phase 边界移动到另一个
- 当前 lane 的主要缺口已经发生实质变化

### 3.2 `IMPLEMENTATION-CHECKLIST.*`

用 checklist 回答：

- 什么已经实现
- 什么仍未完成
- 哪些是 partial，哪些可以 close
- 哪些 readiness gate 仍被卡住

在以下情况应更新 checklist：

- 某个具体实现项完成
- 原本 open 的项目变成 partial 或可 close
- 某个 readiness gate 的状态发生变化

### 3.3 GitHub Issues

用 GitHub Issues 回答：

- 接下来有哪些窄而可执行的工作
- 哪些 checklist gaps 已被切成任务
- 哪些工作可以委派给 bot 或并行 contributor

Issues 不应取代 roadmap 或 checklist 的项目状态，它们只应反映这些状态。

### 3.4 Pages 进度面

用 `docs/PROGRESS.md` 和 `pages/progress.html` 回答：

- 读者应该快速理解什么
- 当前活跃的是哪条 milestone lane
- 哪些 checklist sections 是 mostly done、partial 或 not started

Pages 必须保持 summary-first。它们应该压缩 planning 状态，而不是定义 planning 状态。

## 4. 同步规则

### 4.1 如果 milestone 状态发生变化

按这个顺序更新：

1. `ROADMAP.md`
2. `IMPLEMENTATION-CHECKLIST.*`
3. `docs/PROGRESS.md`
4. `pages/progress.html`
5. 关闭或新增相关 GitHub Issues

示例：

- `M1` 从 “late partial” 进入“足以开始收尾”
- `M2` 成为更明确的 active lane

### 4.2 如果某个 checklist 项关闭，但 phase 没变

按这个顺序更新：

1. `IMPLEMENTATION-CHECKLIST.*`
2. 相关 GitHub Issue 状态
3. 如果 section-level 状态变化，再更新 `docs/PROGRESS.md`
4. 如果公开摘要措辞发生变化，再更新 `pages/progress.html`

示例：

- `Implement snapshot parsing` 完成
- 但路线图的 active lane 没变

### 4.3 如果发现新的可执行缺口

按这个顺序更新：

1. 如果这个缺口是真实且持久的，先更新 `IMPLEMENTATION-CHECKLIST.*`
2. 如果它足够窄、可执行，再开 GitHub Issue
3. 除非 milestone 重心变化，否则不要更新 `ROADMAP.md`
4. 只有在 section 状态实质变化时才更新 progress 摘要

### 4.4 如果 issue triage 只改变执行形状

只更新：

1. GitHub Issues

以下情况不应更新 roadmap、checklist 或 pages：

- 项目底层状态没有变化
- 只是把工作拆得更细
- 只是改 labels 或 ownership，没有影响 closure 状态

### 4.5 如果 Pages 只是为了可读性调整

更新：

1. `docs/PROGRESS.md`
2. `pages/progress.html`

除非底层状态真的变化，否则不要动 roadmap 或 checklist。

## 5. GitHub Issue 对应规则

### 5.1 Issue 来源

每一张 planning-oriented 的实现 issue 都应对应到下列之一：

- 一个 checklist item
- 一个 checklist 子缺口
- 一个 milestone-close proof point

避免一张 issue 跨越多个无关 checklist sections。

### 5.2 建议的 issue metadata

每张 issue 应该包含：

- 它支撑的是哪个 checklist section 或 roadmap milestone
- start files
- acceptance criteria
- verification commands
- non-goals

对 bot-friendly 任务，建议使用：

- `ai-ready`
- `well-scoped`
- `tests-needed`
- `fixture-backed`
- 适用时加 `spec-follow-up`

### 5.3 Issue lifecycle

采用以下生命周期：

1. 缺口真实且可执行时开 issue
2. 只要 checklist 对应缺口仍未实质关闭，就保持 open
3. 当该 issue 的窄 acceptance criteria 完成时关闭
4. 如果更大的 checklist closure 仍未完成，就开 follow-up issues，而不是让原 issue 一直保持模糊

## 6. Pages 派生规则

### 6.1 `docs/PROGRESS.md`

这是公开 progress view 的 Markdown 摘要来源。

它应该：

- 重述 active lane
- 压缩 milestone 状态
- 压缩 checklist section 状态
- 回链到 roadmap 和 checklist 这两个权威来源

它不应该：

- 引入 `ROADMAP.md` 里不存在的 milestone 名称
- 在 checklist 没有对应依据时，把某个区域标成完成
- 推测 roadmap 尚未定义的未来 phase

### 6.2 `pages/progress.html`

这个文件应被视为 `docs/PROGRESS.md` 的展示层。

当 planning 状态发生变化时：

- 先更新 `docs/PROGRESS.md`
- 再更新 `pages/progress.html`

如果两者不一致，应把 HTML 改回与 Markdown 摘要一致，而不是反过来。

## 7. 更新节奏

### 7.1 事件驱动更新

在下列情况下，立即更新 planning surfaces：

- milestone 有实质推进
- checklist section 状态变化
- active implementation lane 发生变化

### 7.2 Commit-count refresh

使用：

```bash
`scripts/check-plan-refresh.py`
```

补充规则：

- `sync doc` 在 10 commits 时到门槛
- `sync issue` 在 10 commits 时到门槛
- `sync web` 在 20 commits 时到门槛

如果它报告 `due`，则在下一次 planning-sync batch 中更新被点名的表面：

- `ROADMAP.md`
- `ROADMAP.zh-TW.md`
- `IMPLEMENTATION-CHECKLIST.en.md`
- `IMPLEMENTATION-CHECKLIST.zh-TW.md`
- 对齐后的 GitHub Issues
- 属于 Markdown 的 planning 摘要面，比如 `docs/PROGRESS.md`，在 `sync doc` 到门槛时更新
- 对齐后的 GitHub Issues，在 `sync issue` 到门槛时更新
- GitHub Pages 上的 HTML 摘要面，比如 `pages/progress.html` 和各语言 landing pages 的非 issue 文案，在 `sync web` 到门槛时更新

即使没有任何单一变更强迫你更新，也要做这次 refresh。

## 8. 建议同步流程

对于一个有实质意义的 implementation batch：

1. 先落 code 和 tests
2. 判断 checklist 状态是否变化
3. 判断 roadmap 重心是否变化
4. 更新 GitHub Issues
5. 更新 `docs/PROGRESS.md`
6. 更新 `pages/progress.html`
7. 运行相关验证和 plan-refresh 检查

对于一个 `sync plan` 批次：

1. 按这个顺序扫描 handoff mailboxes：
   先扫 registry 中 `active` agents 声明的 mailbox paths
   再扫 registry 中 `paused` agents 声明的 mailbox paths
   接着扫 registry 中最近 `inactive`、但可能仍有 unresolved planning notes 的 mailbox paths
   最后才扫 `.agent-local/coding-to-doc.md`、`.agent-local/doc-to-coding.md` 这类 fallback shared mailboxes（如果存在）
2. 除非当前 mailbox 明确指向 archive 里仍未解决的条目，否则不要回头扫描 archived mailboxes
3. `doc` 必须运行 planning-refresh cadence checker（规划同步节奏检查工具）
4. 如果 `sync doc` 到门槛，refresh `ROADMAP.md`、`IMPLEMENTATION-CHECKLIST.*`、`docs/PROGRESS.md` 和相关 README 文案
5. 如果 `sync issue` 到门槛，对齐 GitHub Issues
6. 如果 `sync web` 到门槛，再更新 `pages/progress.html` 和各语言 landing pages 的非 issue HTML 摘要
7. 确认 GitHub Pages 上的 planning 摘要与刷新后的 roadmap/checklist/issues 状态一致

对于一个 `sync doc` 批次：

1. 按这个顺序扫描 handoff mailboxes：
   先扫 registry 中 `active` agents 声明的 mailbox paths
   再扫 registry 中 `paused` agents 声明的 mailbox paths
   接着扫 registry 中最近 `inactive`、但可能仍有 unresolved planning notes 的 mailbox paths
   最后才扫 `.agent-local/coding-to-doc.md`、`.agent-local/doc-to-coding.md` 这类 fallback shared mailboxes（如果存在）
2. 除非当前 mailbox 明确指向 archive 里仍未解决的条目，否则不要回头扫描 archived mailboxes
3. `doc` 必须运行 planning-refresh cadence checker（规划同步节奏检查工具）
4. refresh `ROADMAP.md`、`IMPLEMENTATION-CHECKLIST.*`、`docs/PROGRESS.md` 和相关 README 文案

对于一个 `sync web` 批次：

1. 按这个顺序扫描 handoff mailboxes：
   先扫 registry 中 `active` agents 声明的 mailbox paths
   再扫 registry 中 `paused` agents 声明的 mailbox paths
   接着扫 registry 中最近 `inactive`、但可能仍有 unresolved planning notes 的 mailbox paths
   最后才扫 `.agent-local/coding-to-doc.md`、`.agent-local/doc-to-coding.md` 这类 fallback shared mailboxes（如果存在）
2. 除非当前 mailbox 明确指向 archive 里仍未解决的条目，否则不要回头扫描 archived mailboxes
3. `doc` 必须运行 planning-refresh cadence checker（规划同步节奏检查工具）
4. 更新 `pages/progress.html` 和各语言 landing pages 的非 issue HTML 摘要

## 9. Anti-Drift 规则

不要让以下情况长期存在：

1. roadmap 说 lane 已经变化，但 progress page 还显示旧 lane
2. checklist 把某项标成完成，但相关 issue 没关，也没有 follow-up split
3. progress page 说某个区块 mostly done，但 checklist 仍然大多未勾
4. issue 标题漂移成 roadmap 还不支持的推测性工作
5. Pages 自己发明 roadmap 或 checklist 里没有的项目状态说法
6. landing page 的 contributor-entry 链接指向过期、已关闭或已不具代表性的 issue

## 10. 最低完成条件

当下面这些条件都成立时，可以认为 planning surfaces 已经同步：

- roadmap 的 milestone wording 符合当前 active lane
- checklist boxes 反映当前实现 closure
- open issues 对应真实剩余缺口
- `docs/PROGRESS.md` 的摘要与 roadmap/checklist 一致
- `pages/progress.html` 与 `docs/PROGRESS.md` 一致
- `pages/index.html` 和各语言 landing pages 中精选的 contributor-entry 链接仍然指向具代表性的 open starter issues

## 11. 当前针对 Mycel 的实务指引

现在请采用以下具体规则：

1. 把 `ROADMAP.md` 视为 milestone 和 lane 的权威来源
2. 把 `IMPLEMENTATION-CHECKLIST.*` 视为 closure 状态的权威来源
3. 把 open 的 `ai-ready` issues 视为 checklist gaps 的窄执行切片
4. 把 `docs/PROGRESS.md` 和 `pages/progress.html` 视为公开摘要层
5. 把 README 中的 contributor 指引视为 Markdown 文案；README 只需要指向 GitHub issue 列表，不在这里精选 starter issues
6. 把 `pages/index.html` 和各语言 landing pages 中精选的 contributor issue 链接视为公开的 curated issue 入口，并在 planning sync 时一起刷新

这样可以让 roadmap、implementation closure、task queue 和公开进度页保持一致，而不会让任何一个表面承担过多角色。
