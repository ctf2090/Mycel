# Mycel Protocol v0.1

語言：繁體中文 | [English](./PROTOCOL.en.md)

## 0. 定位

Mycel 是一種具備以下特性的文本協議：

- Git 式版本模型
- P2P 複製
- 簽章驗證
- 多分支共存
- 不要求全域單一共識

它不是區塊鏈，也不是 Git 複製品；它是為文字與知識內容而設計的，去中心、可分叉、可驗證歷史的協議。

適用場景包含：

- 長期文本
- 註解
- 宣言文件
- 社群章程
- 規範文件
- 去中心 wiki
- 難以被刪除的知識網

## 1. 設計目標

Mycel 的設計目標：

1. **可驗證歷史**：所有被接受的修改都必須可追溯、可重放驗證。
2. **去中心存活**：在沒有單一伺服器時，內容仍可保存與同步。
3. **分支合法**：分叉是第一級合法狀態，不是錯誤。
4. **合併可選**：社群可發布已簽章的治理 View，而 reader client 依固定 profile 規則導出 accepted head。
5. **匿名可用**：作者可使用假名金鑰，並應最小化 metadata 暴露。
6. **文本優先（v0.1）**：在 v0.1 以 block / paragraph 為主要操作單位。

## 2. 協議概念

Mycel 把資料拆成 6 種核心概念：

- **Document**：文件
- **Block**：段落/區塊
- **Patch**：一次修改
- **Revision**：某個可驗證狀態
- **View**：用來導出 accepted head 的已簽章治理訊號
- **Snapshot**：某一時刻的快照包

## 3. 基本原則

### 3.1 邏輯 ID 與 canonical object ID

Mycel 使用兩種不同的識別子類別：

- **邏輯 ID**：文件狀態內的穩定參照，例如 `doc_id` 與 `block_id`
- **Canonical object ID**：可複製物件的內容定址 ID，例如 `patch_id`、`revision_id`、`view_id`、`snapshot_id`

邏輯 ID 屬於應用層狀態，MUST NOT 被解讀為內容雜湊。
Canonical object ID 由 canonical bytes 導出：

```text
object_hash = HASH(canonical_serialization(object_without_derived_ids_or_signatures))
object_id = <type-prefix>:<object_hash>
```

在 v0.1：

- `doc_id` 與 `block_id` 是邏輯 ID
- `patch_id`、`revision_id`、`view_id`、`snapshot_id` 是 canonical object ID
- 導出 ID 欄位本身與 `signature` 欄位 MUST NOT 納入 hash 輸入

這個拆分可避免自我遞迴雜湊，也讓 transport 參照保持明確。

### 3.2 簽章必須存在

所有作者產生的 Patch、Revision、View 都必須有數位簽章。
所有 v0.1 物件型別的簽章要求，以第 6.4 節為規範性定義。

### 3.3 多個 head 合法

同一個文件可以有多個 heads。

### 3.4 Accepted Head 由 Profile 治理，而非全域真理

所謂「採信版本」只是某個治理 View profile 的輸出，不是全網唯一版本。
不同合法 profile 可以並存，但合規的 reader client MUST 以固定的 protocol-defined profile 輸入導出 active accepted head，而不是依本地偏好自由裁量。

### 3.5 傳輸與接受分離

節點可以接收某 object，但不讓它進入 profile-governed accepted-head 路徑。
物件只有在完整驗證且符合固定 selector profile 的條件後，才會影響 accepted-head selection。

## 4. 物件模型

### 4.1 Document

Document 定義一份文本的身份與基礎設定。

```json
{
  "type": "document",
  "version": "mycel/0.1",
  "doc_id": "doc:origin-text",
  "title": "Origin Text",
  "language": "zh-Hant",
  "content_model": "block-tree",
  "created_at": 1777777777,
  "created_by": "pk:authorA",
  "genesis_revision": "rev:0ab1"
}
```

欄位：

- `doc_id`：文件固定邏輯 ID，不是內容雜湊
- `title`：標題
- `language`：語言
- `content_model`：內容模型，v0.1 固定為 `block-tree`
- `genesis_revision`：初始 revision

### 4.2 Block

Block 是最小文本結構單位。

```json
{
  "type": "block",
  "block_id": "blk:001",
  "block_type": "paragraph",
  "content": "起初沒有終稿，只有傳遞。",
  "attrs": {},
  "children": []
}
```

`block_type` 可用值：

- `title`
- `heading`
- `paragraph`
- `quote`
- `verse`
- `list`
- `annotation`
- `metadata`

`block_id` 是文件狀態內的邏輯 block 參照，不是內容雜湊。

### 4.3 Patch

Patch 表示一次對文件的修改。

