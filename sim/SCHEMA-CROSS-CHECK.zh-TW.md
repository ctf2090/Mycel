# Schema Cross-check

本文件說明 simulator 各份 schema 之間如何互相關聯，以及哪些欄位應在不同檔案之間做一致性檢查。

目標是讓下列內容彼此對齊：

- fixture data
- peer definitions
- topology definitions
- test cases
- run reports

同時又不把實作綁死在單一語言上。

## 概覽

simulator 的資料流如下：

1. fixture 定義測試資料情境
2. peer definition 定義單一 peer 的形狀
3. topology 把多個 peers 組成一個有界網路
4. test case 選定 topology 與 fixture，並宣告預期 outcomes
5. report 記錄實際發生的結果

## Schema 角色

| Schema | 主要用途 | 典型檔案位置 |
| --- | --- | --- |
| `fixture.schema.json` | 定義一組 fixture set | `fixtures/object-sets/*/fixture.json` |
| `peer.schema.json` | 定義一份 peer config | `sim/peers/*.json` |
| `topology.schema.json` | 定義一份 peer graph | `sim/topologies/*.json` |
| `test-case.schema.json` | 定義一個可執行的 test case | `sim/tests/*.json` |
| `report.schema.json` | 定義一次 run result | `sim/reports/*.json` 或 `sim/reports/out/*.json` |

## Cross-check 規則

### 1. Fixture -> Topology

以下值應該彼此一致：

- `fixture.seed_peer` 應對上 topology 中預定載入 seed data 的 peer 角色或 node mapping。
- `fixture.reader_peers[]` 應對上映射到所選 topology 中的 reader peers。
- `fixture.fault_peer` 若存在，應對上 topology 中具 fault 能力的 peer。
- `fixture.expected_outcomes[]` 應與 topology 預期的情境相容。

最低 loader 檢查：

- 所選 topology 至少包含足夠的 peers，能滿足 fixture 的角色參照

## 2. Peer -> Topology

`topology.schema.json` 會重用 `peer.schema.json`。

這表示 `topology.peers[]` 中的每個 entry，都應已滿足 standalone peer contract。

Cross-check 重點：

- 同一個 topology 裡的 `node_id` 必須唯一
- `bootstrap_peers[]` 除非測試明確允許外部 peers，否則只能引用同一 topology 內存在的 peer IDs
- peer 的 `role` 值應與所選 fixture 和 test case 一致

最低 loader 檢查：

- 所有 bootstrap references 都能成功 resolve

## 3. Topology -> Test Case

以下值應該彼此一致：

- `test_case.topology` 應指向一個 topology file
- 若 topology 宣告了 `execution_mode`，則 `test_case.execution_mode` 應與之相同或相容
- `test_case.expected_outcomes[]` 應是 topology 與 fixture 情境的子集，或至少相容

最低 loader 檢查：

- 被引用的 topology file 必須存在且可解析

## 4. Fixture -> Test Case

以下值應該彼此一致：

- `test_case.fixture_set` 應指向一個 fixture directory
- 所選 fixture 應能支援 `test_case.category`
- `test_case.expected_result` 應與 fixture 的意圖一致

例子：

- `minimal-valid` 通常應對應 `expected_result: "pass"`
- `hash-mismatch` 若測試目標是「正確拒絕」，整體仍可能是 pass
- malformed 的負向測試不應自動等於 `expected_result: "fail"`，除非 harness 對 failure 有那樣的定義

最低 loader 檢查：

- 被引用的 fixture directory 必須存在，且包含合法的 `fixture.json`

## 5. Test Case -> Report

以下值應該彼此一致：

- `report.test_id` 應等於 `test_case.test_id`
- `report.topology_id` 應等於載入後 topology 的 `topology_id`
- `report.fixture_id` 應等於載入後 fixture 的 `fixture_id`
- `report.execution_mode` 應等於這次 run 實際使用的 mode

最低 runner 檢查：

- report 的 identity fields 應來自已 resolve 的輸入，不應手動重打

## 6. Report -> Expected Outcomes

執行後應比較以下值：

- `report.result` 對 `test_case.expected_result`
- `report.summary.matched_expected_outcomes[]` 對 `test_case.expected_outcomes[]`
- `report.failures[]` 對 test case 定義的 assertions

最低 validator 檢查：

- 每個必要的 test-case assertion，要嘛已滿足，要嘛在 report failure entry 中被表示出來

## Identity Map

以下 identifiers 應在 simulator 資料模型中保持穩定：

| Identifier | Source of truth | 被哪些地方重用 |
| --- | --- | --- |
| `fixture_id` | fixture | test case、report |
| `node_id` | peer / topology peer entry | fixture 角色參照、report peer entries |
| `topology_id` | topology | report |
| `test_id` | test case | report |
| `run_id` | report runtime | 僅 report |

## 最小 Loader 順序

建議的解析順序如下：

1. 載入 test case
2. 載入其引用的 topology
3. 載入其引用的 fixture
4. 依 `peer.schema.json` 驗證 topology peer entries
5. 執行 fixture、topology、test case 之間的 cross-check
6. 執行 run
7. 產生 report
8. 依 `report.schema.json` 驗證 report

這個順序可以讓 reference resolution 保持明確，避免隱藏預設值。

## 最低 Validation Checklist

第一版實作至少應拒絕：

- 缺少被引用的 topology file
- 缺少被引用的 fixture file
- 同一 topology 中重複的 `node_id`
- 無法 resolve 的 `bootstrap_peers` 參照
- 無法映射到 topology peers 的 fixture 角色參照
- 與已解析測試輸入不一致的 report identity fields

## 非目標

這份 cross-check 文件不要求：

- 為整個 repo 建立所有 peer IDs 的全域 registry
- 為所有未來 profiles 定義一套通用角色分類
- 對每個 expected outcome string 做全自動語意驗證

它只定義 simulator v0 所需的最小一致性檢查。
