# Mycel 術語表（繁中）

狀態：working glossary

這份文件提供 Mycel repo 範圍的繁中術語對照與推薦寫法。

目標：

- 統一協定核心、profile、設計備忘錄與 README 的中文用語
- 降低 `document`、`accepted head`、`profile` 等術語的誤讀
- 提供對外說明時較穩定的中文表述

這份術語表不是規範來源。規範性定義仍以 [`PROTOCOL.zh-TW.md`](/workspaces/Mycel/PROTOCOL.zh-TW.md) 為準。

## 1. 使用原則

- 涉及協定核心的正式術語，優先沿用規格文件既有寫法。
- 翻譯或修訂 `zh-TW` 文件時，優先使用台灣慣用詞與表達方式，而不只是把文字轉成繁體。
- 若中文直譯容易誤導，保留英文並補短中文解釋。
- 欄位名、enum 值、物件型別名在資料結構中保留英文；正文可用中文解釋。
- `accepted` 不應直接翻成「共識」，因為 Mycel 不要求全網單一共識。

## 2. Core Protocol 術語

| English term | 建議中文 | 補充說明 |
| --- | --- | --- |
| protocol core | 協定核心 | 指 Mycel 最底層可驗證物件與規則，不含應用層語意。 |
| document | Document / 文件 | 在 Mycel 裡是「一條可長期演化且可重播的物件歷史」，不必然是傳統文字文件。 |
| block | 區塊 / 段落區塊 | v0.1 主要操作單位。 |
| patch | 修改 | 一次對 document state 的修改。 |
| revision | 修訂 | 表示某個可驗證狀態，不只是編輯紀錄。 |
| view | 治理 View | 已簽章治理訊號，用來導出 accepted head。 |
| snapshot | 快照包 | 某一時刻的打包狀態。 |
| logical ID | 邏輯 ID | 例如 `doc_id`、`block_id`，屬於狀態內穩定參照，不是內容雜湊。 |
| canonical object ID | canonical object ID / 內容定址物件 ID | 例如 `patch_id`、`revision_id`、`view_id`、`snapshot_id`。 |
| replay | replay / 重播驗證 | 依據歷史物件重建狀態並檢查其正確性。 |
| state hash | `state_hash` / 狀態雜湊 | 由 canonical state 導出的可重現雜湊。 |
| head | head | 建議保留英文；表示某條 document 歷史目前沒有子孫的修訂端點。 |
| accepted head | accepted head / 已採信 head | 在固定 profile 下被導出的預設 head。 |
| accepted reading | 預設閱讀版本 | 指 reader 在固定 profile 下預設採用的閱讀版本。 |
| eligible head | eligible head / 合格 head | 符合 selector 前置條件、可進一步參與 accepted-head 選擇的 head。 |
| selector | selector / 選擇規則 | 用來在合法 heads 間導出 accepted head 的規則。 |
| selector epoch | `selector_epoch` / 選擇 epoch | selector 計算上下文的一部分。 |
| view maintainer | View 維護者 | 對 View 發布治理訊號的維護角色。 |
| reader client | 閱讀客戶端 | 顯示 document family 並導出 accepted reading 的客戶端。 |

## 3. Profile 與治理術語

| English term | 建議中文 | 補充說明 |
| --- | --- | --- |
| profile | profile / 規則組 | 正式技術語境建議保留 `profile`。 |
| fixed profile | 固定 profile | 指不可臨時依本地偏好改動的規則組。 |
| profile-governed | 由 profile 決定 | 強調結果來自固定規則，而不是自由裁量。 |
| policy | policy / 政策約束 | 可以是 profile 內的一部分，也可以是一組更具體的執行條件。 |
| policy bundle | policy bundle / 政策包 | 一組共同生效的政策條件。 |
| accepted-state derivation | accepted-state 推導 | 從已驗證物件與固定規則導出可採用狀態。 |
| governance signal | 治理訊號 | 例如 View 的簽章聲明。 |
| non-discretionary | 非自由裁量 | 指 client 不應依本地偏好任意選 accepted head。 |

## 4. Replication 與實作術語

| English term | 建議中文 | 補充說明 |
| --- | --- | --- |
| replication | 複製 | 指物件在 peers 間被傳遞與保存。 |
| peer | peer / 對等節點 | 建議保留 `peer`，必要時補中文。 |
| ingest | 匯入 | 物件進入本地儲存。 |
| rebuild | 重建 | 由已知物件重新構建狀態或索引。 |
| fixture | fixture / 測試樣本 | repo 中用於決定性驗證的固定資料。 |
| simulator | simulator / 模擬器 | 用於測試 peer / topology / reports 的模擬層。 |
| negative validation | 負向驗證 | 確認錯誤案例會被正確拒絕。 |
| deterministic | 決定性 | 輸入相同時結果可重現。 |

## 5. App-layer 常用術語

| English term | 建議中文 | 補充說明 |
| --- | --- | --- |
| app layer | 應用層 | 位於協定核心之上，承載領域語意。 |
| record family | 記錄家族 | 一組相關 document families 或 object streams。 |
| runtime | runtime / 執行環境 | 指執行外部效果或本地判定的執行環境。 |
| effect layer | 外部效果層 | 顯式表示外部觀測、支付、通知等副作用。 |
| consent profile | consent profile / 同意規則組 | 使用者事前授權條件。 |
| session record | 時段記錄 | 一次有邊界的執行或觀測摘要。 |
| derived event | 導出事件 | 從 session 或其他 evidence 摘要出的高階事件。 |
| intent | 意圖 | 系統準備採取某動作前的可驗證中間狀態。 |
| pledge | 承諾 | 尚未實際結算的承諾或待確認狀態。 |
| receipt | 收據 / 回執 | 外部效果完成或失敗後的可稽核紀錄。 |
| dispute | 爭議 | 針對某意圖、結算或狀態提出的異議。 |
| revoke | 撤回 | 取消既有授權。 |
| pause | 暫停 | 暫時停用，但不代表永久撤回。 |

## 6. 建議避免的說法

以下說法容易造成誤解，建議避免：

- 把 `accepted head` 寫成「全網共識版本」
- 把 `document` 直接理解成「一篇文章」或「一個檔案」
- 把 `profile` 寫成「使用者個人偏好」
- 把 `View` 寫成「畫面」或單純 UI view
- 把 `replay` 寫成單純的「回放動畫」

## 7. 目前推薦短句

若要用繁中快速介紹 Mycel，建議優先用這幾句：

- Mycel 是一個用於可驗證文本歷史、依規則導出的預設閱讀版本與去中心化複製的協定。
- 所謂「預設採用版本」不是全網共識，而是依固定 profile 規則，從已驗證物件推導出的結果。
- `Document` 在 Mycel 中是長期可重播的物件歷史，不必然是傳統文字文件。
- 應用層語意應放在 profiles 與 applications，不應寫死進協定核心。