```json
{
  "type": "patch",
  "version": "mycel/0.1",
  "patch_id": "patch:91ac",
  "doc_id": "doc:origin-text",
  "base_revision": "rev:0ab1",
  "author": "pk:authorA",
  "timestamp": 1777778888,
  "ops": [
    {
      "op": "replace_block",
      "block_id": "blk:001",
      "new_content": "起初沒有終稿，只有傳遞與再寫。"
    },
    {
      "op": "insert_block_after",
      "after_block_id": "blk:001",
      "new_block": {
        "block_id": "blk:002",
        "block_type": "paragraph",
        "content": "凡被寫下者，皆可再寫。",
        "attrs": {},
        "children": []
      }
    }
  ],
  "signature": "sig:..."
}
```

Patch 的簽章輸入至少要包含：

- `type`
- `version`
- `doc_id`
- `base_revision`
- `timestamp`
- `author`
- `ops`

`patch_id` 是導出的 canonical object ID，格式為 `patch:<object_hash>`。
它 MUST 由省略 `patch_id` 與 `signature` 後的 canonical Patch 內容計算而得。

對 v0.1 的 genesis-state Patch 物件，`base_revision` MUST 使用固定 sentinel 值 `rev:genesis-null`。

### 4.4 Patch Operations

v0.1 建議只定義少量基本操作：

- `insert_block`
- `insert_block_after`
- `delete_block`
- `replace_block`
- `move_block`
- `annotate_block`
- `set_metadata`

範例：刪除

```json
{
  "op": "delete_block",
  "block_id": "blk:009"
}
```

範例：註解

```json
{
  "op": "annotate_block",
  "block_id": "blk:001",
  "annotation": {
    "block_id": "blk:ann01",
    "block_type": "annotation",
    "content": "此段為社群常用的維護版本。"
  }
}
```

### 4.4.1 Trivial Change（規範）

在 Mycel v0.1 中，`trivial change` 指的是只改動編輯表面形式，而不改變文件結構、參照目標、metadata 語義、或預期語義的變更。

只有在以下條件全部成立時，某個 Patch 才 MAY 被分類為 trivial：

1. 每個 operation 都作用在同一文件狀態譜系中的既有 block
2. 每個 operation 只能是以下之一：
   - 對既有 block 的 `replace_block`
   - 不改變目標 block 自身內容的 `annotate_block`
3. 沒有任何 operation 改變 block 順序、block parentage、block identity、或 block type
4. 沒有任何 operation 插入、刪除、或移動 block
5. 沒有任何 operation 改變 metadata keys 或 metadata values
6. 沒有任何 operation 以可能改變解讀的方式修改 identifiers、revision references、URLs、numeric values、或 date/time literals
7. 結果文本只打算用於修正或正規化表面形式

典型的 trivial changes 包含：

- 明顯 typo 修正
- 空白正規化
- 標點正規化
- 在語義不變前提下的大小寫正規化
- 不改變註解主張的 annotation formatting 清理

以下不屬於 trivial changes：

- 任何結構變更
- 任何插入、刪除、或移動
- 任何對 `block_id` 的修改
- 任何對 metadata 語義的修改
- 任何可能合理改變解讀的 wording change

Trivial-change classification 只具 advisory 性質。
它 MUST NOT 繞過一般 Patch 驗證、Revision 驗證、簽章檢查、merge 規則、或 `state_hash` 重算。

### 4.5 Revision

Revision 表示某個狀態節點。
它不是全文本本身，而是「parent + patch 集合」形成的可驗證狀態。

```json
{
  "type": "revision",
  "version": "mycel/0.1",
  "revision_id": "rev:8fd2",
  "doc_id": "doc:origin-text",
  "parents": ["rev:0ab1"],
  "patches": ["patch:91ac"],
  "state_hash": "hash:state001",
  "author": "pk:authorA",
  "timestamp": 1777778890,
  "signature": "sig:..."
}
```

merge revision 範例：

```json
{
  "type": "revision",
  "version": "mycel/0.1",
  "revision_id": "rev:c7d4",
  "doc_id": "doc:origin-text",
  "parents": ["rev:8fd2", "rev:b351"],
  "patches": ["patch:a12f"],
  "state_hash": "hash:merged-state",
  "author": "pk:curator1",
  "timestamp": 1777780000,
  "merge_strategy": "semantic-block-merge",
  "signature": "sig:..."
}
```

`revision_id` 是導出的 canonical object ID，格式為 `rev:<object_hash>`。
它 MUST 由省略 `revision_id` 與 `signature` 後的 canonical Revision 內容計算而得。

### 4.5.1 Revision State Construction（規範）

為了讓 v0.1 的 `state_hash` 可重算，Revision 狀態建構規則如下：

1. `parents` 是有順序的陣列。
2. Genesis revision MUST 使用 `parents: []`。
3. 非 merge revision MUST 剛好有一個 parent。
4. 多 parent revision MUST 將 `parents[0]` 視為唯一的執行基底狀態。
5. `parents[1..]` 只記錄被合併的 ancestry；它們 MUST NOT 自動把內容帶入結果狀態。
6. 任何從次要 parents 採納的內容，都 MUST 明確實體化在列出的 `patches` 中。
7. `patches` 是有順序的陣列，且 MUST 依陣列順序逐一套用。
8. Revision 所引用的每個 Patch，其 `doc_id` MUST 與該 Revision 相同。
9. 非 genesis Revision 所引用的每個 Patch，其 `base_revision` MUST 等於 `parents[0]`。對 genesis revision，所有引用的 Patch 都 MUST 使用 `base_revision = rev:genesis-null`。
10. 若任何引用的 Patch 缺失、無效、或無法決定性套用，該 Revision 即為無效。

