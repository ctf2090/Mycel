# Mycel

語言：繁體中文 | [English](./README.md)

Mycel 是一個中立、技術導向的文本協議，用於可驗證歷史與去中心化複製。

## 概覽

Mycel 以文字為優先，針對分散式協作設計：

- 可驗證的變更歷史
- P2P 複製
- 數位簽章驗證
- 多分支並存，且不要求全域單一共識

## 中立原則

Mycel 可以承載各種內容領域；協議本身保持中立且純技術化。

## 目前狀態

- 協議階段：`v0.1` 概念規格
- 目前重點：規格一致性、實作準備度、以及治理參數瘦身

## 文件導覽

- [PROTOCOL.en.md](./PROTOCOL.en.md)：英文完整協議規格
- [PROTOCOL.zh-TW.md](./PROTOCOL.zh-TW.md)：繁體中文完整協議規格
- [WIRE-PROTOCOL.en.md](./WIRE-PROTOCOL.en.md)：英文 wire protocol 草案
- [WIRE-PROTOCOL.zh-TW.md](./WIRE-PROTOCOL.zh-TW.md)：繁體中文 wire protocol 草案
- [IMPLEMENTATION-CHECKLIST.en.md](./IMPLEMENTATION-CHECKLIST.en.md)：英文最小 v0.1 client 實作檢查清單
- [IMPLEMENTATION-CHECKLIST.zh-TW.md](./IMPLEMENTATION-CHECKLIST.zh-TW.md)：繁體中文實作檢查清單
- [DESIGN-NOTES.client-non-discretionary-multi-view.en.md](./DESIGN-NOTES.client-non-discretionary-multi-view.en.md)：英文 client-non-discretionary multi-view 設計草案
- [DESIGN-NOTES.client-non-discretionary-multi-view.zh-TW.md](./DESIGN-NOTES.client-non-discretionary-multi-view.zh-TW.md)：繁體中文同主題設計草案
- [DESIGN-NOTES.two-maintainer-role.en.md](./DESIGN-NOTES.two-maintainer-role.en.md)：英文 two-maintainer-role 設計草案
- [DESIGN-NOTES.two-maintainer-role.zh-TW.md](./DESIGN-NOTES.two-maintainer-role.zh-TW.md)：繁體中文同主題設計草案
- [DESIGN-NOTES.mycel-app-layer.en.md](./DESIGN-NOTES.mycel-app-layer.en.md)：英文 Mycel App Layer 設計草案
- [DESIGN-NOTES.mycel-app-layer.zh-TW.md](./DESIGN-NOTES.mycel-app-layer.zh-TW.md)：繁體中文同主題設計草案
- [DESIGN-NOTES.qa-app-layer.en.md](./DESIGN-NOTES.qa-app-layer.en.md)：英文 Q&A App Layer 設計草案
- [DESIGN-NOTES.qa-app-layer.zh-TW.md](./DESIGN-NOTES.qa-app-layer.zh-TW.md)：繁體中文同主題設計草案
- [DESIGN-NOTES.qa-minimal-schema.en.md](./DESIGN-NOTES.qa-minimal-schema.en.md)：英文 Q&A 最小 schema 草案
- [DESIGN-NOTES.qa-minimal-schema.zh-TW.md](./DESIGN-NOTES.qa-minimal-schema.zh-TW.md)：繁體中文同主題 schema 草案
- [DESIGN-NOTES.donation-app-layer.en.md](./DESIGN-NOTES.donation-app-layer.en.md)：英文 Donation App Layer 設計草案
- [DESIGN-NOTES.donation-app-layer.zh-TW.md](./DESIGN-NOTES.donation-app-layer.zh-TW.md)：繁體中文同主題設計草案
- [DESIGN-NOTES.neuro-triggered-donation.en.md](./DESIGN-NOTES.neuro-triggered-donation.en.md)：英文 Neuro-triggered Donation 設計草案
- [DESIGN-NOTES.neuro-triggered-donation.zh-TW.md](./DESIGN-NOTES.neuro-triggered-donation.zh-TW.md)：繁體中文同主題設計草案
- [AGENTS.md](./AGENTS.md)：repo 協作規則

## 近期優先事項

1. 用 implementation checklist（實作檢查清單）收斂最小 reference client 範圍
2. 決定 v0.1 的治理參數是否還要再瘦身後再視為穩定
3. 若第一版 client 還要更保守，再把 minimal checklist 收斂成更窄的 reference profile（參考實作設定檔）

## 專案定位

Mycel 不是：

- 追求全域強制共識的區塊鏈
- 單純的檔案傳輸系統
- Git 複製品

Mycel 是一個可驗證、可演進、去中心化的文本歷史協議。
