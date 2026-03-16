# Mycel v0.1 實作檢查清單

狀態：`M1` minimal-client gate 已關閉，並在下方保留為已完成清單；新的 post-`M1` follow-up checklist 現在用來追蹤仍未完成的 `M2` / `M3` / `M4` 工作，包括更廣的 governance persistence、更廣的 peer interop，以及 production replication behavior

這份清單把 v0.1 規格轉成偏實作導向的建置計畫，目標是一個最小但可互通的客戶端。

它現在分成兩個角色：

- Part A 保留已關閉的 `M1` minimal-client gate 與其完成 proof points
- Part B 追蹤 `M2`、`M3`、`M4` 仍未完成的 follow-up work

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

## Part A. 已關閉的 `M1` Minimal-Client Gate

以下章節保留為已關閉 minimal-client gate 的歷史記錄。

## 1. Repo 與建置設定

- [x] 選定一個實作語言與 package layout。
- [x] 為 network profile 固定一個 canonical hash algorithm。
- [x] 為 client profile 固定一組 signature algorithms。
- [x] 完成 shared canonical JSON 工具，讓它可被 hash、signature、剩餘 wire-validation 路徑，以及未來 wire code 共用。
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
- [x] 在最近 strictness-surface 擴張、replay-dependency CLI smoke 擴張，以及 ancestry-context proof 擴張後，完成剩餘 malformed field-shape depth、semantic edge case 與角色建模的收尾。
- [x] 將 editor-maintainer 與 view-maintainer 的角色指派分開建模，並涵蓋 mixed-role 與 shared-key case。

## 3. Canonical Serialization 與 Hashing

- [x] 把所有協議物件 canonicalize 成 UTF-8 JSON，且不含多餘空白。
- [x] 強制 object key 以字典序排序。
- [x] 完整保留 array 順序。
- [x] 拒絕 duplicate keys。
- [x] 拒絕 `null`、浮點數等不支援的值型別。
- [x] 重算 object ID 時省略導出 ID 欄位與 `signature`。
- [x] 完成讓剩餘 wire-validation 路徑與未來 wire envelope signature 重用同一套 canonicalization 規則。

## 4. Signature 驗證

- [x] 實作 object signature matrix。
- [x] 禁止 `document` 與 `block` 帶 signature。
- [x] 要求 `patch`、`revision`、`view`、`snapshot` 必須帶 signature。
- [x] 只有在 canonical ID 檢查通過後才驗簽。
- [x] 對所有 v0.1 wire message type 實作 envelope signature 驗證。
- [x] 拒絕任何未通過 profile 必要簽章檢查的物件或訊息。

## 5. Patch 與 Revision Engine

- [x] 實作 v0.1 patch operations：
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
- [x] 把本地 transport 與 safety policy 與可複製的協定物件分開保存。
- [x] 不讓自由裁量的本地 policy 進入 active accepted-head 路徑。
- [x] 支援只靠 object store 就能重建 indexes。

## 7. Wire Protocol

- [x] 實作 canonical wire envelope。
- [x] 驗證 `type`、`version`、`msg_id`、`timestamp`、`from`、`payload`、`sig`。
- [x] 對 wire messages 強制 RFC 3339 時間格式。
- [x] 實作 `HELLO`。
- [x] 實作 `MANIFEST`。
- [x] 實作 `HEADS`。
- [x] 實作 `WANT`。
- [x] 實作 `OBJECT`。
- [x] 實作 `BYE`。
- [x] 實作 `ERROR`。
- [x] 只有在宣告 `snapshot-sync` 時才實作 `SNAPSHOT_OFFER`。
- [x] 只有在宣告 `view-sync` 時才實作 `VIEW_ANNOUNCE`。
- [x] 對每個 `OBJECT` 重算 `hash(body)`。
- [x] 依 `object_type` 與 `hash` 重建預期的 `object_id`。
- [x] 拒絕任何內嵌導出 ID 與 envelope `object_id` 不一致的 `OBJECT`。

## 8. 同步流程

