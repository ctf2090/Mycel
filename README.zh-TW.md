# Mycel

語言：繁體中文 | [English](./README.md)

Mycel 是一個中立、技術導向的文本協議棧，用於可驗證歷史、受治理的閱讀狀態，以及去中心化複製。

## 概覽

Mycel 以文字與參考文本系統為優先，針對分散式環境設計：

- 可驗證的變更歷史
- P2P 複製
- 數位簽章驗證
- 多分支並存，且不要求全域單一共識
- 由 profile 約束的 accepted reading 與狀態選擇
- 建立在穩定 protocol core 之上的可擴充 app-layer models

## 中立原則

Mycel 可以承載各種內容領域；協議本身保持中立且純技術化。

## 目前狀態

- 協議階段：`v0.1` 概念規格，並已延伸出逐漸成形的 profile 與 design-note 層
- 目前重點：first-client 範圍收斂、實作準備度、以及把成熟設計逐步收成具體 profiles
- 目前 Rust CLI 狀態：可用於內部 validation 與可決定 simulator harness 工作流，但還不是 production 等級的 Mycel client 或 node

## 文件導覽

### Specs

- [PROTOCOL.zh-TW.md](./PROTOCOL.zh-TW.md)：完整協議規格
- [WIRE-PROTOCOL.zh-TW.md](./WIRE-PROTOCOL.zh-TW.md)：wire protocol 草案
- [IMPLEMENTATION-CHECKLIST.zh-TW.md](./IMPLEMENTATION-CHECKLIST.zh-TW.md)：最小 v0.1 client 實作檢查清單
- [PROFILE.fund-auto-disbursement-v0.1.zh-TW.md](./PROFILE.fund-auto-disbursement-v0.1.zh-TW.md)：採用 m-of-n custody 的 fund auto-disbursement v0.1 profile 草案
- [PROFILE.mycel-over-tor-v0.1.zh-TW.md](./PROFILE.mycel-over-tor-v0.1.zh-TW.md)：Mycel over Tor v0.1 profile 草案

### Design Notes

