# Mycel v0.1 實作檢查清單

狀態：draft

這份清單把 v0.1 規格轉成偏實作導向的 build plan，目標是一個最小但可互通的 client。

## 0. 建置目標

先鎖定受限版 v0.1 client：

- 一個本地 object store
- 一個固定的 network hash algorithm
- 只支援 canonical JSON
- 支援 `patch` / `revision` / `view` / `snapshot`
- 實作 `HELLO`、`MANIFEST`、`HEADS`、`WANT`、`OBJECT`、`BYE`、`ERROR`
- 若宣告對應 capability，才實作 `SNAPSHOT_OFFER` 與 `VIEW_ANNOUNCE`
- 以 replay 為基礎的 revision 驗證
- 決定性且 profile-locked 的 head selection
- 保守版 merge generation profile

可延後：

- 豐富的 editor UX
- 進階 policy UI
- 自動 key discovery
- 非 JSON 編碼
- 自訂 merge plugin

## 1. Repo 與建置設定

- [ ] 選定一個實作語言與 package layout。
- [ ] 為 network profile 固定一個 canonical hash algorithm。
- [ ] 為 client profile 固定一組 signature algorithms。
- [ ] 加入一個可被 hash、signature、wire 共用的 canonical JSON utility。
- [ ] 加入 protocol examples 與 regression tests 的 fixture 載入機制。

## 2. 物件型別與 ID

- [ ] 實作 `document` 解析，並把 `doc_id` 視為 logical ID。
- [ ] 實作 `block` 解析，並把 `block_id` 視為 logical ID。
- [ ] 實作帶導出 `patch_id` 的 `patch` 解析。
- [ ] 實作帶導出 `revision_id` 的 `revision` 解析。
- [ ] 實作帶導出 `view_id` 的 `view` 解析。
- [ ] 實作帶導出 `snapshot_id` 的 `snapshot` 解析。
- [ ] 拒絕任何內嵌導出 ID 與重算 canonical ID 不一致的內容定址物件。
- [ ] 依我們選定的 strictness policy，拒絕未知必要欄位或非法欄位型別。

## 3. Canonical Serialization 與 Hashing

- [ ] 把所有協議物件 canonicalize 成 UTF-8 JSON，且不含多餘空白。
- [ ] 強制 object key 以字典序排序。
- [ ] 完整保留 array 順序。
- [ ] 拒絕 duplicate keys。
- [ ] 拒絕 `null`、浮點數等不支援的值型別。
- [ ] 重算 object ID 時省略導出 ID 欄位與 `signature`。
- [ ] 對 `state_hash` 與 wire envelope signature 重用同一套 canonicalization 規則。

## 4. Signature 驗證

- [ ] 實作 object signature matrix。
- [ ] 禁止 `document` 與 `block` 帶 signature。
- [ ] 要求 `patch`、`revision`、`view`、`snapshot` 必須帶 signature。
- [ ] 只有在 canonical ID 檢查通過後才驗簽。
- [ ] 對所有 v0.1 wire message type 實作 envelope signature 驗證。
- [ ] 拒絕任何未通過 profile 必要簽章檢查的物件或訊息。

## 5. Patch 與 Revision Engine

- [ ] 實作 v0.1 patch operations：
- [ ] `insert_block`
- [ ] `insert_block_after`
- [ ] `delete_block`
- [ ] `replace_block`
- [ ] `move_block`
- [ ] `annotate_block`
- [ ] `set_metadata`
- [ ] 強制非 genesis patch 的 `base_revision` 等於 execution-base revision。
- [ ] 支援 genesis sentinel `rev:genesis-null`。
- [ ] 依陣列順序套用 revision 的 `patches`。
- [ ] 把 `parents[0]` 視為唯一 execution base state。
- [ ] 把 `parents[1..]` 視為 ancestry-only，除非內容被顯式 patch operation 實體化。
- [ ] 對每個接收的 revision 重算並驗證 `state_hash`。

## 6. 本地狀態與儲存

- [ ] 以 canonical `object_id` 儲存所有接收的物件。
- [ ] 維護 `doc_id -> revisions` 索引。
- [ ] 維護 `revision_id -> parents` 索引。
- [ ] 維護 `author -> patches` 索引。
- [ ] 維護 `view_id -> governance signal contents` 索引。
- [ ] 維護 `profile_id -> selected document heads` 索引。
- [ ] 把本地 transport 與 safety policy 與可複製的協議物件分開保存。
- [ ] 不讓自由裁量的本地 policy 進入 active accepted-head 路徑。
- [ ] 支援只靠 object store 就能重建 indexes。

## 7. Wire Protocol