這表示接收端不會為了重算 Revision 狀態而重新執行 semantic merge 演算法。
接收端只會對執行基底狀態重放有序的 Patch 操作。

### 4.6 View

View 是一個已簽章的治理訊號，用來聲明某維護者在特定 policy body 下採信哪些 revisions。

```json
{
  "type": "view",
  "version": "mycel/0.1",
  "view_id": "view:9aa0",
  "maintainer": "pk:community-curator",
  "documents": {
    "doc:origin-text": "rev:c7d4",
    "doc:governance-rules": "rev:91de"
  },
  "policy": {
    "preferred_branches": ["community-mainline"],
    "accept_keys": ["pk:community-curator", "pk:reviewerB"],
    "merge_rule": "manual-reviewed"
  },
  "timestamp": 1777781000,
  "signature": "sig:..."
}
```

在 v0.1，View 不是終端使用者的偏好物件。
它是用來導出 profile-governed accepted head 的其中一個 selector 輸入。

`view_id` 是導出的 canonical object ID，格式為 `view:<object_hash>`。
它 MUST 由省略 `view_id` 與 `signature` 後的 canonical View 內容計算而得。

### 4.7 Snapshot

Snapshot 用於快速同步。

```json
{
  "type": "snapshot",
  "version": "mycel/0.1",
  "snapshot_id": "snap:44cc",
  "documents": {
    "doc:origin-text": "rev:c7d4"
  },
  "included_objects": [
    "rev:c7d4",
    "patch:91ac",
    "patch:a12f"
  ],
  "root_hash": "hash:snapshot-root",
  "created_by": "pk:mirrorA",
  "timestamp": 1777782000,
  "signature": "sig:..."
}
```

`snapshot_id` 是導出的 canonical object ID，格式為 `snap:<object_hash>`。
它 MUST 由省略 `snapshot_id` 與 `signature` 後的 canonical Snapshot 內容計算而得。

## 5. 序列化與雜湊

### 5.1 Canonical Serialization

在 hash 或簽章之前，所有協議物件都 MUST 先轉成 Appendix A 定義的 canonical JSON 形式。
同一套 canonicalization 規則也適用於 `state_hash` 計算所用的 state object，以及 `WIRE-PROTOCOL.zh-TW.md` 所引用的 wire envelope。

### 5.2 Hash

在 v0.1，同一個 network MUST 對 canonical object ID 與物件驗證使用同一個固定雜湊演算法。
預設建議為：

```text
hash = BLAKE3(canonical_bytes)
```

如果想保守，也可換成 SHA-256；但協議要固定，不可同網混用。

### 5.3 導出 ID 規則

對 v0.1 中任何內容定址物件型別：

1. 對物件內容做 canonicalize
2. 省略導出 ID 欄位（`patch_id`、`revision_id`、`view_id`、`snapshot_id`）
3. 省略 `signature`
4. 以網路固定的雜湊演算法計算剩餘 canonical bytes
5. 以 `<type-prefix>:<object_hash>` 重建導出 ID

接收端 MUST 拒絕任何內嵌導出 ID 與重算 canonical object ID 不一致的內容定址物件。

### 5.4 State Hash Computation（規範）

在 v0.1，Revision 的 `state_hash` 依以下方式計算：

1. 解析執行基底狀態：
   - 若 `parents` 為空，使用空狀態 `{ "doc_id": <revision.doc_id>, "blocks": [] }`
   - 否則，載入 `parents[0]` 已完整驗證的狀態
2. 依陣列順序，把引用的 `patches` 重放到該執行基底狀態上。
3. 產生結果文件狀態，表示為 canonical state object：

```json
{
  "doc_id": "doc:origin-text",
  "blocks": [
    {
      "block_id": "blk:001",
      "block_type": "paragraph",
      "content": "...",
      "attrs": {},
      "children": []
    }
  ]
}
```

4. 以協議其他部分相同的 serialization 規則對該 state object 做 canonicalize。
5. 計算 `state_hash = HASH(canonical_state_bytes)`。

補充規則：

- 頂層 block 順序 MUST 保留在結果 `blocks` 陣列中。
- 子 block 順序 MUST 保留在各自的 `children` 陣列中。
- 已刪除的 block MUST 不出現在結果狀態中。
- 若要保留 multi-variant 結果，MUST 由套用後的 Patch 結果狀態明確表達，而不能只從 parent ancestry 隱式推導。
- 接收端 MUST 拒絕任何宣告的 `state_hash` 與重算值不一致的 Revision。

## 6. 身分與簽章

### 6.1 作者身份

Mycel 作者身份預設是**假名公鑰身份**。

```text
author_id = pk:<public_key_fingerprint>
```

不是帳號，不是真名。

### 6.2 簽章算法

