# Commentary and Citation Schema

狀態：design draft

這份筆記定義一套狹窄的 app-level schema，供 maintainer 編寫的 commentary works 在大量引用一份或多份 source documents 時使用，同時不覆寫原文。

這些 schema 不是新的 core protocol primitives。
它們是要放在一般 Mycel documents 裡、作為 app-layer model 的邏輯記錄形狀。

## 0. 目標

讓 Mycel deployments 能支援這種模式：

- 一份 source document 保持為 root text
- 某位 maintainer 或 editor 發布一份獨立 commentary work
- commentary work 大量引用 source text
- reader 可以同時檢視 commentary 與 source，而不是把兩者折疊成同一份被改寫過的文件

這份筆記在內容領域上保持中立。

它可適用於法律註解、技術註釋、學術評註、編輯說明，或由 profile 治理的 interpretation layers。

## 1. 核心規則

commentary work 必須與被引用的 source work 保持區分。

以下幾種東西都應保留為彼此分離的層：

- source text
- commentary text
- citation links
- 對 commentary 或 interpretation 的 profile-specific acceptance

commentary 不可默默偽裝成 source witness。

## 2. 範圍

這份最小 schema 涵蓋四種 record families：

- `commentary_work`
- `commentary_section`
- `citation_set`
- `commentary_resolution`

目標是在不擴張 protocol core 的前提下，讓大量交叉引用的 commentary 具備可審查、可機器檢查、且對 UI 友善的形狀。

## 3. 一般規則

1. 所有 records 都是 app-level JSON payload；若經過一般 document flow 做簽章或雜湊，應遵守 Mycel 的 canonical serialization 規則。
2. 這裡出現的 ID 都是 app-level logical IDs，不是新的 protocol object types。
3. 在 strict profile 下，未知欄位應拒絕；只有 app profile 明確允許 extension 時，才可忽略未知欄位。
4. 時間戳一律使用 Unix 秒整數。
5. commentary document 即使緊密對應某個 source work，也應保有自己的 `doc_id`。
6. 跨文件引用應優先使用穩定的 logical references，例如 `source_id` 加上 `locator`；若 profile 有需要，也可要求 version-locking 欄位。

## 4. 共用欄位慣例

### 4.1 共用必要欄位

每種 record family 都應帶有：

- `type`
- `app_id`
- `created_at`
- `updated_at`

### 4.2 建議 ID Prefixes

建議的 logical ID prefixes：

- `cw:` 給 commentary works
- `cs:` 給 commentary sections
- `cit:` 給 citation sets
- `cr:` 給 commentary resolutions

這些 prefixes 只是 app-level 慣例。

## 5. Commentary Work Schema

### 5.1 必要欄位

- `type`：必須是 `commentary_work`
- `commentary_id`：commentary work 的 logical ID
- `app_id`
- `doc_id`：承載 commentary text 的 Mycel document ID
- `title`
- `commentary_kind`：`editorial`、`gloss`、`interpretation`、`study-note`、`practical-guidance`、`comparative-note` 之一
- `source_documents`：非空的 source document IDs 陣列
- `authored_by`：maintainer 金鑰或 app-level actor ID
- `created_at`
- `updated_at`

### 5.2 可選欄位

- `language`
- `summary`
- `audience_label`
- `supersedes_commentary`
- `default_citation_policy`
- `active_resolution_id`

### 5.3 範例

```json
{
  "type": "commentary_work",
  "commentary_id": "cw:main-commentary-a",
  "app_id": "app:commentary-reference",
  "doc_id": "doc:commentary-main-a",
  "title": "Maintainer Notes on Source A",
  "commentary_kind": "editorial",
  "source_documents": ["doc:source-a"],
  "authored_by": "pk:maintainer-014",
  "language": "en",
  "summary": "A section-by-section commentary on Source A.",
  "default_citation_policy": "exact-or-locator-only",
  "created_at": 1772941800,
  "updated_at": 1772942400
}
```

## 6. Commentary Section Schema

### 6.1 必要欄位

- `type`：必須是 `commentary_section`
- `section_id`
- `app_id`
- `commentary_id`
- `section_kind`：`overview`、`line-note`、`anchor-note`、`cross-reference`、`application-note`、`dispute-note` 之一
- `body`
- `order_key`
- `created_at`
- `updated_at`

### 6.2 可選欄位

- `title`
- `anchor_refs`：source anchors 或 block references 的陣列
- `citation_ids`：citation-set IDs 陣列
- `supersedes_section`
- `visibility_label`

### 6.3 驗證規則

- `commentary_id` 必須能對到既有的 `commentary_work`。
- 若有 `citation_ids`，則必須能對到屬於同一 `commentary_id` 的 citation sets。
- 若 section 是在描述 source text，`anchor_refs` 就不應指向 commentary sections。

### 6.4 範例

```json
{
  "type": "commentary_section",
  "section_id": "cs:note-001",
  "app_id": "app:commentary-reference",
  "commentary_id": "cw:main-commentary-a",
  "section_kind": "line-note",
  "title": "Why this phrase matters",
  "body": "This phrase narrows the scope of the surrounding obligation and should be read together with the next paragraph.",
  "order_key": "0001",
  "anchor_refs": ["block:source-a-14"],
  "citation_ids": ["cit:note-001"],
  "created_at": 1772942100,
  "updated_at": 1772942280
}
```

