# 开发环境设置

这份文档提供从全新 checkout 到可以实际开发 Mycel 的最短路径。

建议配合阅读：

- [`README.zh-CN.md`](../README.zh-CN.md)
- [`CONTRIBUTING.md`](../CONTRIBUTING.md)
- [`BOT-CONTRIBUTING.md`](../BOT-CONTRIBUTING.md)
- [`RUST-WORKSPACE.md`](../RUST-WORKSPACE.md)

## 0. 需要的工具

必需工具：

- Rust toolchain manager：`rustup`
- Rust `stable` toolchain
- Rust components：`rustfmt`、`clippy`
- GitHub CLI：`gh`
- ripgrep：`rg`

当前 workspace metadata 声明：

- 最低 Rust 版本：`1.79`
- 仓库内置 toolchain channel：`stable`

当前维护者实际使用、能够正常工作的版本是：

- `cargo 1.94.0`
- `rustc 1.94.0`
- `gh 2.83.1`
- `rg 14.1.0`

## 1. 安装或检查工具

先检查本地环境：

```bash
cargo --version
rustup --version
rustc --version
gh --version
rg --version
```

如果缺少 Rust components，可以执行：

```bash
rustup toolchain install stable
rustup component add rustfmt --toolchain stable
rustup component add clippy --toolchain stable
```

如果想用仓库内建的单一工具检查，也可以直接用 `scripts/check-dev-env.py`。

## 1.1 给新 chat 的本地 Ready 文件

请把 `.agent-local/dev-setup-status.md` 当成这个 workspace 的本地 readiness record（就绪记录）。

新的 chat 应该：

- 如果文件存在，先读 `.agent-local/dev-setup-status.md`
- 如果文件写的是 `Status: ready`，就不要在 bootstrap 阶段重复做 dev setup 检查
- 如果文件不存在，或不是 `Status: ready`，就重新执行必要检查
- 把文件写得足够详细，让后续 chat 能看出工具检查与 repo 验证面是否都已覆盖

格式可参考 [`.agent-local/DEV-SETUP-STATUS.example.md`](../.agent-local/DEV-SETUP-STATUS.example.md)。

本地状态文件至少应记录：

- 整体状态
- 检查时间
- 检查者
- `cargo`、`rustup`、`rustc`、`gh`、`rg` 的工具检查
- `rustfmt`、`clippy` 的 Rust component 检查
- 是否跑过完整 repo 验证
- 各个验证命令与其是否成功

建议使用以下工具生成内容：

- `scripts/check-dev-env.py` 用来获取 repo-local 的环境与验证结果
- `scripts/update-dev-setup-status.py` 用来更新本地 readiness record（就绪记录）
- `scripts/check-runtime-preflight.py` 用来在特定测试或验证命令前检查当前 shell session

只有当记录内容已覆盖当前 workspace 需要的工具与验证面时，才把它视为有效的 `Status: ready`。

`Status: ready` 不保证当前 shell session 的 `PATH` 与辅助工具已经满足你接下来要执行的那条验证命令。对于像 `cargo test ... | grep ...` 这类依赖额外 shell 工具的命令，先做一次轻量 runtime preflight：

```bash
scripts/check-runtime-preflight.py
scripts/check-runtime-preflight.py --require grep --require tail
```

如果缺少命令，或后续命令出现 `126`、`127` 这类退出码，应先视为环境阻塞，而不是直接判定为产品失败。

## 2. Clone 并进入仓库

```bash
git clone https://github.com/MycelLayer/Mycel.git
cd Mycel
```

## 2.1 启用 Repo-local Hooks

先为这个 clone 启用 repo 内建的 git hooks：

```bash
git config core.hooksPath .githooks
```

当前的 pre-commit hook 会在 staged 变更触及 `pages/` 时自动执行 `npm run lint:pages`。

## 3. 第一次阅读顺序

开始改任何东西之前，建议按这个顺序先读：

1. [`README.zh-CN.md`](../README.zh-CN.md)
2. [`ROADMAP.zh-CN.md`](../ROADMAP.zh-CN.md)
3. [`IMPLEMENTATION-CHECKLIST.zh-CN.md`](../IMPLEMENTATION-CHECKLIST.zh-CN.md)
4. [`PROTOCOL.zh-CN.md`](../PROTOCOL.zh-CN.md)
5. [`WIRE-PROTOCOL.zh-CN.md`](../WIRE-PROTOCOL.zh-CN.md)
6. [`README.md`](../README.md)
7. 如果你是 AI coding agent，再读 [`BOT-CONTRIBUTING.md`](../BOT-CONTRIBUTING.md)

当前 `zh-CN` 文档已经覆盖 README、roadmap、implementation checklist、protocol 和 wire-spec 这几类主要入口；如果你需要对照最原始的规范措辞或补看尚未本地化的设计说明，再回看英文版。

## 4. 第一次应该运行的命令

在仓库根目录执行：

```bash
cargo fmt --all --check
cargo test -p mycel-core
cargo test -p mycel-cli
cargo run -p mycel-cli -- info
cargo run -p mycel-cli -- validate fixtures/object-sets/minimal-valid/fixture.json --json
./sim/negative-validation/smoke.py --summary-only
```

这些命令分别确认：

- formatting 可用
- core tests 可运行
- CLI tests 可运行
- repo-local CLI wiring 正常
- fixture validation 正常
- simulator negative-validation smoke coverage 正常

## 5. 什么时候算 Setup 完成

如果下面这些条件都成立，就可以认为 setup 完成：

- `cargo fmt --all --check` 成功
- `cargo test -p mycel-core` 成功
- `cargo test -p mycel-cli` 成功
- `mycel-cli -- info` 能在仓库根目录执行
- `fixtures/object-sets/minimal-valid/fixture.json` 能成功验证
- `./sim/negative-validation/smoke.py --summary-only` 成功

完整 setup 验证也可以直接用 `scripts/check-dev-env.py`。

## 6. 常见工作规则

- 优先做范围小、容易 review 的修改。
- protocol-core 变更要保守。
- 如果你改到 protocol 或 design 概念，在中英文双语都存在时应同步更新。
- 优先做 deterministic verification 和 fixture-backed 变更。
- 仓库特定协作规则请读 [`AGENTS.md`](../AGENTS.md)。

## 7. 常用后续命令

```bash
cargo run -p mycel-cli -- object inspect <path> --json
cargo run -p mycel-cli -- object verify <path> --json
cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json --json
```

实用的 repo-local 工具：

- `scripts/check-dev-env.py` 用来做环境验证
- `scripts/check-labels.py` 用来核对 tracked labels
- `scripts/check-plan-refresh.py` 用来检查 planning refresh cadence（规划同步节奏）

## 8. 如果你是新的 AI Agent

setup 完成后，建议下一步：

1. 读 [`docs/PROGRESS.md`](./PROGRESS.md)
2. 读 [`docs/MULTI-AGENT-COORDINATION.md`](./MULTI-AGENT-COORDINATION.md)
3. 找一个 `ai-ready` issue，或者一个很窄的 checklist gap
4. 开始修改前，先确认精确的 file boundary

这个仓库最适合的贡献类型是：范围窄、结果确定、并且能直接对应到某个 spec 或 checklist closure item。
