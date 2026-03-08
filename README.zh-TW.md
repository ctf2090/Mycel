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
- 目前重點：物件模型、簽章模型、複製流程、合併行為

## 文件導覽

- [PROTOCOL.en.md](./PROTOCOL.en.md)：英文完整協議規格
- [PROTOCOL.zh-TW.md](./PROTOCOL.zh-TW.md)：繁體中文完整協議規格
- [AGENTS.md](./AGENTS.md)：repo 協作規則

## 近期優先事項

1. 完成 wire protocol 欄位定義（`HELLO`、`WANT`、`OBJECT`）
2. 鎖定 canonical serialization 規則，確保雜湊一致
3. 定義 block 層級的合併語義

## 專案定位

Mycel 不是：

- 追求全域強制共識的區塊鏈
- 單純的檔案傳輸系統
- Git 複製品

Mycel 是一個可驗證、可演進、去中心化的文本歷史協議。