v0.1 建議：

- 簽章：Ed25519
- 金鑰交換：X25519

### 6.3 身分模式

Mycel 支援 3 種：

- **Persistent pseudonym**：長期筆名
- **Rotating pseudonym**：定期換 key
- **One-time signer**：一次性作者

### 6.4 Object Signature Matrix（規範）

v0.1 的物件簽章要求如下：

| 物件型別 | 簽章狀態 | 簽署者欄位 | 簽章 payload |
| --- | --- | --- | --- |
| `document` | forbidden | 無 | 無 |
| `block` | forbidden | 無 | 無 |
| `patch` | required | `author` | 省略 `signature` 後的 canonical Patch |
| `revision` | required | `author` | 省略 `signature` 後的 canonical Revision |
| `view` | required | `maintainer` | 省略 `signature` 後的 canonical View |
| `snapshot` | required | `created_by` | 省略 `signature` 後的 canonical Snapshot |

規則：

1. 接收端 MUST 拒絕任何缺少 `signature` 的 v0.1 `patch`、`revision`、`view`、`snapshot` 物件。
2. 接收端 MUST 拒絕任何帶有頂層 `signature` 欄位的 `document` 或 `block` 物件。
3. 簽署者欄位所指向的金鑰 MUST 能驗證對應 canonical payload 的簽章。
4. 對內容定址物件型別，內嵌的導出 ID MUST 先與重算出的 canonical object ID 一致，簽章驗證才可成立。
5. `signature` 欄位本身 MUST NOT 納入簽章 payload。

### 6.5 Object Signature Inputs（規範）

每一種需簽章的 v0.1 物件，其簽章 payload 都是「只省略 `signature` 欄位後」的 canonical serialization。

這表示：

- `patch` 的簽章覆蓋 `patch_id`、`doc_id`、`base_revision`、`author`、`timestamp`、`ops`
- `revision` 的簽章覆蓋 `revision_id`、`doc_id`、`parents`、`patches`、`state_hash`、`author`、`timestamp`，以及任何宣告的 merge 欄位
- `view` 的簽章覆蓋 `view_id`、`maintainer`、`documents`、`policy`、`timestamp`
- `snapshot` 的簽章覆蓋 `snapshot_id`、`documents`、`included_objects`、`root_hash`、`created_by`、`timestamp`

## 7. 節點模型

Mycel 節點分成 5 類角色（同一節點可兼任多種角色）：

1. **Author Node**：產生 patch / revision
2. **Mirror Node**：保存與提供內容
3. **Curator Node**：發布 View objects 並維護採信分支訊號
4. **Relay Node**：轉發 metadata 與 objects
5. **Archivist Node**：保存完整歷史

## 8. P2P 同步層

Mycel 不要求全節點同步全部資料，支援 partial replication。

### 8.1 節點宣告：Manifest

每個節點可公布 manifest：

```json
{
  "type": "manifest",
  "version": "mycel/0.1",
  "node_id": "node:alpha",
  "topics": ["text/core", "text/commentary"],
  "heads": {
    "doc:origin-text": ["rev:c7d4", "rev:b351"]
  },
  "snapshots": ["snap:44cc"],
  "capabilities": ["patch-sync", "snapshot-sync", "view-sync"]
}
```

### 8.2 同步流程

第一次加入：

1. 節點取得 bootstrap peers
2. 取得 manifest
3. 拉最近 snapshot
4. 補差額 patch / revision
5. 為一個或多個固定 profiles 建立 accepted-head 索引

日常更新：

1. 收到 head announcement
2. 檢查本地是否缺物件
3. 以 canonical object ID 拉取缺失
4. 驗 hash、驗簽章
5. 存入本地 store
6. 依固定 profile 規則重算 accepted heads

### 8.3 交換訊息類型

v0.1 最小訊息集：

- `HELLO`
- `MANIFEST`
- `HEADS`
- `WANT`
- `OBJECT`
- `SNAPSHOT_OFFER`
- `VIEW_ANNOUNCE`
- `BYE`

這些訊息的 transport 格式以 `WIRE-PROTOCOL.zh-TW.md` 為規範性定義。
本核心協議文件只描述概念性的同步流程與被複製物件的語義。

## 9. 衝突與合併

Mycel 不把衝突視為協議失敗。

### 9.1 合法狀態

以下都合法：

- 多個 heads
- 不同分支長期並存
- 同一段文本有多個地方版本

### 9.2 合併結果可分三類

- **Auto-merged**：自動合併成功
- **Multi-variant**：保留並列版本
- **Manual-curation-required**：需要人工整理

在 v0.1，任何以 Revision 發布的 merge 結果，都 MUST 已經被實體化成明確的 Patch 操作。
接收端是靠重放這些 Patches 驗證結果狀態，而不是根據 parent ancestry 重新計算 semantic merge（語義合併）。

### 9.3 Merge Generation Profile v0.1（規範）

Mycel v0.1 定義一個保守版 semantic merge generation profile（語義合併生成設定檔）。
這個 profile 只用來產生候選 merge Patch 操作。
驗證仍然只依賴最終產生的 Patch、Revision 與 `state_hash`。

