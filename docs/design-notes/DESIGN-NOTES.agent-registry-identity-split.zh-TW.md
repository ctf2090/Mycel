# Agent-Registry Identity Split Model

狀態：design draft

這份筆記提議把 local agent registry 裡目前混在一起的 agent 身分與人類可讀短 id 拆開：

- `agent_uid`：真正的 agent 身分，永不重用
- `display_id`：人類可讀的短 id，例如 `coding-1`，可以回收再分配

目標是同時解決兩件事：

- 讓顯示用 id 可以收斂，不會只增不減
- 避免舊 chat 與新 chat 因為共用 `coding-1` 而發生身分撞號

這份文件是未來設計草案，不會直接覆寫目前 [AGENT-REGISTRY.md](../AGENT-REGISTRY.md) 的 active protocol。

## 0. 問題

目前的 local registry 用單一欄位同時承擔：

- agent 的真實身份
- CLI 的操作主鍵
- 對人顯示的短名稱

這會產生兩個互相拉扯的需求：

1. 如果 id 永不重用，resume 與審計最安全，但 `coding-17` 這類 id 會一直長。
2. 如果 id 可重用，顯示比較收斂，但舊 chat 與新 chat 很容易因為共用 `coding-1` 而混成同一個 agent。

`agent_uid + 可回收 display_id` 的做法，是把這兩個需求拆成不同欄位處理。

## 1. 目標

保留：

- 每個 chat 的穩定 agent 身分
- 舊 chat 回來後可被明確識別
- 完整的 assignment / resume / recover / takeover 審計線

新增：

- `coding-1`、`doc-1` 這種顯示 id 的回收能力
- 對舊 chat 的安全 resume 判定
- 對同 role 多 chat 的可預期 display-slot 分配

不追求：

- 強密碼學層級的 chat 身分驗證
- 跨機器同步 registry
- 向後相容於舊版 registry schema

## 2. 名詞

### 2.1 `agent_uid`

agent 的真正身份主鍵。

特性：

- 在 claim 時建立
- 永不重用
- CLI 的狀態改寫操作都應以它為主鍵
- mailbox 與審計歷史應綁在它身上

建議格式：

- `agt_` 前綴加隨機字串，例如 `agt_7b2e9f4c`

### 2.2 `display_id`

只給人看的短 id。

特性：

- 格式仍維持 `coding-1`、`doc-1`
- 一次只會綁定到一個當前 agent
- 可在釋放後回收給下一個 agent
- 不應再當作真正身份主鍵

### 2.3 Display Slot

一個 role 下的某個數字位置，例如 `coding-1`、`coding-2`。

slot 是可重用的。
`display_id` 代表該 slot 目前的租用狀態，不代表跨時間的永久身份。

## 3. 提議資料模型

### 3.1 Registry v2 Top-Level Shape

```json
{
  "version": 2,
  "updated_at": "2026-03-12T12:00:00+0800",
  "agent_count": 2,
  "agents": [
    {
      "agent_uid": "agt_a1b2c3d4",
      "role": "coding",
      "current_display_id": "coding-1",
      "display_history": [
        {
          "display_id": "coding-1",
          "assigned_at": "2026-03-12T11:00:00+0800",
          "released_at": null,
          "released_reason": null
        }
      ],
      "assigned_by": "user",
      "assigned_at": "2026-03-12T11:00:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-12T11:01:00+0800",
      "last_touched_at": "2026-03-12T11:10:00+0800",
      "inactive_at": null,
      "status": "active",
      "scope": "forum inbox sync",
      "files": [],
      "mailbox": ".agent-local/mailboxes/agt_a1b2c3d4.md",
      "recovery_of": null,
      "superseded_by": null
    },
    {
      "agent_uid": "agt_e5f6g7h8",
      "role": "doc",
      "current_display_id": "doc-1",
      "display_history": [
        {
          "display_id": "doc-1",
          "assigned_at": "2026-03-12T11:05:00+0800",
          "released_at": null,
          "released_reason": null
        }
      ],
      "assigned_by": "user",
      "assigned_at": "2026-03-12T11:05:00+0800",
      "confirmed_by_agent": true,
      "confirmed_at": "2026-03-12T11:06:00+0800",
      "last_touched_at": "2026-03-12T11:15:00+0800",
      "inactive_at": null,
      "status": "active",
      "scope": "registry design note",
      "files": [],
      "mailbox": ".agent-local/mailboxes/agt_e5f6g7h8.md",
      "recovery_of": null,
      "superseded_by": null
    }
  ]
}
```

### 3.2 Required Fields

Top level：

- `version`
- `updated_at`
- `agent_count`
- `agents`

Per agent：

- `agent_uid`
- `role`
- `current_display_id`
- `display_history`
- `assigned_by`
- `assigned_at`
- `confirmed_by_agent`
- `confirmed_at`
- `last_touched_at`
- `inactive_at`
- `status`
- `scope`
- `files`
- `mailbox`
- `recovery_of`
- `superseded_by`

### 3.3 Field Rules

