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

如果想用一条命令检查，也可以直接运行：

```bash
scripts/check-dev-env.sh
scripts/check-dev-env.sh --full
scripts/check-dev-env.sh --json
scripts/check-dev-env.sh --full --json
```

`--full` 不只是检查工具是否存在，也会直接执行当前仓库的验证面，所以它可能因为当前 workspace 状态失败，而不一定只是环境没装好。
`--json` 适合给自动化工具或 agent 使用，方便读取机器可解析的结果。

## 2. Clone 并进入仓库

```bash
git clone https://github.com/ctf2090/Mycel.git
cd Mycel
```

## 3. 第一次阅读顺序

开始改任何东西之前，建议按这个顺序先读：

1. [`README.zh-CN.md`](../README.zh-CN.md)
2. [`README.md`](../README.md)
3. [`ROADMAP.md`](../ROADMAP.md)
4. [`IMPLEMENTATION-CHECKLIST.en.md`](../IMPLEMENTATION-CHECKLIST.en.md)
5. [`PROTOCOL.en.md`](../PROTOCOL.en.md)
6. [`WIRE-PROTOCOL.en.md`](../WIRE-PROTOCOL.en.md)
7. 如果你是 AI coding agent，再读 [`BOT-CONTRIBUTING.md`](../BOT-CONTRIBUTING.md)

当前 `zh-CN` 文档还是第一批入口覆盖，协议和路线类文档暂时以英文版为主参考面。

## 4. 第一次应该运行的命令

在仓库根目录执行：

```bash
cargo fmt --all --check
cargo test -p mycel-core
cargo test -p mycel-cli
cargo run -p mycel-cli -- info
cargo run -p mycel-cli -- validate fixtures/object-sets/minimal-valid/fixture.json --json
./sim/negative-validation/smoke.sh --summary-only
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
- `./sim/negative-validation/smoke.sh --summary-only` 成功

完整 setup 验证也可以直接用：

```bash
scripts/check-dev-env.sh --full
```

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
scripts/check-dev-env.sh
scripts/check-labels.sh
scripts/check-doc-refresh.sh
scripts/check-doc-refresh.sh --json
```

## 8. 如果你是新的 AI Agent

setup 完成后，建议下一步：

1. 读 [`docs/PROGRESS.md`](./PROGRESS.md)
2. 读 [`docs/MULTI-AGENT-COORDINATION.md`](./MULTI-AGENT-COORDINATION.md)
3. 找一个 `ai-ready` issue，或者一个很窄的 checklist gap
4. 开始修改前，先确认精确的 file boundary

这个仓库最适合的贡献类型是：范围窄、结果确定、并且能直接对应到某个 spec 或 checklist closure item。
