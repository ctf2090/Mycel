# Forum App Layer

狀態：design draft

這份筆記描述 Mycel 如何承載一個 forum-style 的應用層，同時把 forum 語義留在核心協議之外。

核心原則是：

- Mycel 承載 forum state、governance state、moderation history 與 audit traces
- client 從已驗證物件渲染 boards、threads 與 replies
- 多個 candidate moderation outcomes 可以並存
- 在 active profile 下導出一個 accepted forum reading
- core protocol 保持中立且純技術化

另見：

- [`DESIGN-NOTES.forum-qa-relationship.zh-TW.md`](./DESIGN-NOTES.forum-qa-relationship.zh-TW.md)：說明 Forum 與 Q&A app-layer examples 的邊界

## 0. 目標

讓 Mycel 可以承載一個可持久保存的 forum system，同時不把 core protocol 變成 forum-specific 的 primitive set。

放在 Mycel 裡：

- forum app definition
- board state
- thread state
- post state
- moderation actions
- accepted-thread 與 accepted-board reading state
- optional notification 或 indexing effect history

留在 Mycel core 外：

- ranking algorithms 作為 protocol rules
- anti-spam heuristics 作為 protocol rules
- private trust scores
- delivery infrastructure
- search-engine internals
- secrets 與 runtime credentials

## 1. 設計規則

forum app layer 應遵守六條規則。

1. boards、threads 與 posts 是 app-layer objects，不是 protocol primitives。
2. individual posts 應能被獨立定址，也能獨立複製。
3. 合規的 reader client 應在 active profile 下顯示一個 accepted reading 的 thread 或 board。
4. alternative moderation outcomes 應保留可審計性，並在 policy 允許時以 alternatives 形式可見。
5. moderation history 應是顯式物件，而不是無聲狀態變更。
6. 大型 forum views 應由已驗證物件導出的 local indexes 支撐，而不是依賴單一龐大的 thread object。

## 2. 建議形狀

建議的形狀是：

- `board`、`thread` 與 `post` 作為分離的 object families
- 一個或多個 resolution documents 用來導出 accepted forum reading
- moderation 作為顯式簽章 actions
- optional derived local indexes 來加速 list 與 thread rendering

這比較接近 post-centric forum，而不是 document-centric forum。

偏好這個形狀的原因：

- 單一 reply 可以獨立複製
- moderation 可以只針對一篇 post，而不必重寫整條 thread
- thread rendering 可以靠 derived indexes 擴展
- branch divergence 更容易被檢查

## 3. 三層分工

### 3.1 Client Layer

client 是 reader 與 authoring surface。

責任：

- 瀏覽 boards
- 開啟 threads
- 以 chronological 或 policy-derived order 渲染 replies
- 顯示 accepted reading 與 moderation status
- 建立 post intents 與 moderation intents
- 檢查歷史與 alternatives

非責任：

- 不重定義 accepted-reading 規則
- 不隱藏 policy 要求保留的 audit history
- 不在 Mycel objects 外做 silent state mutation

### 3.2 Runtime Layer

runtime 是 optional 且 assistive 的層。

責任：

- 維護 derived board 與 thread indexes
- 生成 notification effects
- 支援 bounded search 或 feed materialization
- 為 optional external actions 發布 effect receipts

非責任：

- 不可自行決定 accepted moderation state
- 不可覆寫 profile-governed visibility
- 不可把 ranking heuristics 當成 protocol truth

### 3.3 Effect Layer

effect layer 是 optional 的。

例子：

- subscriber notification delivery
- digest generation
- bounded search-index refresh
- 對已核准 external surface 的 bridge delivery

effect objects 應保持顯式、可審計且 replay-safe。

## 4. 核心 Forum 物件

### 4.1 Forum App Manifest

定義 forum application 本身。

建議欄位：

- `app_id`
- `app_version`
- `board_documents`
- `thread_documents`
- `post_documents`
- `resolution_documents`
- `moderation_documents`
- `allowed_effect_types`
- `runtime_profile`

用途：

- 識別 forum app
- 宣告參與的 document families
- 宣告允許的 effect classes

### 4.2 Board Document

表示一個 board 或 category。

建議欄位：

- `board_id`
- `app_id`
- `slug`
- `title`
- `description`
- `posting_policy`
- `moderation_policy_ref`
- `created_at`
- `updated_at`

用途：

- 定義一個 forum surface
- 宣告 local policy context
- 提供 threads 的穩定掛載點

### 4.3 Thread Document

表示一個 thread root。

建議欄位：

- `thread_id`
- `board_id`
- `opened_by`
- `title`
- `opening_post`
- `status`
- `tags`
- `created_at`
- `updated_at`

建議的 `status` 值：

- `open`
- `locked`
- `resolved`
- `archived`
- `hidden`

### 4.4 Post Document

表示一篇可獨立複製的 post。

建議欄位：

- `post_id`
- `thread_id`
- `board_id`
- `reply_to`
- `posted_by`
- `body`
- `edit_policy`
- `created_at`
- `supersedes_post`

用途：

- 承載一個 atomic forum contribution
- 支援 reply trees
- 透過 supersession 保留 edit history，而不是 silent overwrite

### 4.5 Moderation Action Document

表示一個帶簽章的 moderation action。

建議欄位：