- `agent_uid` 是 registry 主鍵。
- `current_display_id` 可以是 `null`。
- `current_display_id != null` 時，該值必須在所有 agent 間唯一。
- `display_history` 必須依時間排序，最後一筆代表最近一次 display slot 指派。
- `display_history` 中任一筆 `released_at == null` 的紀錄，必須對應到 `current_display_id`。
- `mailbox` 應改成依 `agent_uid` 命名，不再依 `display_id` 命名。
- `recovery_of` 用於標記此 agent 是否由別的 stale agent takeover 而來。
- `superseded_by` 用於標記此 agent 是否已由別的 agent 接手。

### 3.4 為什麼 `current_display_id` 可以是 `null`

這是讓 display slot 可回收的關鍵。

當 agent 本身還需要保留在 registry 裡做審計，但它的 display slot 已經釋放給別人時：

- agent entry 仍保留
- `agent_uid` 不變
- `current_display_id` 變成 `null`
- `display_history` 裡最後一筆會有 `released_at`

## 4. Slot 回收規則

### 4.1 Stale 與 Slot Release

沿用目前的 command-level lease 概念：

- `finish` 後 agent 變成 `inactive`
- `inactive` 超過 1 小時後變成 stale

新設計下，stale agent 不再保留原 display slot。
系統可以在 `cleanup` 或下一次 registry 寫入前，釋放它的 display slot：

- `current_display_id` 設為 `null`
- 更新最後一筆 `display_history.released_at`
- `released_reason = "stale-recycled"`

agent entry 本身仍保留，所以審計與 resume 判斷不會遺失。

### 4.2 新 Claim 的 Slot 分配

新的 display slot 應取該 role 的「最小可用正整數 suffix」，不是歷史最大值加一。

例子：

- 現在使用中的 slot 是 `coding-2` 與 `coding-4`
- 可用 slot 是 `coding-1`、`coding-3`
- 新 claim 應拿 `coding-1`

這就是 display id 收斂的來源。

## 5. Lifecycle

### 5.1 新 Chat Claim

1. 建立新的 `agent_uid`
2. 依 role 選可用的最小 `display_id`
3. 建立 mailbox：`.agent-local/mailboxes/<agent_uid>.md`
4. 寫入新 agent entry
5. 回傳 `agent_uid` 與 `display_id`

### 5.2 Start

`start <agent_uid>` 只確認這個 agent entry：

- `confirmed_by_agent = true`
- `confirmed_at = now`
- `status = active`
- `inactive_at = null`

對外顯示仍用 `display_id`，但底層寫入主鍵是 `agent_uid`。

### 5.3 每次 User Command

1. `touch <agent_uid>`
2. 執行工作
3. `finish <agent_uid>`

這和目前規則一樣，只是 CLI 主鍵改成 `agent_uid`。

### 5.4 舊 Chat 在 Slot 尚未被回收前回來

如果 A 的 `agent_uid` 還持有 `current_display_id = coding-1`，則：

- `resume-check <agent_uid=A>` 應回傳 `safe_to_resume = true`
- A 可以直接 `touch A`
- 對外仍自稱 `coding-1`

### 5.5 舊 Chat 在 Slot 已被回收並重分配後回來

如果：

- A 還在 registry 裡
- A 的 `current_display_id = null`
- B 已拿到 `coding-1`

則：

- `resume-check <agent_uid=A>` 不可回傳 direct resume
- 應回傳 `must_recover = true`
- A 不能再直接碰 `coding-1`
- A 必須走 `recover <agent_uid=A>`

recover 完成後，A 會拿到新的 display slot，例如 `coding-2`。

### 5.6 不同 Chat 的 Takeover

如果不是 A 自己回來，而是另一個新 chat 要接手 A 的 scope，則不應重用 A 的 `agent_uid`。

應改走 takeover：

1. 新 chat claim 新的 `agent_uid`
2. 新 chat 拿新的 `display_id`
3. 舊 A 標記 `superseded_by = <new-agent-uid>`
4. 新 agent 標記 `recovery_of = <old-agent-uid>`
5. 新 mailbox 追加 takeover note

這代表：

- self-resume 是保留同一個 `agent_uid`
- takeover 是新的 `agent_uid`

## 6. 提議 CLI 行為

## 6.1 Primary Keys

提議從這版開始：

- 寫入型命令預設只接受 `agent_uid`
- 查詢型命令可以接受 `agent_uid` 或 `display_id`

理由是：

- `display_id` 可能重用
- `agent_uid` 不會重用

### 6.2 `claim`

```text
scripts/agent_registry.py claim <role|auto> [--scope <scope>] [--json]
```

輸出：

- `agent_uid`
- `display_id`
- `role`
- `scope`
- `mailbox`

claim 時就建立 `agent_uid`，並分配最小可用 `display_id`。

### 6.3 `start`

```text
scripts/agent_registry.py start <agent_uid> [--json]
```

行為：

- 確認 `agent_uid` 存在
- 如果 `current_display_id` 是 `null`，拒絕 start，要求先 recover
- 成功時回傳 `agent_uid` 與 `display_id`

### 6.4 `status`