## 7. Citation Set Schema

### 7.1 必要欄位

- `type`：必須是 `citation_set`
- `citation_id`
- `app_id`
- `commentary_id`
- `section_id`
- `references`：非空陣列
- `created_at`
- `updated_at`

### 7.2 Reference Item Shape

`references` 裡的每個 item 應包含：

- `source_id`
- `locator`
- `relation_kind`：`supports`、`interprets`、`contrasts`、`applies`、`questions` 之一

每個 reference 的可選欄位：

- `quote`
- `note`
- `source_revision_id`
- `source_head`
- `source_profile_id`
- `anchor_hash`

### 7.3 頂層可選欄位

- `quote_policy`
- `notes`
- `source_bundle_id`

### 7.4 驗證規則

- `source_id` 應能對到某個被引用的 source document 或 source bundle。
- `locator` 應指向穩定的 logical target，例如 block ID、anchor ID、witness segment，或 app 自定的 source locator。
- strict profile 可要求至少一個 version-locking 欄位，例如 `source_revision_id`、`source_head` 或 `anchor_hash`。
- 若有 `quote`，則應能在 active profile 下對照被引用的 source 做稽核。

### 7.5 範例

```json
{
  "type": "citation_set",
  "citation_id": "cit:note-001",
  "app_id": "app:commentary-reference",
  "commentary_id": "cw:main-commentary-a",
  "section_id": "cs:note-001",
  "references": [
    {
      "source_id": "doc:source-a",
      "locator": "block:source-a-14",
      "relation_kind": "interprets",
      "quote": "review process",
      "source_revision_id": "rev:source-a-r14"
    },
    {
      "source_id": "doc:source-a",
      "locator": "block:source-a-15",
      "relation_kind": "supports",
      "note": "Read together with the immediately following block.",
      "source_head": "head:source-a-main"
    }
  ],
  "quote_policy": "exact-or-locator-only",
  "created_at": 1772942100,
  "updated_at": 1772942280
}
```

## 8. Commentary Resolution Schema

### 8.1 必要欄位

- `type`：必須是 `commentary_resolution`
- `resolution_id`
- `app_id`
- `commentary_id`
- `candidate_sections`：非空的 section IDs 陣列
- `accepted_sections`：section IDs 陣列
- `accepted_under_profile`
- `updated_at`

### 8.2 可選欄位

- `alternative_sections`
- `decision_trace_ref`
- `rationale_summary`
- `state_label`
- `created_at`

### 8.3 目的

這個 record 是可選的。

它是給需要對 commentary layers 做 governed acceptance 的 deployments 使用，例如：

- 某個 profile 下有一套 accepted maintainer commentary
- 一套 accepted commentary 加上仍可見的 alternatives
- 某些 commentary sections 被標成 advisory only

若某 deployment 不治理 commentary acceptance，可以省略這個 record family。

## 9. 最低受理規則

某個 commentary section 若要被視為可審查的 commentary，至少應提供：

- 可閱讀的 body text
- 對 source-facing claims 至少提供一個明確 anchor reference 或 citation set
- 足夠的 citation context，讓內容可被稽核

這不保證它會被接受。

它只保證這份 commentary 可以被審查。

## 10. Version-Locking 選項

不同 deployments 可能想要不同強度的引用約束。

建議分層：

- `locator-only`：只要求 `source_id` 與 `locator`
- `revision-locked`：要求 `source_revision_id`
- `accepted-head-locked`：要求具名 profile 下的 `source_head`
- `anchor-hash-locked`：要求穩定的 anchor hash 或 witness-segment hash

這應保持為 profile choice，而不是強制的 core protocol 規則。

## 11. Client 行為

合格的 reader client 應：

- 把 source text 與 commentary 呈現為分離圖層
- 讓 reader 能檢視每則 note 所對應的 cited source target
- 清楚區分 commentary acceptance 與 source acceptance
- 若 active profile 定義了 commentary governance，則顯示 unresolved 或 alternative commentary

client 不應：

- 默默用 commentary wording 改寫 source text
- 把 commentary 當成 source witness 呈現
- 在顯示 commentary snippets 時丟掉 citation context

## 12. 範例流程

建議流程：

1. maintainer 以獨立的 `doc_id` 發布一份新的 commentary document
2. 該 document 帶有一筆 `commentary_work` record 與多筆 `commentary_section` records
3. 每個 section 都用一筆或多筆 `citation_set` records 指向被引用的 source blocks 或 anchors
4. client 渲染 source text，並讓 reader 以 side by side 方式檢視 commentary
5. 若 deployment 需要治理 commentary acceptance，則再發布 `commentary_resolution`

## 13. 與其他筆記的關係

這份筆記是下列文件的 companion：

- `DESIGN-NOTES.mycel-app-layer`
- `DESIGN-NOTES.qa-minimal-schema`
- `DESIGN-NOTES.interpretation-dispute-model`
- `DESIGN-NOTES.canonical-text-profile`

它刻意不進入 protocol core，只定義 commentary-heavy documents 所需的狹窄 app-layer shape。