- [docs/design-notes/DESIGN-NOTES.client-non-discretionary-multi-view.zh-TW.md](./docs/design-notes/DESIGN-NOTES.client-non-discretionary-multi-view.zh-TW.md)：client-non-discretionary multi-view 設計草案
- [docs/design-notes/DESIGN-NOTES.two-maintainer-role.zh-TW.md](./docs/design-notes/DESIGN-NOTES.two-maintainer-role.zh-TW.md)：two-maintainer-role 設計草案
- [docs/design-notes/DESIGN-NOTES.mycel-app-layer.zh-TW.md](./docs/design-notes/DESIGN-NOTES.mycel-app-layer.zh-TW.md)：Mycel App Layer 設計草案
- [docs/design-notes/DESIGN-NOTES.qa-app-layer.zh-TW.md](./docs/design-notes/DESIGN-NOTES.qa-app-layer.zh-TW.md)：Q&A App Layer 設計草案
- [docs/design-notes/DESIGN-NOTES.qa-minimal-schema.zh-TW.md](./docs/design-notes/DESIGN-NOTES.qa-minimal-schema.zh-TW.md)：Q&A 最小 schema 草案
- [docs/design-notes/DESIGN-NOTES.commentary-citation-schema.zh-TW.md](./docs/design-notes/DESIGN-NOTES.commentary-citation-schema.zh-TW.md)：maintainer 評註文件大量引用原文時的 schema 草案
- [docs/design-notes/DESIGN-NOTES.app-signing-model.zh-TW.md](./docs/design-notes/DESIGN-NOTES.app-signing-model.zh-TW.md)：區分 object signing、release signing 與 execution-evidence signing 的設計草案
- [docs/design-notes/DESIGN-NOTES.signature-priority.zh-TW.md](./docs/design-notes/DESIGN-NOTES.signature-priority.zh-TW.md)：整理哪些 Mycel objects 應優先要求簽章的設計草案
- [docs/design-notes/DESIGN-NOTES.signature-role-matrix.zh-TW.md](./docs/design-notes/DESIGN-NOTES.signature-role-matrix.zh-TW.md)：把目前 object families 對到預設 signing roles 的設計草案
- [docs/design-notes/DESIGN-NOTES.donation-app-layer.zh-TW.md](./docs/design-notes/DESIGN-NOTES.donation-app-layer.zh-TW.md)：Donation App Layer 設計草案
- [docs/design-notes/DESIGN-NOTES.canonical-text-profile.zh-TW.md](./docs/design-notes/DESIGN-NOTES.canonical-text-profile.zh-TW.md)：Canonical Text Profile 設計草案
- [docs/design-notes/DESIGN-NOTES.interpretation-dispute-model.zh-TW.md](./docs/design-notes/DESIGN-NOTES.interpretation-dispute-model.zh-TW.md)：Interpretation Dispute Model 設計草案
- [docs/design-notes/DESIGN-NOTES.auto-signer-consent-model.zh-TW.md](./docs/design-notes/DESIGN-NOTES.auto-signer-consent-model.zh-TW.md)：auto-signer consent model 設計草案
- [docs/design-notes/DESIGN-NOTES.blind-address-threat-model.zh-TW.md](./docs/design-notes/DESIGN-NOTES.blind-address-threat-model.zh-TW.md)：blind-address custody threat model 設計草案
- [docs/design-notes/DESIGN-NOTES.signer-availability-emergency-response.zh-TW.md](./docs/design-notes/DESIGN-NOTES.signer-availability-emergency-response.zh-TW.md)：針對 signer availability 下降的 warning / critical / emergency 應對設計草案
- [docs/design-notes/DESIGN-NOTES.signer-activity-model.zh-TW.md](./docs/design-notes/DESIGN-NOTES.signer-activity-model.zh-TW.md)：評估 signer readiness 與 effective signer capacity 的設計草案
- [docs/design-notes/DESIGN-NOTES.policy-driven-threshold-custody.zh-TW.md](./docs/design-notes/DESIGN-NOTES.policy-driven-threshold-custody.zh-TW.md)：policy-driven m-of-n custody 設計草案
- [docs/design-notes/DESIGN-NOTES.mycel-anonymity-model.zh-TW.md](./docs/design-notes/DESIGN-NOTES.mycel-anonymity-model.zh-TW.md)：Mycel anonymity model 設計草案
- [docs/design-notes/DESIGN-NOTES.first-client-scope-v0.1.zh-TW.md](./docs/design-notes/DESIGN-NOTES.first-client-scope-v0.1.zh-TW.md)：first-client scope v0.1 設計草案
- [docs/design-notes/DESIGN-NOTES.mycel-full-stack-map.zh-TW.md](./docs/design-notes/DESIGN-NOTES.mycel-full-stack-map.zh-TW.md)：Mycel full-stack map 設計草案
- [docs/design-notes/DESIGN-NOTES.mycel-protocol-upgrade-philosophy.zh-TW.md](./docs/design-notes/DESIGN-NOTES.mycel-protocol-upgrade-philosophy.zh-TW.md)：Mycel protocol upgrade philosophy 設計草案
- [docs/design-notes/DESIGN-NOTES.peer-discovery-model.zh-TW.md](./docs/design-notes/DESIGN-NOTES.peer-discovery-model.zh-TW.md)：peer discovery model 設計草案
- [docs/design-notes/DESIGN-NOTES.peer-simulator-v0.zh-TW.md](./docs/design-notes/DESIGN-NOTES.peer-simulator-v0.zh-TW.md)：early multi-peer simulator 與 test harness 設計草案
- [docs/design-notes/DESIGN-NOTES.sensor-triggered-donation.zh-TW.md](./docs/design-notes/DESIGN-NOTES.sensor-triggered-donation.zh-TW.md)：Sensor-triggered Donation 設計草案
- [docs/design-notes/DESIGN-NOTES.governance-history-security.zh-TW.md](./docs/design-notes/DESIGN-NOTES.governance-history-security.zh-TW.md)：Governance History Security 設計草案

### Meta

- [PROJECT-INTENT.zh-TW.md](./PROJECT-INTENT.zh-TW.md)：專案意圖與協議邊界說明
- [AGENTS.md](./AGENTS.md)：repo 協作規則

### Implementation Scaffold

- [fixtures/README.md](./fixtures/README.md)：供 simulator 與 verification 測試使用的語言中立 fixture sets
- [sim/README.md](./sim/README.md)：peer simulator 結構、topologies、tests 與 reports 的語言中立骨架
- [sim/SCHEMA-CROSS-CHECK.zh-TW.md](./sim/SCHEMA-CROSS-CHECK.zh-TW.md)：說明 simulator schemas 與各種 IDs 應如何彼此對上的 cross-check 規則
- [RUST-WORKSPACE.md](./RUST-WORKSPACE.md)：Rust core、simulator library 與 CLI 的初始 workspace 佈局

### CI

- [.github/workflows/ci.yml](./.github/workflows/ci.yml)：GitHub Actions workflow，負責 Rust 檢查與 negative validation smoke coverage

## 近期優先事項

1. 先做一個狹窄的 first client，聚焦在 sync、verification、accepted-head selection 與 reader-first 文本閱讀
2. 持續把成熟的 design areas 收成明確的 profiles 或 schemas，而不是過快擴大 protocol core
3. 採逐層往上擴的方式：先 canonical-text reading，再選擇性加入 app-layer 支援

## 專案定位

Mycel 不是：

- 追求全域強制共識的區塊鏈
- 單純的檔案傳輸系統
- Git 複製品

Mycel 是一個可驗證、可演進、去中心化的文本歷史與受治理文本系統之協議棧。
