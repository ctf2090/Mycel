# Mycel Protocol v0.1

## 0. 定位

Mycel 是一種具備以下特性的文本協議：

- Git 式版本模型
- P2P 複製
- 簽章驗證
- 多分支共存
- 不要求全域單一共識

它不是區塊鏈，也不是 Git 複製品；它是為文字／知識／經典而設計的，去中心、可分叉、可驗證歷史的協議。

適用場景包含：

- 經典
- 註解
- manifesto
- 社群章程
- 規範文件
- 去中心 wiki
- 難以被刪除的知識網

## 1. 設計目標

Mycel 的設計目標：

1. **可驗證歷史**：每次修改都可追溯。
2. **去中心存活**：沒有單一伺服器也能保存與同步。
3. **分支合法**：分叉不是錯誤，是合法狀態。
4. **合併可選**：社群可自行形成 canonical view。
5. **匿名可用**：作者可用假名金鑰，而非真實身分。
6. **文本優先**：以 block / paragraph 為主要操作單位。

## 2. 協議概念

Mycel 把資料拆成 6 種核心概念：

- **Document**：文件
- **Block**：段落/區塊
- **Patch**：一次修改
- **Revision**：某個可驗證狀態
- **View**：某社群採信的版本集合
- **Snapshot**：某一時刻的快照包

## 3. 基本原則

### 3.1 內容定址

所有 object 都由內容 hash 決定 ID：

```text
object_id = hash(canonical_serialization(object))
```

### 3.2 簽章必須存在

所有作者產生的 Patch、Revision、View 都必須有數位簽章。

### 3.3 多個 head 合法

同一個文件可以有多個 heads。

### 3.4 Canonical 不是全域真理

所謂「正典版」只是某個 View，不是全網唯一版本。

### 3.5 傳輸與接受分離

節點可以接收某 object，不代表一定接受它進入本地 canonical view。

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
  "genesis_revision": "rev:abc123"
}
```

欄位：

- `doc_id`：文件固定 ID
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

### 4.3 Patch

Patch 表示一次對文件的修改。

```json
{
  "type": "patch",
  "version": "mycel/0.1",
  "patch_id": "patch:91ac",
  "doc_id": "doc:origin-text",
  "base_revision": "rev:old001",
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
    "content": "此段為東港支系常用版本。"
  }
}
```

### 4.5 Revision

Revision 表示某個狀態節點。
它不是全文本本身，而是「parent + patch 集合」形成的可驗證狀態。

```json
{
  "type": "revision",
  "version": "mycel/0.1",
  "revision_id": "rev:new001",
  "doc_id": "doc:origin-text",
  "parents": ["rev:old001"],
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
  "revision_id": "rev:merge001",
  "doc_id": "doc:origin-text",
  "parents": ["rev:branchA", "rev:branchB"],
  "patches": ["patch:mergeA"],
  "state_hash": "hash:merged-state",
  "author": "pk:curator1",
  "timestamp": 1777780000,
  "merge_strategy": "semantic-block-merge",
  "signature": "sig:..."
}
```

### 4.6 View

View 是「某社群／某節點目前採信哪些版本」。

```json
{
  "type": "view",
  "version": "mycel/0.1",
  "view_id": "view:east-school-v3",
  "maintainer": "pk:east-curator",
  "documents": {
    "doc:origin-text": "rev:merge001",
    "doc:ritual-law": "rev:law220"
  },
  "policy": {
    "preferred_branches": ["east-lineage"],
    "accept_keys": ["pk:east-curator", "pk:elderB"],
    "merge_rule": "manual-reviewed"
  },
  "timestamp": 1777781000,
  "signature": "sig:..."
}
```

View 很重要，因為 Mycel 沒有單一全域正統。

### 4.7 Snapshot

Snapshot 用於快速同步。

```json
{
  "type": "snapshot",
  "version": "mycel/0.1",
  "snapshot_id": "snap:weekly-2026-03-08",
  "documents": {
    "doc:origin-text": "rev:merge001"
  },
  "included_objects": [
    "rev:merge001",
    "patch:91ac",
    "patch:mergeA"
  ],
  "root_hash": "hash:snapshot-root",
  "created_by": "pk:mirrorA",
  "timestamp": 1777782000,
  "signature": "sig:..."
}
```

## 5. 序列化與雜湊

### 5.1 Canonical Serialization

所有 object 在 hash 前，必須先轉成固定 canonical form：

- key 順序固定
- UTF-8
- 不保留不必要空白
- 陣列順序固定
- 數字格式固定

### 5.2 Hash

v0.1 可先定為：

```text
hash = BLAKE3(canonical_bytes)
```

如果想保守，也可換成 SHA-256；但協議要固定，不可同網混用。

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

## 7. 節點模型

Mycel 節點分成 5 類角色（同一節點可兼任多種角色）：

1. **Author Node**：產生 patch / revision
2. **Mirror Node**：保存與提供內容
3. **Curator Node**：維護 view / canonical branch
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
    "doc:origin-text": ["rev:merge001", "rev:branchB"]
  },
  "snapshots": ["snap:weekly-2026-03-08"],
  "capabilities": ["patch-sync", "snapshot-sync", "view-sync"]
}
```

