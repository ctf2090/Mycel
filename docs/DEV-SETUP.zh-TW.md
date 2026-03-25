# 開發環境設置

這份文件提供從全新 checkout 到可實際開發 Mycel 的最短路徑。

建議搭配閱讀：

- [`README.zh-TW.md`](../README.zh-TW.md)
- [`CONTRIBUTING.md`](../CONTRIBUTING.md)
- [`BOT-CONTRIBUTING.md`](../BOT-CONTRIBUTING.md)
- [`RUST-WORKSPACE.md`](../RUST-WORKSPACE.md)

## 0. 需要的工具

必要工具：

- Rust toolchain manager：`rustup`
- Rust `stable` toolchain
- Rust components：`rustfmt`、`clippy`
- GitHub CLI：`gh`
- ripgrep：`rg`

目前 workspace metadata 宣告：

- 最低 Rust 版本：`1.79`
- repo 內建 toolchain channel：`stable`

目前維護者實際使用、可正常工作的版本是：

- `cargo 1.94.0`
- `rustc 1.94.0`
- `gh 2.83.1`
- `rg 14.1.0`

## 1. 安裝或確認工具

先檢查本機環境：

```bash
cargo --version
rustup --version
rustc --version
gh --version
rg --version
```

若缺少 Rust components，可用：

```bash
rustup toolchain install stable
rustup component add rustfmt --toolchain stable
rustup component add clippy --toolchain stable
```

如果想用 repo 內建的單一工具檢查，也可以直接用 `scripts/check-dev-env.py`。

## 1.1 給新 chat 的本地 Ready 檔

請把 `.agent-local/dev-setup-status.md` 當成這個 workspace 的本地 readiness record（就緒紀錄）。

新的 chat 應該：

- 若檔案存在，先讀 `.agent-local/dev-setup-status.md`
- 若檔案寫的是 `Status: ready`，就不要在 bootstrap 期間重複做 dev setup 檢查
- 若檔案不存在，或不是 `Status: ready`，就重新執行必要檢查
- 把檔案寫得足夠詳細，讓之後的 chat 能看出工具檢查與 repo 驗證面是否都已涵蓋

格式可參考 [`.agent-local/DEV-SETUP-STATUS.example.md`](../.agent-local/DEV-SETUP-STATUS.example.md)。

本地狀態檔至少應記錄：

- 整體狀態
- 檢查時間
- 檢查者
- `cargo`、`rustup`、`rustc`、`gh`、`rg` 的工具檢查
- `rustfmt`、`clippy` 的 Rust component 檢查
- 是否跑過完整 repo 驗證
- 各個驗證命令與其是否成功

建議使用以下工具產生內容：

- `scripts/check-dev-env.py` 用來取得 repo-local 的環境與驗證結果
- `scripts/update-dev-setup-status.py` 用來更新本地 readiness record（就緒紀錄）
- `scripts/check-runtime-preflight.sh` 用來在特定測試或驗證命令前檢查目前 shell session

只有當記錄內容已涵蓋目前 workspace 需要的工具與驗證面時，才把它視為有效的 `Status: ready`。

`Status: ready` 不保證目前 shell session 的 `PATH` 與輔助工具已符合你接下來要跑的那條驗證命令。對於像 `cargo test ... | grep ...` 這類會依賴額外 shell 工具的命令，先做一次輕量 runtime preflight：

```bash
scripts/check-runtime-preflight.sh
scripts/check-runtime-preflight.sh --require grep --require tail
```

如果缺少命令，或後續命令出現 `126`、`127` 這類退出碼，應先視為環境阻塞，而不是直接判定為產品失敗。

## 2. Clone 並進入 Repo

```bash
git clone https://github.com/MycelLayer/Mycel.git
cd Mycel
```

## 2.1 啟用 Repo-local Hooks

先為這個 clone 啟用 repo 內建的 git hooks：

```bash
git config core.hooksPath .githooks
```

目前的 pre-commit hook 會在 staged 變更觸及 `pages/` 時，自動執行 `npm run lint:pages`。

## 3. 第一次閱讀順序

開始改任何東西前，建議依這個順序先讀：

1. [`README.zh-TW.md`](../README.zh-TW.md)
2. [`ROADMAP.zh-TW.md`](../ROADMAP.zh-TW.md)
3. [`IMPLEMENTATION-CHECKLIST.zh-TW.md`](../IMPLEMENTATION-CHECKLIST.zh-TW.md)
4. [`PROTOCOL.zh-TW.md`](../PROTOCOL.zh-TW.md)
5. [`WIRE-PROTOCOL.zh-TW.md`](../WIRE-PROTOCOL.zh-TW.md)
6. 如果你是 AI coding agent，再讀 [`BOT-CONTRIBUTING.md`](../BOT-CONTRIBUTING.md)

## 4. 第一次應該跑的命令

在 repository root 執行：

```bash
cargo fmt --all --check
cargo test -p mycel-core
cargo test -p mycel-cli
cargo run -p mycel-cli -- info
cargo run -p mycel-cli -- validate fixtures/object-sets/minimal-valid/fixture.json --json
./sim/negative-validation/smoke.py --summary-only
```

這些命令分別確認：

- formatting 可用
- core tests 可跑
- CLI tests 可跑
- repo-local CLI wiring 正常
- fixture validation 正常
- simulator negative-validation smoke coverage 正常

## 5. 何時算 Setup 完成

若以下條件都成立，就可視為 setup 完成：

- `cargo fmt --all --check` 成功
- `cargo test -p mycel-core` 成功
- `cargo test -p mycel-cli` 成功
- `mycel-cli -- info` 能在 repo root 執行
- `fixtures/object-sets/minimal-valid/fixture.json` 可成功驗證
- `./sim/negative-validation/smoke.py --summary-only` 成功

完整 setup 驗證也可以直接用 `scripts/check-dev-env.py`。

## 6. 常見工作規則

- 優先做範圍窄、容易 review 的修改。
- protocol-core 變更要保守。
- 若你改到 protocol 或 design 概念，當中英兩版都存在時要同步更新。
- 優先做 deterministic verification 與 fixture-backed 變更。
- repo 特定協作規則請讀 [`AGENTS.md`](../AGENTS.md)。

## 7. 常用後續命令

```bash
cargo run -p mycel-cli -- object inspect <path> --json
cargo run -p mycel-cli -- object verify <path> --json
cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json --json
```

實用的 repo-local 工具：

- `scripts/check-dev-env.py` 用來做環境驗證
- `scripts/check-labels.py` 用來核對 tracked labels
- `scripts/check-plan-refresh.py` 用來檢查 planning refresh cadence（規劃同步節奏）

## 8. 如果你是新的 AI Agent

setup 完成後，建議下一步：

1. 讀 [`docs/PROGRESS.md`](./PROGRESS.md)
2. 讀 [`docs/MULTI-AGENT-COORDINATION.md`](./MULTI-AGENT-COORDINATION.md)
3. 找一張 `ai-ready` issue 或一個很窄的 checklist gap
4. 開始改之前先確認精確的 file boundary

這個 repo 最適合的貢獻類型是：範圍窄、可決定、並且直接對應到某個 spec 或 checklist closure item。