#### 9.3.1 輸入

一個 merge generator 的輸入為：

- `base_revision`
- `left_revision`
- `right_revision`

三者都 MUST：

1. 屬於同一個 `doc_id`
2. 是已完整驗證的 revision
3. 在開始 merge generation 前先還原成 canonical document states

`base_revision` 是比對用的共同祖先狀態。
`left_revision` 與 `right_revision` 是兩個待整合的後代狀態。

#### 9.3.2 逐 Block 分類

對任何出現在三個狀態之一中的邏輯 `block_id`，都要將其分類為：

- unchanged
- inserted
- deleted
- replaced
- moved
- annotated
- metadata-changed

分類一律相對於 `base_revision` 進行。

#### 9.3.3 Auto-Merge 規則

只有當所有受影響 block 都能依以下規則解決時，merge generator MAY 產生 `Auto-merged`：

1. 若只有一側修改某 block，而另一側保持不變，則採用有修改的一側。
2. 若兩側對同一 block 做出 byte-identical 的修改，則採用該共同結果。
3. 若兩側在不同位置插入不同的新 block，則兩個 insert 都保留，且以決定性順序排列：
   1. 較小的 parent position index
   2. 當 parent position 相同時，left-side insert 先於 right-side insert
   3. 字典序較小的新增 `block_id`
4. 若一側對 block 做 annotation，而另一側修改其內容但未刪除該 block，則同時保留內容修改與 annotation。
5. 若兩側修改的是不同 metadata keys，則合併這些 key 更新。

若任一受影響 block 不屬於以上規則，generator MUST NOT 輸出 `Auto-merged`。

#### 9.3.4 強制非自動情況

遇到以下任一情況，merge generator MUST 輸出 `Multi-variant` 或 `Manual-curation-required`：

1. 兩側對同一 block 做不同內容的 replace
2. 一側刪除某 block，而另一側對其做 replace、move、或 annotate
3. 兩側把同一 block move 到不同目的地
4. 兩側對同一 metadata key 設定不同值
5. 任一側改變 block structure，而另一側對同一 subtree 做不相容修改

#### 9.3.5 Multi-Variant 輸出規則

若衝突僅限於同一邏輯 block 的替代性存活內容，generator SHOULD 優先輸出 `Multi-variant`。
最終 merge Patch MUST 在合併後狀態中明確實體化這些並存 alternatives。

#### 9.3.6 Manual Curation 規則

若衝突影響到 structure、ordering、deletion semantics、或 metadata，且無法安全表達成平行並存 variant，generator MUST 輸出 `Manual-curation-required`。

#### 9.3.7 輸出形式

產生的結果 MUST 被實體化成一般 Patch 操作。
Generator MUST NOT 依賴隱藏的 merge metadata 來讓結果狀態成立。

若 generator 輸出 `Auto-merged`，其 Patch 操作 MUST 足以讓任一接收端從 `parents[0]` 決定性重放出同樣結果。

### 9.4 多版本 block 範例

```json
{
  "type": "variant_block",
  "block_id": "blk:001",
  "variants": [
    {
      "from_revision": "rev:8fd2",
      "content": "起初沒有終稿，只有傳遞。"
    },
    {
      "from_revision": "rev:b351",
      "content": "起初沒有終稿，只有傳遞與再寫。"
    }
  ]
}
```

## 10. View 與採信

Mycel 不定義全域唯一 accepted head。
同一組文件可同時存在多個固定的 View profiles。
這個設計正是 Mycel 與 blockchain 的大差異。

### 10.0 Reader Client 合規要求（規範）

為了在保留 multi-view 的前提下，盡量降低 client 的自由裁量影響：

1. 合規的 reader client MUST 將每個顯示中的文件家族綁定到一個固定的 View profile。
2. 在 v0.1，accepted-head selection 的 profile 識別值為 `policy_hash`。
3. 合規的 reader client MUST 只依已驗證的 protocol objects 與該固定 profile 導出 active accepted head。
4. 合規的 reader client MUST NOT 提供會改變 active accepted head 的自由裁量本地 policy controls。
5. 合規的 reader client MAY 為了審計而顯示 raw heads、branch graphs、或其他 profile 的結果，但除非另有有效固定 profile 治理該結果，否則 MUST NOT 將其顯示為 active accepted head。

### 10.1 決定性 Head 選擇（規範）

為了降低 client 端分歧，head 選擇必須由協議規範驅動：

1. client MUST 先解析一個固定的 `profile_id`，並以 `profile_id` 與 `doc_id` 發出請求，且 MAY 附帶 selection-time boundary（選擇時間邊界）。
2. client MUST NOT 強制指定 `head_id`。
3. node MUST 依請求的固定 profile，從 eligible heads 即時計算 `selected_head`。
4. 對同一組已驗證物件集合、固定 profile 參數、以及有效 selection time（選擇時間），選擇器 MUST 產生決定性結果。
5. 回應 MUST 包含 `selected_head` 與可機器解析的 decision trace（決策軌跡）。