- `moderation_action_id`
- `target_kind`
- `target_id`
- `action_kind`
- `issued_by`
- `reason_code`
- `reason_summary`
- `issued_at`
- `supersedes_action`

建議的 `action_kind` 值：

- `hide-post`
- `unhide-post`
- `lock-thread`
- `unlock-thread`
- `pin-thread`
- `unpin-thread`
- `move-thread`
- `label-thread`
- `archive-thread`

moderation 應保持顯式且可檢查。

### 4.6 Thread Resolution Document

表示一條 thread 的 accepted reading state。

建議欄位：

- `thread_resolution_id`
- `thread_id`
- `accepted_posts`
- `hidden_posts`
- `pinned_reply_order`
- `accepted_under_profile`
- `decision_trace_ref`
- `updated_at`

用途：

- 定義 default reader 應看到什麼
- 保留 visibility 與 ordering decisions
- 指向導出這個結果所依據的 profile

### 4.7 Board Resolution Document

表示 board-level 的 accepted state。

建議欄位：

- `board_resolution_id`
- `board_id`
- `visible_threads`
- `pinned_threads`
- `hidden_threads`
- `accepted_under_profile`
- `updated_at`

用途：

- 定義 default board listing state
- 支援 pinned 與 hidden thread 行為
- 讓 board rendering 由 profile-governed 狀態導出，而不是臨時本地偏好

## 5. Thread Resolution 範例

```json
{
  "type": "forum_thread_resolution",
  "thread_resolution_id": "tres:8c0b7d10",
  "app_id": "app:forum-main",
  "thread_id": "thr:92ab771e",
  "accepted_posts": [
    "post:001",
    "post:002",
    "post:004"
  ],
  "hidden_posts": [
    "post:003"
  ],
  "pinned_reply_order": [
    "post:001",
    "post:002",
    "post:004"
  ],
  "accepted_under_profile": "policy:forum-main-v1",
  "decision_trace_ref": "trace:3f91aa72",
  "updated_at": 1772942400
}
```

這展示了一個常見的 Mycel-style forum 模式：

- thread history 裡存在多篇 posts
- 並不是所有 posts 都 default-visible
- visibility 是顯式的
- default order 是顯式的
- 一個 profile 決定 accepted reading

## 6. Accepted Reading Model

forum app 應沿用 Mycel 其他部分相同的 accepted-head 原則。

建議的 reader behavior：

1. 載入某個 board 的 accepted board resolution
2. 載入某個 thread 的 accepted thread resolution
3. 取回其中引用的 posts
4. 在本地驗證所有 objects
5. 渲染一個 accepted forum reading
6. 視需求暴露 alternatives 與 history

這代表：

- 「預設看到什麼」是 profile-governed 的
- 「還存在什麼」仍然保持可審計
- moderation 不是藏在本地自由裁量狀態後面

## 7. Moderation Model

moderation 應被建模成帶簽章的 app-layer state，而不是隱藏資料庫旗標。

建議的 moderation split：

- maintainers 或 moderators 發布 `moderation_action` objects
- resolution documents 把這些 actions 納入 accepted reading
- clients 同時顯示結果狀態與足夠的 trace context 來解釋它

client 應能回答：

- 為什麼這篇 post 被 hidden
- 為什麼這條 thread 被 locked
- 是哪個 profile 或 maintainer set 讓這個結果變成 active

## 8. Forks 與 Disputes

建立在 Mycel 上的 forum app，不應假裝 moderation disputes 永遠不會分叉。

可能的結果：

- 兩組 moderators 發布不同的 thread resolutions
- 一個 reader profile 採用其中一條 branch
- 另一個 profile 採用另一條 branch
- 兩者都保持可檢查

這正是 forum semantics 適合落在 Mycel 上的重要原因之一：

- disputes 保持可見
- history 保持可重播
- 一方不需要先抹除另一方，才能發布自己的 accepted reading

## 9. Scaling 與 Local Indexes

實用的 forum client 應使用可重建的 local indexes。

有用的 indexes 包括：

- board-to-thread index
- thread-to-post index
- reply-tree index
- moderation-action-by-target index
- accepted-resolution index

這些 indexes 應：

- 從已驗證物件導出
- 可只靠 canonical data 重建
- 被視為 local acceleration structures，而不是 portable truth

## 10. 非目標

這份筆記不主張：

- protocol-level forum primitives
- protocol-level ranking rules
- global upvote consensus
- 在 protocol core 裡解決 spam prevention
- private-message secrecy model
- 大規模 public search architecture

這些要嘛是 app-policy 問題、runtime 問題，或更後面的 deployment 問題。

## 11. 為什麼這適合 Mycel

forum app 和 Mycel 的相性其實很高，因為論壇需要：

- 可持久保存的文本歷史
- 顯式 moderation history
- disputes 發生時的 branch tolerance
- 由治理導出的 default reading
- object-level replication

因此，forum app 很適合作為 Mycel 的 app-layer example，雖然它仍應留在 protocol core 外。

## 12. 建議下一步

如果要沿著這個方向往前走，下一個具體步驟應是以下其中之一：

1. 一份 minimal forum schema note，附帶 example JSON envelopes
2. 一組 fixture-backed sample objects，包含一個 board、一條 thread 與一次 moderation split
3. 一份 reader-surface note，說明 thread rendering 與 trace inspection
