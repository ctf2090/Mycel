# Mycel 特性解說（繁中）

狀態：working explanation

這份文件用來說明幾個常被拿來描述 Mycel 的特性詞，在 Mycel 裡各自代表什麼，以及它們不應被誤解成什麼。

這不是規範來源。若某個詞和實際資料模型、驗證規則或 accepted-head 推導有衝突，仍以 [`PROTOCOL.zh-TW.md`](../PROTOCOL.zh-TW.md)、[`ROADMAP.zh-TW.md`](../ROADMAP.zh-TW.md) 與相關實作為準。

## 1. `content-addressed`

在 Mycel 中，`content-addressed` 指的是某些核心物件的 identity 由 canonical 內容導出，而不是由中心伺服器分配。

這在 Mycel 裡的意義：

- `patch`、`revision`、`view`、`snapshot` 等物件可由內容導出 canonical object ID。
- 物件是否相同，優先看 canonical 內容是否相同，而不是它被誰先上傳或儲存在什麼位置。
- 複製、驗證與重建可以建立在內容定址物件之上，而不是依賴單一儲存節點的本地資料庫主鍵。

這不代表：

- 所有 ID 都是內容雜湊。
- Mycel 沒有邏輯 ID。像 `doc_id`、`block_id` 仍然是邏輯參照，不等同內容定址 ID。
- 只要資料有 hash，就自動得到完整治理或 accepted-head 語意。

## 2. `append-only`

在 Mycel 中，`append-only` 指的是可驗證歷史以新增物件為主，而不是原地覆寫既有歷史物件。

這在 Mycel 裡的意義：

- 新的 patch、revision、view、snapshot 會擴充歷史，而不是直接修改舊物件。
- 歷史可以被 replay / rebuild，因此需要保留先前物件，而不是只保留最後一份狀態。
- audit、驗證與 accepted-state 推導建立在累積的歷史之上。

這不代表：

- 所有本地索引或快取都不能更新。
- 沒有 garbage collection、壓縮或 presentation-layer 摘要。
- 每個 app-layer 都必須把自己的所有衍生資料也做成 append-only。

## 3. `replayable`

在 Mycel 中，`replayable` 指的是目前可見狀態可以從已驗證歷史物件重新推導，而不是只能依賴某個不可重建的即時資料庫狀態。

這在 Mycel 裡的意義：

- `revision` 的狀態可由 patch/revision 歷史重播驗證。
- store index 或 accepted reading 的一部分可從既有物件重建，而不是唯一真相來源。
- 「重建後應得到相同結果」是重要工程目標。

這不代表：

- 所有結果都只靠 replay 就能得到，完全不需要 policy、profile 或 governance signal。
- replay 一定便宜或即時。
- 任何 app-layer 副作用都必須可完整重播。

## 4. `policy-driven`

在 Mycel 中，`policy-driven` 指的是某些可見結果不是硬編碼唯一答案，而是依照 profile、policy bundle 或其他明確規則推導。

這在 Mycel 裡的意義：

- accepted head 不是單靠「最新時間戳」或「最後寫入」決定。
- 是否接受某個 head、如何處理治理訊號、哪些讀法被視為預設，都可能受固定規則影響。
- app-layer 或 profile-layer 可以承載較高階的決策，而不是把所有世界觀寫死進 protocol core。

這不代表：

- client 可以自由裁量地任意挑自己喜歡的結果。
- policy 就只是使用者 UI 偏好。
- 每個節點都一定會得到同一個 accepted head；結果仍取決於共同使用的規則與可見物件集合。

## 5. `head-selected`

在 Mycel 中，`head-selected` 指的是系統允許存在多個合法 head，但會透過明確 selector 規則導出預設採用的 head。

這在 Mycel 裡的意義：

- Mycel 不要求資料模型先天只允許 one head。
- 多個候選 head 可以同時存在，然後由 selector 在固定上下文下導出 accepted head。
- selector 是資料模型與治理模型之間的重要層，而不是單純 UI 排序。

這不代表：

- Mycel 只是一般的 multi-head version graph。
- 任何多 head 都會被自動 merge 成單一真相。
- `accepted head` 等同全網共識。

## 6. `governance-aware`

在 Mycel 中，`governance-aware` 指的是系統不只保存內容歷史，也把治理訊號、角色與規則看成影響 accepted-state 的正式輸入。

這在 Mycel 裡的意義：

- `view` 不是單純註解，而是治理訊號的一部分。
- accepted reading 可能依賴 view maintainer、profile 與治理規則，而不只是內容本身。
- 關係摘要、治理索引與 inspect/list/publish surface 都是 reader-facing interpretation 的一部分。

這不代表：

- Mycel 的核心協定已經內建所有治理制度。
- governance 一定意味著 centralized authority。
- 所有 app-layer 治理都必須進 protocol core。

## 7. 這幾個詞合在一起時的 Mycel 讀法

把這六個詞放在一起時，Mycel 比較接近下面這種描述：

- Mycel 以 content-addressed objects 為基礎。
- 它保留 append-only 的可驗證歷史。
- 它重視 replayable / rebuildable 的狀態推導。
- 它允許合法 heads 並存，再透過 head selection 導出預設採用結果。
- 它把 policy 與 governance 視為正式的一級語意，而不是純 UI 附加層。

若要用一句較短的繁中描述，可以寫成：

- Mycel 是一個以內容定址物件為基礎、保留可重播歷史、並透過 policy-driven head selection 與 governance-aware interpretation 導出預設閱讀結果的系統。

## 8. 建議避免的誤讀

以下說法容易把這些詞講得太粗，建議避免：

- 「content-addressed = 所有 ID 都是 hash」
- 「append-only = 任何東西都不能更新」
- 「replayable = 不需要本地索引或快取」
- 「policy-driven = 使用者愛怎麼選就怎麼選」
- 「head-selected = 一定只有一個真正的 head」
- 「governance-aware = 一定是中心化治理」