#### 10.1.1 Selector Inputs

Selector 的輸入 tuple（輸入組）為：

- `profile_id`
- `doc_id`
- `effective_selection_time`

在 v0.1，`profile_id` 就是 active View profile 的固定 `policy_hash`。
若 client 支援多個固定 profiles，MUST 以明確列舉方式提供；它 MUST NOT 為 active accepted-head 路徑臨時構造 ad hoc local policies。

`effective_selection_time` 定義如下：

- 若 client 有提供 boundary，則使用該值
- 否則使用 node 處理請求時的本地時間

若 client 省略 boundary，node MUST 在 decision trace（決策軌跡）中輸出解析後的 `effective_selection_time`。

Selector 只可使用 `policy_hash` 等於 `profile_id` 的完整驗證 View 物件。

#### 10.1.2 Eligible Heads

對某個 `doc_id` 而言，Revision 若要成為 eligible head，必須同時符合以下條件：

1. 該 Revision 已依所有 object、hash、signature、state 規則完整驗證
2. 該 Revision 的 `doc_id` 與請求的 `doc_id` 相同
3. 該 Revision 的 timestamp 小於或等於 `effective_selection_time`
4. 不存在另一個同文件、同樣已完整驗證且 timestamp 小於或等於 `effective_selection_time` 的 descendant Revision

若不存在 eligible heads，選擇必須失敗，並回傳像 `NO_ELIGIBLE_HEAD` 這類可機器解析的原因。

#### 10.1.3 Maintainer Signals

對每個已準入的 maintainer key `k`，selector 在 selector epoch（選擇器 epoch）中最多導出一個 signal（訊號）：

1. 依第 10.2 節規則決定 selector epoch
2. 收集所有完整驗證過的 View 物件，且需符合：
   - `maintainer == k`
   - `timestamp` 落在 selector epoch 內
   - `timestamp <= effective_selection_time`
   - `HASH(canonical_serialization(view.policy)) == profile_id`
3. 依以下順序選出其中最新的一個 View：
   1. 較新的 `timestamp`
   2. 字典序較小的 `view_id`
4. 若該 View 含有 `documents[doc_id]`，且其值正好是某個 eligible head，則 `k` 對該 head 貢獻一個 support signal
5. 否則 `k` 對該 `doc_id` 不貢獻 signal

對任一 `(profile_id, doc_id, selector_epoch)`，每個 admitted maintainer 最多只能對一個 eligible head 貢獻 signal。

#### 10.1.4 Selector Score

對每個 eligible head `h`：

```text
weighted_support(h) = sum(effective_weight(k)) for all maintainers k signaling to h
supporter_count(h) = count(k) for all maintainers k signaling to h
selector_score(h) = weighted_support(h)
```

被選中的 head，是 ordered tuple 最大的 eligible head：

```text
(selector_score, revision_timestamp, inverse_lexicographic_priority)
```

Tie-break 順序 MUST 固定為：

1. 較高 `selector_score`
2. 較新 `revision_timestamp`
3. 字典序較小的 `revision_id`

Raw supporter count MAY 出現在 trace 中以利審計，但 MUST NOT 高於 `selector_score`。

#### 10.1.5 Decision Trace Schema

Decision trace（決策軌跡）MUST 可機器解析，且至少包含：

```json
{
  "profile_id": "hash:...",
  "doc_id": "doc:origin-text",
  "effective_selection_time": 1777781000,
  "selector_epoch": 587,
  "eligible_heads": [
    {
      "revision_id": "rev:0ab1",
      "revision_timestamp": 1777780000,
      "weighted_support": 7,
      "supporter_count": 3,
      "selector_score": 7
    }
  ],
  "selected_head": "rev:0ab1",
  "tie_break_reason": "higher_selector_score"
}
```

對同一組已驗證物件集合、固定 profile 參數、以及 effective selection time（選擇時間），此 trace MUST 可重現。

### 10.2 View Profile 參數 + 維護者權重準入（規範）

Mycel 採用假名、身份盲的維護者治理。
維護者以 key 識別，不要求真實身份，也不要求彼此相識。

準入與加權規則：

1. 維護者候選資格 MUST 只依可驗證的協議行為評估，不依聲稱的真實身份。
2. 提供 accepted-head results 的 node MUST 保存並公布其固定 profile 參數，以便審計。
3. 固定 profile 參數至少 MUST 包含：
   - `epoch_seconds`
   - `epoch_zero_timestamp`
   - `admission_window_epochs`
   - `min_valid_views_for_admission`
   - `min_valid_views_per_epoch`
   - `weight_cap_per_key`
4. `epoch_seconds` MUST 是正整數。
5. Selector epoch 為：

```text
selector_epoch = floor((effective_selection_time - epoch_zero_timestamp) / epoch_seconds)
```

6. 對每個 maintainer key `k` 與 epoch `e`，定義：
   - `valid_view_count(e, k)`：在 epoch `e` 中，由 `k` 發布且 policy hash 等於 selector `profile_id` 的完整驗證 View 物件數量
   - `critical_violation_count(e, k)`：在 epoch `e` 中可驗證歸因於 `k` 的重大違規數量