```text
scripts/agent_registry.py status [--agent-uid <agent_uid> | --display-id <display_id>] [--json]
```

預設列出所有 agents，且每筆都顯示：

- `agent_uid`
- `current_display_id`
- `status`
- `scope`
- `last_touched_at`
- `inactive_at`

若查詢 `display_id`，只查目前持有該 slot 的 agent，不查歷史持有者。

### 6.5 `touch`

```text
scripts/agent_registry.py touch <agent_uid> [--json]
```

行為：

- 若 `current_display_id` 是 `null`，拒絕 touch，要求先 recover
- 否則更新 `last_touched_at` 並設為 `active`

### 6.6 `finish`

```text
scripts/agent_registry.py finish <agent_uid> [--json]
```

行為：

- 設為 `inactive`
- 更新 `inactive_at`
- 不釋放 display slot

slot 釋放是 stale recycle 或 explicit recover 時的事，不是 finish 當下就做。

### 6.7 `resume-check`

```text
scripts/agent_registry.py resume-check <agent_uid> [--json]
```

輸出應至少包含：

- `agent_uid`
- `current_display_id`
- `safe_to_resume`
- `must_recover`
- `recommended_action`
- `reason`

判斷規則：

1. 如果 agent 已 `paused`、`done`、`blocked`，回傳 stop。
2. 如果 `current_display_id != null`，回傳 direct resume。
3. 如果 `current_display_id == null`，回傳 `must_recover = true`。

### 6.8 `recover`

```text
scripts/agent_registry.py recover <agent_uid> [--scope <scope>] [--json]
```

這裡的 `recover` 指的是「同一個 agent 身分恢復工作，但需要新的 display slot」。

行為：

1. 確認該 `agent_uid` 存在
2. 確認該 agent 不是 `done`
3. 確認目前 `current_display_id == null`
4. 指派新的最小可用 `display_id`
5. 在 `display_history` 追加新紀錄
6. 若有傳 `--scope` 則更新 scope
7. 回傳新的 `display_id`

recover 後：

- `agent_uid` 不變
- `display_id` 會變
- mailbox 不變

### 6.9 `takeover`

```text
scripts/agent_registry.py takeover <stale-agent-uid> [--scope <scope>] [--json]
```

這是新的命令，與 `recover` 分開。

用途：

- 不是原 agent 回來
- 而是另一個 chat 要接手 stale agent 的工作

行為：

1. 建立新的 `agent_uid`
2. 分配新的 `display_id`
3. 舊 agent 設 `superseded_by = <new-agent-uid>`
4. 新 agent 設 `recovery_of = <old-agent-uid>`
5. 產生新的 mailbox

## 7. Edge Case Walkthrough

情境：

- A 原本是 `agent_uid=A`, `display_id=coding-1`
- A 做完工作後變成 `inactive`
- 一小時後 A stale，slot 被回收
- 新 chat B claim，拿到 `agent_uid=B`, `display_id=coding-1`
- 後來使用者又回到舊 chat A

這時應發生：

1. A 跑 `resume-check A`
2. 系統看到 A 的 `current_display_id == null`
3. 系統回傳 `must_recover = true`
4. A 不能直接 `touch A`
5. A 跑 `recover A`
6. 系統分配下一個可用 slot，例如 `coding-2`
7. A 之後以 `display_id=coding-2` 繼續工作

結果：

- B 保持 `coding-1`
- A 安全地變成 `coding-2`
- 兩者不會撞身份

## 8. Migration 建議

### 8.1 Schema Migration

把舊版每個 agent entry：

- `id` 轉成 `current_display_id`
- 新增 `agent_uid`
- 新增 `display_history`

若舊版 mailbox 是 `.agent-local/coding-1.md` 這種 display-id 路徑，migration 後建議：

- 保留舊檔
- 搬移或複製到 `.agent-local/mailboxes/<agent_uid>.md`
- registry 一律改指向 uid-based mailbox

### 8.2 CLI Migration

建議分兩階段：

1. 過渡期
   - `touch/start/finish/stop/resume-check/recover` 同時接受 `agent_uid` 與 `display_id`
   - 若收到 `display_id`，輸出 deprecation warning
2. 穩定期
   - 寫入型命令只接受 `agent_uid`
   - `display_id` 只留給查詢與顯示

## 9. 為什麼這比「永不重用 id」更好

和目前方案相比，這個模型：

- 保留舊 chat 的穩定身份
- 讓顯示 id 可以收斂
- 不需要靠 tombstone 避免撞號
- 能清楚分離 self-resume 與 takeover

代價是：

- schema 較複雜
- CLI 要從 display-id-first 改成 uid-first
- 文件與測試都要重寫

## 10. 開放問題

1. 是否需要再加 `session_token`，避免別的 chat 冒用同一個 `agent_uid`。
2. `takeover` 是否應該沿用 `recover` 名稱，還是明確拆成兩個命令。
3. stale 後多久釋放 display slot，是否要與目前 1 小時 inactive TTL 共用同一個門檻。
4. `status` 預設輸出是否應該把 `agent_uid` 縮短顯示，避免對人閱讀太長。
