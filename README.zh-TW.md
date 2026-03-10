# Mycel

語言：繁體中文 | [English](./README.md)

Mycel 是一個以 Rust 實作為主的協議棧，用於可驗證的文本歷史、受治理的閱讀狀態，以及去中心化複製。

它面向的是需要以下能力的 text-first 系統：

- 可重播驗證的歷史
- 可驗證的簽章治理訊號
- 多個有效分支並存，而不要求全域強制共識
- 由固定 profile 規則決定 accepted reading，而不是任意本地偏好

## 為什麼是 Mycel

多數協作工具通常落在兩種形態：

- 由中心化平台維護可變狀態
- 為程式碼協作或全域共識最佳化的分散式系統

Mycel 走的是另一條路。它把文本歷史、accepted reading、以及 replication 視為彼此分離但可互通的層次。

因此，它特別適合長期文本、評註系統、受治理的參考文本集合，以及其他以文字為核心的分散式工作流。

## 它的差異點

- 可驗證歷史：revision 預期要能被 replay 與檢查，而不是只靠信任。
- 受治理的閱讀狀態：accepted head 來自固定 profile 規則與已驗證的 View objects。
- 容許分叉：多個 head 可以並存，而不假設整個網路必須只有一個全域真相。
- 中立的 protocol core：領域語義應放在 profiles 與 app layers，不應寫死進 core protocol。

換句話說，所謂「採信版本」不是全網共識，而是在某個固定 profile 下，從已驗證物件推導出的版本。

## 它不是什麼

Mycel 不是：

- 要求全域強制共識的區塊鏈
- Git 複製品
- 泛用檔案傳輸層

## 60 秒內可以試什麼

目前的 Rust CLI 是內部驗證與 simulator 工具鏈，還不是 production 等級的 Mycel client 或 node。

如果你是從全新環境開始，請先看 [`docs/DEV-SETUP.zh-TW.md`](./docs/DEV-SETUP.zh-TW.md)。

在 repo 根目錄執行：

```bash
cargo run -p mycel-cli -- info
cargo run -p mycel-cli -- validate fixtures/object-sets/minimal-valid/fixture.json --json
cargo run -p mycel-cli -- sim run sim/tests/three-peer-consistency.example.json --json
```

這三個命令分別可以看到：

- `info`：repo 內部 workspace 與 scaffold 路徑
- `validate`：對已提交 fixtures 的穩定 validation 輸出
- `sim run`：可決定的 simulator harness 執行摘要，以及產生出的 report 路徑

## 目前狀態

- 協議階段：`v0.1` 概念規格，並持續擴充 profile 與 design-note 層
- 目前實作重點：收斂 first-client 範圍、加強 replay 與 verification、以及穩定 deterministic simulator workflows
- 目前 CLI 邊界：適合在本 repo 內做 validation、object inspection、object verification、accepted-head inspection、report inspection 與 simulator runs
- 尚未交付：production node 行為、公開網路 wire sync、或完整的終端使用者 client

## 依目的閱讀

如果你想走最短理解路徑：

- 先看 protocol core：[PROTOCOL.zh-TW.md](./PROTOCOL.zh-TW.md)
- 再看 transport 規則：[WIRE-PROTOCOL.zh-TW.md](./WIRE-PROTOCOL.zh-TW.md)
- 看實作順序：[ROADMAP.zh-TW.md](./ROADMAP.zh-TW.md)
- 看 build checklist：[IMPLEMENTATION-CHECKLIST.zh-TW.md](./IMPLEMENTATION-CHECKLIST.zh-TW.md)

如果你想從全新環境開始貢獻：

- 先看 setup：[docs/DEV-SETUP.zh-TW.md](./docs/DEV-SETUP.zh-TW.md)
- 再看貢獻預期：[CONTRIBUTING.md](./CONTRIBUTING.md)
- 如果你使用 AI coding agent，再接著看：[BOT-CONTRIBUTING.md](./BOT-CONTRIBUTING.md)

## 從這裡開始貢獻

如果你想先接一張範圍窄的任務，可以從這幾張仍開放的 issue 開始：

