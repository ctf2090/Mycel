# Client-Non-Discretionary Multi-View

狀態：design draft

這份筆記描述一種做法：保留 Mycel 的 multi-view，同時盡量降低一般客戶端對 active accepted head 的影響力。

## 0. 目標

保留：

- 多個已簽章的 View objects
- 多個可並存的 branches 與 heads
- 決定性的 accepted-head selection

降低：

- 使用者主動切換 view
- client 本地覆寫 selector
- UI 驅動的 policy 變動
- 臨時把偏好的 head 提升成 active accepted head

## 1. 核心想法

Mycel 在 protocol layer 仍然是 multi-view，但合規的 reader 客戶端不應對 active accepted head 擁有太多自由裁量權。

客戶端主要只做：

- 同步已驗證物件
- 驗證 hash 與 signature
- 計算 selector 輸出
- 顯示 accepted result 與其 trace

客戶端不應自由做：

- 修改 selector parameters
- 更改 maintainer weights
- 強制讓偏好的 head 變成 active
- 臨時拼出本地 policy，並把它當成 active accepted view

## 2. 名詞

- `View object`：已簽章的 governance signal object，不是使用者偏好
- `View profile`：用來評估 View objects 的固定 selector profile
- `accepted head`：對 `(profile_id, doc_id, effective_selection_time)` 算出的唯一 selected revision
- `reader client`：消費 accepted result，但不產生治理訊號的客戶端
- `curator client`：可發布 View objects，但仍不能改 selector math 的客戶端

## 3. 把 View object 視為治理訊號

在這個模型裡，View object 不表示「使用者自己選的視角」。

它表示：

- maintainer 對採信文件 revision 的已簽章聲明
- 對某份 policy body 的已簽章承諾
- 決定 accepted-head selection 的其中一個輸入

這樣可以保留 `view` 作為可複製的 protocol object，同時把它從一般終端使用者的偏好控制中抽離。

## 4. Profile-Locked Selection

每個 network profile 或 document family 應固定一個 View profile。

至少要固定：

- `policy_hash`
- `epoch_seconds`
- `epoch_zero_timestamp`
- `admission_window_epochs`
- `min_valid_views_for_admission`
- `min_valid_views_per_epoch`
- `weight_cap_per_key`
- tie-break 順序

一般客戶端不得在 active accepted-head 路徑上本地修改這些值。

## 5. Accepted-Head 計算

對每個 `(profile_id, doc_id, effective_selection_time)`：

1. 載入該文件的已驗證 revisions
2. 載入 `policy_hash` 相符的已驗證 View objects
3. 計算 eligible heads
4. 導出 maintainer signals
5. 計算 `selector_score`
6. 套用固定 tie-break 順序
7. 輸出唯一 `accepted_head`

accepted head 是 protocol 導出的，不是使用者自己選的。
它也不是在宣稱整個網路只有一個普遍接受的真版本；它是在某個固定 profile 與某個 effective selection time 下算出的 accepted result。

## 6. 客戶端角色

### 6.1 Reader Client

合規的 reader 客戶端：

- 可以同步並驗證所有物件
- 可以顯示 raw heads 與 branch graphs
- 可以顯示 decision traces
- 必須把算出的 accepted head 顯示為 active
- 不得讓使用者用自由裁量的本地選擇取代 active accepted head

### 6.2 Curator Client

curator 客戶端：

- 可以建立並簽署 View objects
- 可以向 network 發布 governance signals
- 仍必須使用 protocol-defined selector profile
- 不得為自己的 accepted-head output 改寫 selector math

### 6.3 Governance Update Tooling

若 profile 本身要改，應透過明確的 profile versioning 或 governance-update 工作流程，而不是透過安靜的本地客戶端設定。

## 7. UI 規則

建議 reader UI：

- 顯示一個預設 accepted head
- 顯示 governing `profile_id`
- 顯示 `effective_selection_time`
- 顯示可機器解析或可檢查的 decision trace
- 只把其他 heads 當成 alternatives、branches、或 audit material 顯示

避免：

- 把「choose your preferred view」做成主要 reader control
- 隱藏本地覆寫
- 把 raw branch choice 偽裝成 protocol-accepted result

## 8. Client 仍然可能影響什麼

即使在這個模型裡，客戶端仍可能透過以下方式間接影響結果：

- 沒有同步到足夠物件
- 驗證實作錯誤
- selector 實作錯誤
- 隱藏 trace 或 branch alternatives

所以目標不是零影響，而是降低自由裁量影響，並把決策盡量移到可驗證的 protocol data。

## 9. 建議的未來規範語句

可考慮在未來 spec revision 中加入：

- View objects are governance signals, not end-user preference objects.
- A conforming reader client MUST derive the active accepted head from verified objects and the fixed protocol-defined View profile only.
- A conforming reader client MUST NOT expose discretionary local policy controls that alter the active accepted head.
- A conforming reader client MAY expose raw heads, branch graphs, and rejected alternatives, but MUST NOT present them as the active accepted head unless another valid View profile governs that result.

## 10. 取捨

好處：

- 保留 Mycel 的 multi-view
- 降低 client-side divergence
- 提高 auditability
- 降低 UI 設定默默改變 acceptance 的風險

成本：

- profile 設計會變得更重要
- governance updates 需要更清楚的 versioning path
- reader tooling 和 curator tooling 應更清楚分層

## 11. 開放問題

- 一個 network 應允許多個固定 profiles，還是每個 document family 只能有一個 active profile？
- reader 客戶端可以檢視其他合法 profiles，還是只能看 network default？
- 在 governed multi-view network 中，`VIEW_ANNOUNCE` 應繼續是 optional，還是實質上變成必要？
- implementation checklist 是否應明確拆成 reader-client 與 curator-client 兩份需求？
