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

## 文件導覽

### Specs

- Protocol： [EN](./PROTOCOL.en.md) | [繁中](./PROTOCOL.zh-TW.md)
- Wire Protocol： [EN](./WIRE-PROTOCOL.en.md) | [繁中](./WIRE-PROTOCOL.zh-TW.md)
- Implementation Checklist： [EN](./IMPLEMENTATION-CHECKLIST.en.md) | [繁中](./IMPLEMENTATION-CHECKLIST.zh-TW.md)
- Fund Auto-disbursement Profile v0.1： [EN](./PROFILE.fund-auto-disbursement-v0.1.en.md) | [繁中](./PROFILE.fund-auto-disbursement-v0.1.zh-TW.md)
- Mycel over Tor Profile v0.1： [EN](./PROFILE.mycel-over-tor-v0.1.en.md) | [繁中](./PROFILE.mycel-over-tor-v0.1.zh-TW.md)

### Design Notes

- Client Non-discretionary Multi-view： [EN](./DESIGN-NOTES.client-non-discretionary-multi-view.en.md) | [繁中](./DESIGN-NOTES.client-non-discretionary-multi-view.zh-TW.md)
- Two-Maintainer-Role Model： [EN](./DESIGN-NOTES.two-maintainer-role.en.md) | [繁中](./DESIGN-NOTES.two-maintainer-role.zh-TW.md)
- Mycel App Layer： [EN](./DESIGN-NOTES.mycel-app-layer.en.md) | [繁中](./DESIGN-NOTES.mycel-app-layer.zh-TW.md)
- Q&A App Layer： [EN](./DESIGN-NOTES.qa-app-layer.en.md) | [繁中](./DESIGN-NOTES.qa-app-layer.zh-TW.md)
- Q&A Minimal Schema： [EN](./DESIGN-NOTES.qa-minimal-schema.en.md) | [繁中](./DESIGN-NOTES.qa-minimal-schema.zh-TW.md)
- Donation App Layer： [EN](./DESIGN-NOTES.donation-app-layer.en.md) | [繁中](./DESIGN-NOTES.donation-app-layer.zh-TW.md)
- Canonical Text Profile： [EN](./DESIGN-NOTES.canonical-text-profile.en.md) | [繁中](./DESIGN-NOTES.canonical-text-profile.zh-TW.md)
- Interpretation Dispute Model： [EN](./DESIGN-NOTES.interpretation-dispute-model.en.md) | [繁中](./DESIGN-NOTES.interpretation-dispute-model.zh-TW.md)
- Auto-signer Consent Model： [EN](./DESIGN-NOTES.auto-signer-consent-model.en.md) | [繁中](./DESIGN-NOTES.auto-signer-consent-model.zh-TW.md)
- Policy-driven Threshold Custody： [EN](./DESIGN-NOTES.policy-driven-threshold-custody.en.md) | [繁中](./DESIGN-NOTES.policy-driven-threshold-custody.zh-TW.md)
- Mycel Anonymity Model： [EN](./DESIGN-NOTES.mycel-anonymity-model.en.md) | [繁中](./DESIGN-NOTES.mycel-anonymity-model.zh-TW.md)
- First-client Scope v0.1： [EN](./DESIGN-NOTES.first-client-scope-v0.1.en.md) | [繁中](./DESIGN-NOTES.first-client-scope-v0.1.zh-TW.md)
- Mycel Full-stack Map： [EN](./DESIGN-NOTES.mycel-full-stack-map.en.md) | [繁中](./DESIGN-NOTES.mycel-full-stack-map.zh-TW.md)
- Mycel Protocol Upgrade Philosophy： [EN](./DESIGN-NOTES.mycel-protocol-upgrade-philosophy.en.md) | [繁中](./DESIGN-NOTES.mycel-protocol-upgrade-philosophy.zh-TW.md)
- Peer Discovery Model： [EN](./DESIGN-NOTES.peer-discovery-model.en.md) | [繁中](./DESIGN-NOTES.peer-discovery-model.zh-TW.md)
- Sensor-triggered Donation： [EN](./DESIGN-NOTES.sensor-triggered-donation.en.md) | [繁中](./DESIGN-NOTES.sensor-triggered-donation.zh-TW.md)
- Governance History Security： [EN](./DESIGN-NOTES.governance-history-security.en.md) | [繁中](./DESIGN-NOTES.governance-history-security.zh-TW.md)

### Meta

- Project Intent： [EN](./PROJECT-INTENT.md) | [繁中](./PROJECT-INTENT.zh-TW.md)
- [AGENTS.md](./AGENTS.md)：repo 協作規則

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