### 8.2 同步流程

第一次加入：

1. 節點取得 bootstrap peers
2. 取得 manifest
3. 拉最近 snapshot
4. 補差額 patch / revision
5. 建立本地 view

日常更新：

1. 收到 head announcement
2. 檢查本地是否缺物件
3. 用 object hash 拉取缺失
4. 驗 hash、驗簽章
5. 存入本地 store
6. 依 policy 決定是否納入 view

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

`WANT`：

```json
{
  "type": "want",
  "objects": ["rev:merge001", "patch:91ac"]
}
```

`OBJECT`：

```json
{
  "type": "object",
  "object_id": "patch:91ac",
  "payload": { "...": "..." }
}
```

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

### 9.3 多版本 block 範例

```json
{
  "type": "variant_block",
  "block_id": "blk:001",
  "variants": [
    {
      "from_revision": "rev:branchA",
      "content": "起初沒有終稿，只有傳遞。"
    },
    {
      "from_revision": "rev:branchB",
      "content": "起初沒有終稿，只有傳遞與再寫。"
    }
  ]
}
```

## 10. View 與 Canon

Mycel 不定義全域唯一 canon，只存在：

- local view
- school view
- public view
- archival view

例子：

- 某教派維護自己的正典
- 某學者維護一個批判校勘版
- 某節點只接受自己信任作者的 patch

這個設計正是 Mycel 與 blockchain 的大差異。

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

### 11.4 信任策略

每個節點可定義：

- 接受哪些作者 key
- 接受哪些 curator key
- 是否接受匿名 key
- 新 key 是否先 quarantine

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
- `view_id -> current head map`

### 12.3 Policy Store

保存本地信任與接受規則。

## 13. URI / 命名格式

v0.1 可用這種命名：

- `mycel://doc/origin-text`
- `mycel://rev/merge001`
- `mycel://patch/91ac`
- `mycel://view/east-school-v3`
- `mycel://snap/weekly-2026-03-08`

## 14. CLI 雛形

未來工具可包含：

```bash
mycel init
mycel create-doc origin-text
mycel patch origin-text
mycel commit origin-text
mycel branch create east-lineage
mycel merge rev:branchA rev:branchB
mycel view create east-school-v3
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
- local policy store

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

目前這版是概念規格。下一步最值得補的是三塊：

1. **Wire protocol**：`HELLO`、`WANT`、`OBJECT` 的具體欄位
2. **Canonical serialization spec**：避免不同實作 hash 不一致
3. **Merge semantics**：block-based auto-merge 規則

下一步可直接延伸成：**Mycel wire protocol v0.1**，定義節點封包格式與同步流程細節。
