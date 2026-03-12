# Mycel v0.1 實作檢查清單

狀態：late partial progress，已在最近一批 canonical-helper consolidation、merge-authoring coverage 擴張，以及 editor-admission head inspect/render 工作後刷新；實作狀態未變，M1 parsing、parser / verify / CLI strictness coverage、更廣的 inspect-surface parity、signature-edge 與 replay/verify smoke coverage、fixture isolation、test-foundation cleanup 與 canonical reproducibility core 仍接近完成

這份清單把 v0.1 規格轉成偏實作導向的建置計畫，目標是一個最小但可互通的客戶端。

## 0. 建置目標

先鎖定受限版 v0.1 客戶端：

- 一個本地物件儲存
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

- [x] 選定一個實作語言與 package layout。
- [x] 為 network profile 固定一個 canonical hash algorithm。
- [x] 為 client profile 固定一組 signature algorithms。
- [ ] 加入一個可被 hash、signature、wire 共用的 canonical JSON 工具。
- [x] 加入 protocol examples 與 regression tests 的 fixture 載入機制。

## 2. 物件型別與 ID

- [x] 實作 `document` 解析，並把 `doc_id` 視為 logical ID。
- [x] 實作 `block` 解析，並把 `block_id` 視為 logical ID。
- [x] 實作帶導出 `patch_id` 的 `patch` 解析。
- [x] 實作帶導出 `revision_id` 的 `revision` 解析。
- [x] 實作帶導出 `view_id` 的 `view` 解析。
- [x] 實作帶導出 `snapshot_id` 的 `snapshot` 解析。
- [x] 拒絕任何內嵌導出 ID 與重算 canonical ID 不一致的內容定址物件。
- [x] 在 shared parsing 與 verification 中，拒絕 typed object 的未知頂層欄位與非法必要欄位型別。
- [ ] 在最近 strictness-surface 與 replay/verify smoke 擴張後，完成剩餘 malformed field-shape depth 與 semantic edge case 的收尾。
- [ ] 將 editor-maintainer 與 view-maintainer 的角色指派分開建模。

## 3. Canonical Serialization 與 Hashing

- [x] 把所有協議物件 canonicalize 成 UTF-8 JSON，且不含多餘空白。
- [x] 強制 object key 以字典序排序。
- [x] 完整保留 array 順序。
- [x] 拒絕 duplicate keys。
- [x] 拒絕 `null`、浮點數等不支援的值型別。
- [x] 重算 object ID 時省略導出 ID 欄位與 `signature`。
- [ ] 對 `state_hash` 與 wire envelope signature 重用同一套 canonicalization 規則。

## 4. Signature 驗證

- [x] 實作 object signature matrix。
- [x] 禁止 `document` 與 `block` 帶 signature。
- [x] 要求 `patch`、`revision`、`view`、`snapshot` 必須帶 signature。
- [x] 只有在 canonical ID 檢查通過後才驗簽。
- [ ] 對所有 v0.1 wire message type 實作 envelope signature 驗證。
- [ ] 拒絕任何未通過 profile 必要簽章檢查的物件或訊息。

## 5. Patch 與 Revision Engine

- [ ] 實作 v0.1 patch operations：
- [x] `insert_block`
- [x] `insert_block_after`
- [x] `delete_block`
- [x] `replace_block`
- [x] `move_block`
- [x] `annotate_block`
- [x] `set_metadata`
- [x] 強制非 genesis patch 的 `base_revision` 等於 execution-base revision。
- [x] 支援 genesis sentinel `rev:genesis-null`。
- [x] 依陣列順序套用 revision 的 `patches`。
- [x] 把 `parents[0]` 視為唯一 execution base state。
- [x] 把 `parents[1..]` 視為 ancestry-only，除非內容被顯式 patch operation 實體化。
- [x] 對每個接收的 revision 重算並驗證 `state_hash`。
- [x] 讓 revision 發布權與 accepted-head governance weight 維持分離。

## 6. 本地狀態與儲存