7. 若某 key 在前 `admission_window_epochs` 個已完成 epoch 中同時滿足：
   - `valid_view_count` 總和至少為 `min_valid_views_for_admission`
   - `critical_violation_count` 總和為零
   則該 key 在 epoch `e` 中視為 admitted。
8. 非 admitted key 的 effective weight MUST 為 `0`。
9. Admitted key 第一次取得的 weight 為 `1`。
10. 之後每個 epoch 的 effective weight 更新規則如下：

```text
delta(e, k) =
  -1 if critical_violation_count(e-1, k) > 0
  +1 if critical_violation_count(e-1, k) == 0
       and valid_view_count(e-1, k) >= min_valid_views_per_epoch
   0 otherwise

effective_weight(e, k) =
  clamp(effective_weight(e-1, k) + delta(e, k), 0, weight_cap_per_key)
```

11. `clamp(x, lo, hi)` 的意義是：若 `x < lo` 回傳 `lo`，若 `x > hi` 回傳 `hi`，否則回傳 `x`。
12. 若某 key 在 epoch `e-1` 有一個或多個重大違規，則它在 epoch `e` MUST 至少失去一個 weight unit。
13. 合規的 reader client MUST NOT 套用會改變 active accepted-head 路徑的自由裁量 per-installation quarantine 或 removal 規則。
14. head 選擇 MUST 使用 `effective_weight(e, k)`，且 MUST NOT 單獨依賴原始 hit count。

## 11. 匿名與安全預設

### 11.1 傳輸匿名

Mycel 建議預設跑在匿名傳輸上，例如：

- Tor onion services
- 或其他匿名 mesh transport

### 11.2 內容安全

每個 object 都需通過：

- hash 驗證
- signature 驗證
- context 驗證

### 11.3 Metadata 最小化

建議節點：

- 批次轉發
- 隨機延遲
- 不公開真實作者身份
- topic 名稱可 capability 化

### 11.4 本地傳輸與安全策略

每個節點仍可定義：

- 接受哪些作者 key
- 接受哪些 curator key
- 是否接受匿名 key
- 新 key 是否先 quarantine

這些本地策略 MAY 影響 storage、relay、moderation、或 private inspection。
但對合規的 reader client 而言，它們 MUST NOT 改變第 10 節所定義的 fixed-profile active accepted head。

## 12. 本地儲存模型

本地儲存分成：

### 12.1 Object Store

用 `object_id` 存所有物件。

### 12.2 Index Store

建立索引：

- `doc_id -> revisions`
- `revision -> parents`
- `block_id -> latest states`
- `author -> patches`
- `view_id -> governance signal contents`
- `profile_id -> current accepted-head map`

### 12.3 Policy Store

將本地傳輸、安全、與 moderation 規則，與 fixed-profile accepted-head 路徑分開保存。

## 13. URI / 命名格式

v0.1 可用這種命名：

- `mycel://doc/origin-text`
- `mycel://rev/c7d4`
- `mycel://patch/91ac`
- `mycel://view/9aa0`
- `mycel://snap/44cc`

## 14. CLI 雛形

未來工具可包含：

```bash
mycel init
mycel create-doc origin-text
mycel patch origin-text
mycel commit origin-text
mycel branch create community-mainline
mycel merge rev:8fd2 rev:b351
mycel view create community-curation-v3
mycel sync
mycel serve
mycel verify
```

## 15. 最小實作架構

一個 Mycel client 最少要有：

### 15.1 Core

- object serializer
- hash engine
- signature engine
- patch applier
- revision builder

### 15.2 Store

- object store
- index store
- local transport/safety policy store
- accepted-head profile index

### 15.3 Network

- peer transport
- manifest exchange
- want/object exchange
- snapshot sync

### 15.4 UI

- CLI
- wiki-like reader/editor
- diff viewer
- branch/view browser

## 16. 典型流程示例

### 16.1 建立文件

1. 作者 A 建立 origin-text
2. 建立 genesis blocks
3. 建立 genesis revision
4. 簽章
5. 發布給 peers

### 16.2 修改文件

作者 B 想改一段：

1. 下載最新 revision
2. 建立 patch
3. 用自己的 key 簽章
4. 產生新 revision
5. 發佈到網路

### 16.3 分支

作者 C 不同意主線：

1. 以同一 `base_revision` 建 patch
2. 發表不同 revision
3. 網路形成第二個 head

### 16.4 合併

Curator D 想把兩邊整合：

1. 取得兩個 heads
2. 試 semantic block merge
3. 成功則產生 merge revision
4. 用自己 key 發表新的 view

## 17. 協議精神

Mycel 的核心不是唯一真理，而是：

> 文可改，史可驗，支可分，網可散。

英文可寫成：

> Write locally. Sign changes. Replicate freely. Merge socially.

## 18. 協議特色總結

若用一句話定義 Mycel 與其他系統的差異：