- [#1 在共用 object parsing 中拒絕重複 JSON object keys](https://github.com/ctf2090/Mycel/issues/1)
- [#3 補上 document 與 block objects 的 malformed logical-ID coverage](https://github.com/ctf2090/Mycel/issues/3)
- [#4 補上 snapshot derived-ID verification smoke coverage](https://github.com/ctf2090/Mycel/issues/4)

這些連結應和 [`docs/PLANNING-SYNC-PLAN.zh-TW.md`](./docs/PLANNING-SYNC-PLAN.zh-TW.md) 的 planning sync 流程一起刷新。

如果你想看更結構化的任務入口，請直接瀏覽帶有 `ai-ready` 與 `well-scoped` labels 的 issues。

如果你想理解目前的 Rust 實作：

- Workspace 地圖：[RUST-WORKSPACE.md](./RUST-WORKSPACE.md)
- Simulator scaffold：[sim/README.md](./sim/README.md)
- Fixture 佈局：[fixtures/README.md](./fixtures/README.md)

如果你想理解目前決策背後的設計層：

- First-client 邊界：[docs/design-notes/DESIGN-NOTES.first-client-scope-v0.1.zh-TW.md](./docs/design-notes/DESIGN-NOTES.first-client-scope-v0.1.zh-TW.md)
- Full-stack 地圖：[docs/design-notes/DESIGN-NOTES.mycel-full-stack-map.zh-TW.md](./docs/design-notes/DESIGN-NOTES.mycel-full-stack-map.zh-TW.md)
- Protocol 升級哲學：[docs/design-notes/DESIGN-NOTES.mycel-protocol-upgrade-philosophy.zh-TW.md](./docs/design-notes/DESIGN-NOTES.mycel-protocol-upgrade-philosophy.zh-TW.md)

## 主要文件

### Specs

- [PROTOCOL.zh-TW.md](./PROTOCOL.zh-TW.md)：core protocol 規格
- [WIRE-PROTOCOL.zh-TW.md](./WIRE-PROTOCOL.zh-TW.md)：transport message format 與 sync flow 草案
- [ROADMAP.zh-TW.md](./ROADMAP.zh-TW.md)：從 first client 到後續擴展的分階段建置順序
- [IMPLEMENTATION-CHECKLIST.zh-TW.md](./IMPLEMENTATION-CHECKLIST.zh-TW.md)：窄版可互通 client 的實作檢查清單
- [PROFILE.fund-auto-disbursement-v0.1.zh-TW.md](./PROFILE.fund-auto-disbursement-v0.1.zh-TW.md)：窄版 app-layer custody profile 草案
- [PROFILE.mycel-over-tor-v0.1.zh-TW.md](./PROFILE.mycel-over-tor-v0.1.zh-TW.md)：窄版 Tor 導向部署 profile 草案

### Design Notes

- [docs/design-notes/DESIGN-NOTES.first-client-scope-v0.1.zh-TW.md](./docs/design-notes/DESIGN-NOTES.first-client-scope-v0.1.zh-TW.md)：first client 現階段該做什麼、刻意延後什麼
- [docs/design-notes/DESIGN-NOTES.client-non-discretionary-multi-view.zh-TW.md](./docs/design-notes/DESIGN-NOTES.client-non-discretionary-multi-view.zh-TW.md)：受 protocol 約束的 reader model
- [docs/design-notes/DESIGN-NOTES.two-maintainer-role.zh-TW.md](./docs/design-notes/DESIGN-NOTES.two-maintainer-role.zh-TW.md)：editor 與 view maintainer 權責拆分
- [docs/design-notes/DESIGN-NOTES.mycel-full-stack-map.zh-TW.md](./docs/design-notes/DESIGN-NOTES.mycel-full-stack-map.zh-TW.md)：目前文件集的分層地圖
- [docs/design-notes/DESIGN-NOTES.peer-simulator-v0.zh-TW.md](./docs/design-notes/DESIGN-NOTES.peer-simulator-v0.zh-TW.md)：早期 simulator 與 harness 方向

### Meta

- [PROJECT-INTENT.zh-TW.md](./PROJECT-INTENT.zh-TW.md)：專案意圖邊界說明
- [CONTRIBUTING.md](./CONTRIBUTING.md)：貢獻預期
- [docs/DEV-SETUP.zh-TW.md](./docs/DEV-SETUP.zh-TW.md)：從全新 checkout 到可用開發環境的最短路徑
- [AGENTS.md](./AGENTS.md)：repo 協作規則

## 近期優先事項

1. 完成窄版 first-client core，聚焦 verification、replay、storage 與 accepted-head inspection。
2. 在擴大 protocol-core 範圍之前，先把成熟想法落成明確的 profiles、schemas、fixtures 與 tests。
3. 採逐層上推的方式擴展：先 canonical-text reading，再逐步加入選擇性的 app-layer 支援。

## 授權

本 repository 採用 [MIT License](./LICENSE)，除非未來有個別檔案或目錄另有標示。

關於貢獻與授權預期，請參考 [CONTRIBUTING.md](./CONTRIBUTING.md)。