- [x] 以 canonical `object_id` 儲存所有接收的物件。
- [x] 維護 `doc_id -> revisions` 索引。
- [x] 維護 `revision_id -> parents` 索引。
- [x] 維護 `author -> patches` 索引。
- [x] 維護 `view_id -> governance signal contents` 索引。
- [x] 維護 `profile_id -> selected document heads` 索引。
- [ ] 把本地 transport 與 safety policy 與可複製的協定物件分開保存。
- [x] 不讓自由裁量的本地 policy 進入 active accepted-head 路徑。
- [x] 支援只靠 object store 就能重建 indexes。

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

- [x] 把已驗證的 `view` 物件當成 governance signals 保存，並與本地 transport/safety policy state 分開。
- [x] 依 `profile_id`、`doc_id`、`effective_selection_time` 分組 selector inputs。
- [x] 將 `profile_id` 解析為 active reader profile 的固定 `policy_hash`。
- [x] 精準實作 eligible heads 判定。
- [x] 只使用 `policy_hash` 相同且已驗證的 View 物件作為 view-maintainer signals。
- [x] 精準實作 selector epoch 計算。
- [x] 實作規範中的 `selector_score`。
- [x] 實作規範中的 tie-break 順序。
- [x] 輸出或保存最小 decision trace schema。
- [x] 不提供會改變 active accepted head 的自由裁量本地 policy controls。
- [x] 若支援多個固定 profiles，必須明確列舉，而不是允許 ad hoc local profiles。
- [x] 確保僅有 editor-maintainer 身分不會自動取得 selector weight。
- [ ] 若支援 dual-role keys，必須分別驗證 editor-maintainer 與 view-maintainer 準入。

## 10. Merge Generation

- [x] 保持 revision 驗證為 replay-based；不要要求接收端重跑 merge generation。
- [x] 為本地作者工具實作保守版 merge generation profile。
- [x] 區分 `Auto-merged`、`Multi-variant`、`Manual-curation-required`。
- [x] 把 merge 結果實體化成一般 patch operations。
- [x] 拒絕用隱藏 merge metadata 取代顯式狀態變更。

## 11. CLI 或 API 介面

- [x] 提供本地 init command 或 API。
- [x] 提供 object verification 工具。
- [x] 提供 document creation 與 patch authoring 入口。
- [x] 提供 revision commit 入口。
- [ ] 提供 sync pull 入口。
- [x] 提供 view inspection 或 head inspection 入口。
- [x] 提供可從 stored objects 或明確 object bundles 進行 accepted-head render 的入口。
- [x] 把 reader-facing accepted-head inspection 與 curator-facing View publication workflow 分開。
- [x] 讓 head inspection 的 `decision_trace` 只保留高階摘要層。
- [x] 把 maintainer、weight、violation 的機器可消費細節放在 `effective_weights[]`、`maintainer_support[]`、`critical_violations[]` 這類 typed arrays，而不是塞進 `decision_trace`。
- [x] 把 `decision_trace` 視為給人讀的解釋輸出；把 typed arrays 視為給工具與測試依賴的穩定細節介面。
- [x] 把 editor-maintainer revision publication 與 view-maintainer governance publication workflow 分開。
- [x] 提供 store-rebuild 或 reindex 入口，以利復原。

## 12. Interop Test 最低門檻

- [x] 載入所有規範性 example objects，並確認可解析。
- [x] 對 example `patch`、`revision`、`view`、`snapshot` 重算 derived IDs。
- [x] 至少對一個 single-parent revision 與一個 merge revision 重算 `state_hash`。
- [x] 驗證 example wire envelopes 與 `OBJECT` 驗證行為。
- [x] 加入 hash mismatch、signature mismatch、invalid parent ordering 的 negative tests。
- [x] 加入 canonical serialization 的 round-trip test。
- [x] 加入只靠儲存物件重建 document state 的 replay test。

## 13. 可開始建 client 的門檻

當以下條件都成立時，可把客戶端視為 ready for first interoperable build：

- [ ] 所有必要 object types 都能解析並驗證
- [x] canonical IDs 與 signatures 可重現
- [x] revision replay 與 `state_hash` 驗證通過
- [ ] 最小 wire sync 可端到端跑通
- [x] 決定性 head selection 產出穩定結果
- [x] merge generation 能產生有效且可 replay 的 patch operations
- [x] 本地 store 可只靠 canonical objects 完整重建