- 不是 Git：因為它天生 P2P、天生多 view、天生匿名可用
- 不是 blockchain：因為它不追求全域唯一共識
- 不是 torrent：因為它不是只傳檔案包，而是傳可驗證變更歷史
- 不是普通 wiki：因為版本不是附屬功能，而是核心結構

## 19. 建議下一版

目前這版已包含：

1. **Wire protocol**：規範性的同步訊息 schema
2. **Canonical serialization appendix**：決定性的雜湊與簽章規則
3. **Conservative merge generation profile**：可安全重放的合併輸出規則

下一步最有價值的是：

1. **Implementation checklist（實作檢查清單）**：把規格整理成可落地的實作 profile（設定檔）
2. **Consistency audit（一致性稽核）**：把所有文件中的例子、術語、範圍對齊
3. **Governance simplification review（治理簡化檢查）**：在視 v0.1 為穩定前，先收斂 selector / governance 的可選複雜度

## Appendix A. Canonical Serialization（規範）

Mycel v0.1 以下情境都使用 canonical JSON bytes：

- 內容定址 object ID
- object signatures
- `state_hash` 計算
- wire-envelope signatures

### A.1 編碼

1. Canonical bytes MUST 是 UTF-8 編碼的 JSON text。
2. JSON text MUST NOT 包含 byte order mark。
3. 字串值以外的 insignificant whitespace 一律禁止。

### A.2 資料型別

v0.1 canonical payload 可使用的 JSON 值型別：

- object
- array
- string
- integer number
- `true`
- `false`

以下在 canonical payload 中視為無效：

- `null`
- 浮點數
- 指數記號
- 重複的 object keys

### A.3 Object 規則

1. Object keys MUST 唯一。
2. Object keys MUST 依原始 Unicode code point 的字典序遞增序列化。
3. Object members MUST 以 `"key":value` 形式序列化，不得加入額外空白。

Key 排序範例：

```json
{"author":"pk:a","doc_id":"doc:x","type":"patch","version":"mycel/0.1"}
```

### A.4 Array 規則

1. Arrays MUST 保留協議定義的順序。
2. Arrays MUST 以逗號分隔序列化，不得加入額外空白。
3. Canonicalization 過程 MUST NOT 對 arrays 重新排序。

這表示：

- `parents` 保留宣告順序
- `patches` 保留宣告順序
- `blocks` 保留文件結構順序
- wire `WANT` 的 `objects` 保留發送端請求順序

### A.5 String 規則

1. Strings MUST 使用 JSON 雙引號字串語法序列化。
2. Strings MUST 精確保留原始 code points；實作 MUST NOT 自動做 Unicode normalization。
3. 雙引號（`"`）與反斜線（`\`）MUST 轉義。
4. U+0000 到 U+001F 的控制字元 MUST 使用小寫 `\u00xx` 轉義。
5. `/` MUST NOT 被轉義，除非更高層 transport 在 canonicalization 之外另有要求。
6. 非 ASCII 字元 MAY 直接以 UTF-8 出現，且除非它是控制字元，MUST NOT 被改寫成 `\u` escape。

### A.6 Integer 規則

1. v0.1 canonical payload 中的數字 MUST 是十進位整數。
2. 零 MUST 序列化為 `0`。
3. 正整數 MUST NOT 帶有前置 `+`。
4. 整數 MUST NOT 含有前導零。
5. 負整數只有在欄位定義明確允許時才可使用。

### A.7 Boolean

Boolean 值 MUST 序列化成小寫 `true` 或 `false`。

### A.8 欄位省略

1. 不存在的 optional fields MUST 直接省略。
2. 實作 MUST NOT 用 `null` 表示「缺省」。
3. 導出 ID 欄位與 `signature` 只有在特定 hashing 或 signing 規則明確要求時，才可省略。

### A.9 Canonicalization Procedure

Canonicalize 一個 payload 的步驟：

1. 驗證 payload 只使用允許的 JSON 型別。
2. 拒絕重複 keys。
3. 拒絕禁止的數字格式與 `null`。
4. 依 A.3 規則遞迴排序所有 object keys。
5. 保留所有 array 順序。
6. 以 UTF-8 JSON 並且不含 insignificant whitespace 的形式序列化。

### A.10 Canonical State Object

計算 `state_hash` 時，結果 state object MUST 使用以下形狀：

```json
{
  "doc_id": "doc:origin-text",
  "blocks": [
    {
      "block_id": "blk:001",
      "block_type": "paragraph",
      "content": "Example text",
      "attrs": {},
      "children": []
    }
  ]
}
```

補充規則：

1. State serialization 中的每個 block object 都 MUST 包含 `block_id`、`block_type`、`content`、`attrs`、`children`。
2. `attrs` MUST 是 object；若為空，MUST 序列化成 `{}`。
3. `children` MUST 是 array；若為空，MUST 序列化成 `[]`。

### A.11 Canonical Envelope Serialization

Wire envelopes 使用同一套 canonical JSON 規則。
計算 envelope signature 時，必須在 canonicalization 之前先省略 `sig` 欄位。
