# 即時影音應用層

狀態：設計草案

這份文件描述 Mycel 如何支援即時音訊 / 視訊服務，同時不把協定核心或傳輸層變成影音串流傳輸協定。

白話來說，Mycel 可以承載影音服務的控制面、記錄面與稽核面，而真正的 live 音訊 / 視訊串流則交給專門的影音協定。

## 0. 目標

讓 Mycel 可以支援直播與錄播工作流程，同時維持：

- 媒體封包傳輸留在協定核心之外
- accepted state、moderation、存取政策與記錄歷史留在 Mycel 內
- replay 保持決定性且不帶副作用

放在 Mycel 裡：

- stream 定義
- room 或 channel 狀態
- 存取與角色政策
- moderation 動作
- recording 中繼資料
- subtitle 或 caption 歷史
- 章節標記
- 衍生的播放或發布狀態
- 稽核與爭議歷史

留在 Mycel core 外：

- 低延遲媒體封包傳輸
- codec 協商
- 自適應碼率控制
- TURN / ICE / WebRTC 傳輸細節
- 即時音訊 / 視訊混流
- CDN 端的分段分發

## 1. 設計規則

這個影音 app 應遵守六條規則。

1. Revision replay MUST 保持無副作用。
2. 原始 live 媒體串流 MUST NOT 透過一般 Mycel state 被複製。
3. 存取政策與 accepted 發布狀態 MAY 由 Mycel 承載。
4. Live 傳輸 SHOULD 使用專門的影音協定。
5. Recording、subtitle、moderation 與發布歷史 SHOULD 保持可稽核。
6. 節點 MUST 能在不儲存所有媒體資產的情況下參與系統。

## 2. 分層拆分

### 2.1 Client Layer

client 是面向使用者的層。

責任：

- 顯示 live session 中繼資料
- 顯示 accepted channel 或 room 狀態
- 顯示存取條件與參與者角色
- 顯示 moderation 狀態
- 顯示 recording、caption 與發布歷史
- 讓使用者建立經核准的應用層 intent，例如 publish、annotate、caption 或 moderate

非責任：

- 不透過一般 Mycel objects 承載原始 live 媒體封包
- 不重定義傳輸或 codec 行為
- 不繞過存取政策或 moderation 規則

### 2.2 Media Runtime Layer

media runtime 在 Mycel core 之外執行 live 服務行為。

責任：

- 操作 WebRTC、RTMP、HLS、SRT 或類似的影音傳輸
- 管理媒體 ingest、轉送或分段生成
- 讀取 accepted 的 Mycel channel、存取與 moderation 政策
- 把 receipts 或摘要再寫回 Mycel

非責任：

- 不重定義協定驗證
- 不把未 accepted branch state 視為 live 政策
- 不讓媒體傳輸端的真相凌駕已簽署的 Mycel 記錄

### 2.3 記錄與效果層

效果層負責明確表示外部影音側的動作與結果。

例如：

- 開始直播 session
- 輪替 stream key
- 建立 recording
- 發布 subtitle 批次
- 撤銷 viewer 角色
- 標記 recording 為已發布
- 記錄 moderation 執行結果

## 3. 核心物件家族

### 3.1 Channel 或 Room Document

表示一個長期存在的影音空間。

建議欄位：

- `channel_id`
- `display_name`
- `role_policy`
- `access_policy`
- `recording_policy`
- `moderation_policy`
- `publication_policy`
- `active_runtime_refs`

用途：

- 定義一個 room、stream 或 channel
- 宣告誰可以發布、moderate 或觀看
- 定義哪些 runtime 可以操作它

### 3.2 Session Document

表示一場 live session 或廣播時段。

建議欄位：

- `session_id`
- `channel_id`
- `started_at`
- `ended_at`
- `runtime_ref`
- `status`
- `ingest_summary`
- `session_digest`

常見 `status` 值：

- `scheduled`
- `live`
- `ended`
- `failed`

### 3.3 Access 與 Role Document

表示 accepted 的 viewer、publisher、moderator 或 captioner 權限。

建議欄位：

- `subject_ref`
- `channel_id`
- `role`
- `grant_state`
- `granted_by`
- `granted_at`
- `revoked_at`

常見 `role` 值：

- `viewer`
- `publisher`
- `moderator`
- `captioner`

### 3.4 Moderation Document

表示明確的 moderation 狀態與動作。

建議欄位：

- `action_id`
- `channel_id`
- `target_ref`
- `action_kind`
- `reason`
- `issued_by`
- `issued_at`
- `status`

例如：

- mute publisher
- 移除 viewer
- 暫停聊天連動註記
- 下架 recording

### 3.5 Recording Document

表示一份 recording 或典藏物件。

建議欄位：

- `recording_id`
- `session_id`
- `storage_ref`
- `media_digest`
- `duration_ms`
- `published_state`
- `visibility`
- `created_at`

用途：

- 識別這份 recording
- 把它綁定到某一場 session
- 宣告它目前是已發布還是隱藏
- 在不把原始媒體放進一般 Mycel state 的前提下支援稽核

### 3.6 Subtitle / Caption Document

表示 subtitle 或 caption 歷史。

建議欄位：

- `caption_batch_id`
- `session_id`
- `language`
- `segment_refs`
- `editor_ref`
- `created_at`
- `revision_digest`

這是 Mycel 很適合承載的部分，因為 subtitle 與 caption 歷史非常需要可驗證的修訂軌跡。

### 3.7 Publication 與 Playback View

表示 accepted 的預設播放狀態。

例如：

- 哪一份 recording 是預設發布版本
- 哪一條 caption track 是 accepted 的預設版本
- 哪一組 moderation 狀態目前生效
- 哪些 chapter markers 預設被採用

這正是 Mycel accepted-state 模型有價值的地方：預設影音 view 不需要是全域共識，只需要是在固定規則下導出的結果。

## 4. 建議執行流程

1. client 或服務建立 session intent 或 session update。
2. media runtime 在 Mycel 之外操作 live 傳輸。
3. runtime 把 session 摘要、recording 中繼資料與 effect receipts 寫進 Mycel。
4. moderation、captions 與發布決策累積成已簽署歷史。
5. 固定 profile 或 app 政策導出 accepted 的預設播放狀態。
6. client 呈現 accepted 播放狀態，同時仍允許稽核其他仍然有效的分支。

## 5. Mycel 為何適合這一層

Mycel 適合影音服務，是因為它能保留：

- subtitle、moderation 與發布狀態的可驗證修訂歷史
- 依固定規則導出的預設播放版本
- 中繼資料與治理狀態的去中心化複製
- 為何某個發布 view 成為預設版本的稽核軌跡

它並不是要取代：

- WebRTC
- RTMP
- HLS
- 媒體 CDN 分發
- codec 或 jitter 控制

## 6. 目前建議的 Mycel 立場

目前建議：

- 先把即時影音維持在應用層設計備忘錄階段
- 把 live 音訊 / 視訊傳輸留在 Mycel core 與傳輸協定之外
- 讓 Mycel 承載 accepted state、政策、歷史與稽核表面
- 若未來 `M5` 的應用層擴展真的走向影音服務，再回頭評估是否需要更正式的 runtime / profile 模式