- [x] 支援 peers 之間的首次同步：`HELLO` -> `MANIFEST` / `HEADS` -> `WANT` -> `OBJECT`。
- [x] 支援 peers 之間從更新後 `HEADS` 進行增量同步。
- [x] 只以 canonical object ID 抓取缺失物件。
- [x] 先驗證物件，再建立索引或提供給 reader。
- [x] 若對方宣告 snapshot，可支援 snapshot-assisted catch-up。
- [x] 若啟用 `view-sync`，可支援抓取已公告的 views。
- [x] 將抓回的 View objects 視為 governance signals，而不是使用者偏好狀態。

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
- [x] 若支援多個固定 profiles，必須明確列舉並在介面上清楚呈現，而不是允許 ad hoc local profiles。
- [x] 確保僅有 editor-maintainer 身分不會自動取得 selector weight。
- [x] 若 viewer signals 可影響 `selector_score`，必須把它們建模為 bounded、typed 的 score channels，並採用受 cap 限制的 viewer bonus / penalty paths，而不是 raw popularity counts。
- [x] 若 viewer signals 可影響 `selector_score`，必須定義具 evidence 與 expiry semantics 的 `approval`、`objection`、`challenge` typed signals。
- [x] 若 viewer signals 可影響 `selector_score`，必須以明確的 anti-Sybil、admission 或 reputation 規則來限制 eligibility 與 effective signal weight。
- [x] 若 viewer signals 可影響 `selector_score`，必須以穩定 typed arrays 與 traces 呈現 viewer contribution，避免把 maintainer governance 壓扁成 raw public preference。
- [x] 若支援 dual-role keys，必須分別驗證 editor-maintainer 與 view-maintainer 準入。

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
- [x] 提供 sync pull 入口。
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

- [x] 所有必要 object types 都能解析並驗證
- [x] canonical IDs 與 signatures 可重現
- [x] revision replay 與 `state_hash` 驗證通過
- [x] 最小 wire sync 可端到端跑通
- [x] 決定性 head selection 產出穩定結果
- [x] merge generation 能產生有效且可 replay 的 patch operations
- [x] 本地 store 可只靠 canonical objects 完整重建

## Part B. Post-`M1` Follow-Up Checklist

這一節是目前仍在進行中的實作檢查清單，用來追蹤 post-`M1` lane 的未完成事項。

## 14. `M2` Replay、Storage 與 Rebuild Follow-Up

- [ ] 擴大 persisted store indexes 在 reader 與 recovery workflows 中的重用，避免 accepted-head 與 render paths 過度依賴臨時 CLI glue。
- [ ] 補上比目前直接 proof points 更強的 replay 與 store-rebuild fixtures，涵蓋更真實的 multi-document 與 recovery-oriented 情境。
- [ ] 把更多 authoring 與 replay helper ownership 收斂到 `mycel-core`，避免 storage-write 與 replay 行為過度偏 CLI 驅動。
- [ ] 擴大 conservative merge-authoring 對 richer nested 與 reparenting conflict cases 的 coverage，處理目前仍落回 manual curation 的情況。
- [ ] 明確定義並驗證在 minimal-client gate 之後仍未完成的 narrow object-authoring 與 storage-write path。

## 15. `M3` Reader 與 Governance Follow-Up

- [ ] 補上超出目前 initial reverse-index 與 inspect/list/publish surfaces 的 broader governance persistence。
- [ ] 把 governance tooling 擴展到目前初始 `view inspect` / `view list` / `view publish` workflows 之外。
- [ ] 持續改善 reader profile ergonomics，超出目前 available-profile summaries 與 profile-error feedback。
- [ ] 收掉 separate admission validation 已落地後，仍然存在的 independent dual-role role-assignment follow-up。

## 16. `M4` Wire Sync 與 Peer Interop Follow-Up

- [x] 把 peer-interop proof 擴展到目前 peer-store-driven first-time 與 incremental sync coverage 之外。
- [x] 補上 localhost multi-process 或等價 transport proof，避免目前 sync path 只在窄版 transcript 或 simulator-controlled paths 下被驗證。
- [x] 定義並測試目前 minimal sync proof 之外仍未完成的 production replication behavior。範圍：下列三個子項目。
  - [x] Re-sync 冪等性：reader 已是最新狀態時，再次執行 sync 應產生零次新寫入、無錯誤、accepted heads 不變。
  - [x] Depth-N 增量追趕：位於 revision depth 1 的 reader 透過一次 HEADS/WANT 追上 depth ≥ 3 的 seed，且只抓取差異部分。
  - [x] 部分文件選擇性 sync：reader 只請求 seed 部分文件，最終 store 穩定在所請求的子集，且 accepted heads 僅針對請求文件正確計算（PROTOCOL §8 明訂支援 partial replication）。
- [ ] 擴大 session、capability 與 error-path interop coverage，超出目前 positive-path 與 optional-message proof set。

## 17. Cross-Surface Closure Rules

- [ ] 當任何 post-`M1` checklist section 狀態改變時，同步保持 `ROADMAP.md`、`ROADMAP.zh-TW.md` 與 `docs/PROGRESS.md` 一致。
- [ ] 對 durable 的 follow-up gaps 開立或更新 narrowly-scoped GitHub Issues，而不是只把 post-`M1` work 留在摘要文字裡。