- [ ] 實作 canonical wire envelope。
- [ ] 驗證 `type`、`version`、`msg_id`、`timestamp`、`from`、`payload`、`sig`。
- [ ] 對 wire messages 強制 RFC 3339 時間格式。
- [ ] 實作 `HELLO`。
- [ ] 實作 `MANIFEST`。
- [ ] 實作 `HEADS`。
- [ ] 實作 `WANT`。
- [ ] 實作 `OBJECT`。
- [ ] 實作 `BYE`。
- [ ] 實作 `ERROR`。
- [ ] 只有在宣告 `snapshot-sync` 時才實作 `SNAPSHOT_OFFER`。
- [ ] 只有在宣告 `view-sync` 時才實作 `VIEW_ANNOUNCE`。
- [ ] 對每個 `OBJECT` 重算 `hash(body)`。
- [ ] 依 `object_type` 與 `hash` 重建預期的 `object_id`。
- [ ] 拒絕任何內嵌導出 ID 與 envelope `object_id` 不一致的 `OBJECT`。

## 8. 同步流程

- [ ] 支援首次同步：`HELLO` -> `MANIFEST` / `HEADS` -> `WANT` -> `OBJECT`。
- [ ] 支援從更新後 `HEADS` 進行增量同步。
- [ ] 只以 canonical object ID 抓取缺失物件。
- [ ] 先驗證物件，再建立索引或提供給 reader。
- [ ] 若對方宣告 snapshot，可支援 snapshot-assisted catch-up。
- [ ] 若啟用 `view-sync`，可支援抓取已公告的 views。
- [ ] 將抓回的 View objects 視為 governance signals，而不是使用者偏好狀態。

## 9. Views 與 Head Selection

- [ ] 把已驗證的 `view` 物件當成 governance signals 保存，並與本地 transport/safety policy state 分開。
- [ ] 依 `profile_id`、`doc_id`、`effective_selection_time` 分組 selector inputs。
- [ ] 將 `profile_id` 解析為 active reader profile 的固定 `policy_hash`。
- [ ] 精準實作 eligible heads 判定。
- [ ] 只使用 `policy_hash` 相同且已驗證的 View 物件作為 maintainer signals。
- [ ] 精準實作 selector epoch 計算。
- [ ] 實作規範中的 `selector_score`。
- [ ] 實作規範中的 tie-break 順序。
- [ ] 輸出或保存最小 decision trace schema。
- [ ] 不提供會改變 active accepted head 的自由裁量本地 policy controls。
- [ ] 若支援多個固定 profiles，必須明確列舉，而不是允許 ad hoc local profiles。

## 10. Merge Generation

- [ ] 保持 revision 驗證為 replay-based；不要要求接收端重跑 merge generation。
- [ ] 為本地作者工具實作保守版 merge generation profile。
- [ ] 區分 `Auto-merged`、`Multi-variant`、`Manual-curation-required`。
- [ ] 把 merge 結果實體化成一般 patch operations。
- [ ] 拒絕用隱藏 merge metadata 取代顯式狀態變更。

## 11. CLI 或 API 介面

- [ ] 提供本地 init command 或 API。
- [ ] 提供 object verification 工具。
- [ ] 提供 document creation 與 patch authoring 入口。
- [ ] 提供 revision commit 入口。
- [ ] 提供 sync pull 入口。
- [ ] 提供 view inspection 或 head inspection 入口。
- [ ] 把 reader-facing accepted-head inspection 與 curator-facing View publication workflow 分開。
- [ ] 提供 store-rebuild 或 reindex 入口，以利復原。

## 12. Interop Test 最低門檻

- [ ] 載入所有規範性 example objects，並確認可解析。
- [ ] 對 example `patch`、`revision`、`view`、`snapshot` 重算 derived IDs。
- [ ] 至少對一個 single-parent revision 與一個 merge revision 重算 `state_hash`。
- [ ] 驗證 example wire envelopes 與 `OBJECT` 驗證行為。
- [ ] 加入 hash mismatch、signature mismatch、invalid parent ordering 的 negative tests。
- [ ] 加入 canonical serialization 的 round-trip test。
- [ ] 加入只靠儲存物件重建 document state 的 replay test。

## 13. 可開始建 client 的門檻

當以下條件都成立時，可把 client 視為 ready for first interoperable build：

- [ ] 所有必要 object types 都能解析並驗證
- [ ] canonical IDs 與 signatures 可重現
- [ ] revision replay 與 `state_hash` 驗證通過
- [ ] 最小 wire sync 可端到端跑通
- [ ] 決定性 head selection 產出穩定結果
- [ ] merge generation 能產生有效且可 replay 的 patch operations
- [ ] 本地 store 可只靠 canonical objects 完整重建
