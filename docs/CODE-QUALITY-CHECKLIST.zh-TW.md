# Mycel 程式碼品質檢查清單

狀態：目前有效的工作檢查清單

這份文件是用來反覆審查 Mycel workspace 實作品質的檢查清單。

當我們在做以下事情時，請使用這份清單：

- 審查 pull request 或已落地的 diff
- 規劃重構
- 判斷某個檔案是否該拆分
- 判斷是否該抽出重複 literals 或 helpers
- 檢查測試是否和產品邏輯保持足夠獨立

它的目的是讓程式碼庫保持可審查、可組合，也更容易在不引入隱性回歸的情況下修改。

## 1. 快速關卡

在投入太多時間打磨變更前，先問：

1. 這個變更的範圍是否夠小，小到我們能有信心審查？
2. 這個檔案或函式是否正在一次解決超過一個問題？
3. 這個變更是否重複實作了其他地方已存在的邏輯？
4. 這個變更是否引入了之後難以維護的 literals 或結構？
5. 下一個接手的人是否能快速看懂應該去哪裡改這個行為？

如果答案不明確，先把變更範圍縮小。

## 2. 核心審查概念

每次都要檢查下面這些概念。

### 2.1 範圍與檔案大小

- 這個檔案是否仍然可以在一次閱讀中看懂？
- 這個檔案是否混合了彼此無關的責任？
- 若依 concern 拆分，是否能讓未來審查更容易？
- 某個測試檔是否正在悄悄變成第二個實作表面？

預設偏向：

- 優先使用依目的成形的小模組，而不是包山包海的大檔案

建議的警訊：

- 檔案長度成長到大約 `300-500` 行以上
- 只能靠註解分段，而沒有真正的模組邊界
- 同一檔案同時包含 CLI 解析、領域邏輯與輸出格式化

可用工具 / 模組：

- `wc -l`、`rg --files`、編輯器 outline / symbol view：快速看檔案大小與可掃讀性
- `ast-grep`：找出暗示應按 concern 拆分的重複結構區段
- `cloc` 或其他 repo 統計工具：快速掃出大型檔案熱點

### 2.2 函式大小與意圖

- 每個函式是否只做一件事？
- 函式很長，是因為領域細節真的必要，還是因為缺少 helper？
- 能否把重複 setup 或分支命名後抽出？
- 函式名稱是否表達意圖，而不只是機械動作？

預設偏向：

- 優先使用短函式與明確名稱，而不是長篇 procedural block

建議的警訊：

- 函式長度成長到大約 `40-60` 行以上，卻沒有很強的理由
- 巢狀分支太深，主路徑被淹沒
- setup、執行與 rendering 混在同一個函式裡

可用工具 / 模組：

- `rg` 搭配編輯器 symbol outline：快速找長函式與可疑熱點
- `ast-grep`：找出值得抽 helper 的重複 setup、分支或 method chain 形狀
- `clippy`：補充偵測複雜控制流與可疑寫法

### 2.3 硬編碼值與重複 literals

- 這個 literal 是穩定的領域真相、測試 fixture 資料，還是可避免的 magic value？
- 如果同一段字串、數字或 JSON 片段出現好幾次，它是否該變成 constant 或 helper？
- 這個 literal 是否其實承載了 policy，應該放在 profile、設定表面或共用 builder？
- 之後若要修改這個值，是否會需要碰很多地方？

預設偏向：

- 當 fixture literal 有助於可讀性時，保留在本地
- 當 literal 變成重複維護成本或 policy 耦合時，再抽出

建議的警訊：

- 多個測試重複使用同樣的 ID、prefix、timestamp 或 magic number
- 協定版本字串或 object-type 字串在多處手寫複製
- policy 預設值被手動複製到很多地方

可用工具 / 模組：

- `rg`：搜尋重複字串、prefix、ID、timestamp、object-type literals
- `ast-grep` 或 `comby`：找變數名不同但物件 / JSON 拼裝形狀相似的重複樣板
- 共用 constants、builders、profile / config 模組：當 literal 承載 policy 時優先放這裡

### 2.4 共用邏輯與本地重實作

- 這段程式是否重做了 canonicalization、hashing、signing、parsing、replay 或 selection 邏輯，而這些邏輯其實已存在共用程式中？
- 這個測試是在獨立驗證行為，還是在默默重建同一套實作規則？
- 是否能用共用 helper 表達相同 setup，並降低 drift 風險？
- 這個「小 helper」是否其實是產品邏輯的第二份副本？

預設偏向：

- 優先共用 production 或 test-support helper，而不是本地重實作

建議的警訊：

- 測試裡出現本地 canonical JSON 或 signature 程式
- copy-paste 的 derived ID 或 hash 計算
- 多個模組都手動拼同樣的物件結構

可用工具 / 模組：

- `ast-grep`：找本地重做 canonicalization、hashing、replay、selector 等邏輯的結構樣式
- `rg`：搜尋本應走共用 helper、卻在本地直接實作的呼叫或 literals
- `canonical`、`signature`、`replay`、`verify` 與 test-support helpers：優先回頭重用這些模組

### 2.5 分層邊界

- CLI 程式是否保持輕薄，而 core 邏輯是否保持可重用？
- formatting 邏輯是否滲入 protocol 或 storage 程式？
- 測試是否使用了正確層級來做 setup？
- profile 或 app-layer 語意是否在沒有必要時滲入 shared core？

預設偏向：

- 讓邊界保持明確
- 讓 core 可重用，CLI 保持輕薄

可用工具 / 模組：

- `rg --files` 與編輯器導覽：先看模組布局與責任落點
- `ast-grep`：找 CLI 直接做 core 領域決策、而不是委派給共用模組的區塊
- 既有 crate / module 邊界，例如 `mycel-core`、CLI entrypoints、store / protocol 模組：當作分層地圖

### 2.6 錯誤表面與可除錯性

- 錯誤訊息是否有說清楚哪裡失敗、怎麼失敗？
- CLI 可見的失敗是否能幫助使用者恢復？
- assertion 與 `expect(...)` 的訊息是否足夠具體，能快速定位？
- 使用者實際看得到的失敗路徑是否有覆蓋到？

預設偏向：

- 優先提供清楚失敗，而不是模糊的成功 / 失敗狀態

可用工具 / 模組：

- `clippy`：找可疑錯誤處理與過於脆弱的 `unwrap` / `expect`
- `rg 'expect\\(|unwrap\\(|map_err\\('`：快速巡檢錯誤表面
- CLI-visible smoke tests 與聚焦 unit tests：驗證使用者實際看得到的失敗路徑

### 2.7 測試品質

- 測試是否描述行為，而不是實作細節 trivia？
- 測試是否讓 fixtures 維持可讀？
- 測試是否過度綁定在非契約重要的格式細節？
- 測試是否複製了太多 production logic，而帶來虛假的信心？

預設偏向：

- 優先使用聚焦行為、搭配小型具名 builders 的測試

建議的警訊：

- 大段 inline JSON 在多個測試中反覆出現
- helper 函式重建產品演算法
- assertion 綁定偶然輸出，而非穩定行為

可用工具 / 模組：

- `rg`：找重複 fixture blobs、重複 assertions 與跨檔測試 helpers
- `ast-grep`：找測試是否在結構上過度鏡像 production 演算法
- 共用 test-support helpers / builders：當 fixture setup 開始跨檔重複時優先抽出

### 2.8 可變更性

- 如果下週要調整這個行為，我們應該去哪裡改？
- 這個修改需要碰一個地方，還是很多地方？
- 程式碼是否依照最可能變動的點來組織？
- 命名與模組邊界是在幫忙，還是在阻礙後續變更？

預設偏向：

- 依照預期變更點來組織，而不是只追求當下方便

可用工具 / 模組：

- `git grep` / `rg`：估算未來一次需求變更會牽動多少編輯點
- `ast-grep`：找出暗示未來會多檔同步修改的重複 policy / construction patterns
- `git log -p`、blame、history review：看變更是否反覆集中在同一批區塊

## 3. 反覆審查問題

當我們重新審視某個模組時，再問一次這六個問題：

1. 是否有任何檔案或函式比它需要的還大？
2. 哪些 literals 是合理 fixture 資料，哪些已經是維護債？
3. 我們是否在本地重做了共用邏輯？
4. 模組邊界是否仍然清楚？
5. 在面向使用者的表面上，失敗是否容易理解？
6. 若行為改變，我們是否知道唯一正確的修改位置？

## 4. 決策經驗法則

除非有很強的理由，否則預設採用以下經驗法則：

- 優先使用單一目的的檔案，而不是大型工具雜物間。
- 優先使用具名 helper，而不是重複 setup 區塊。
- 優先共用協定 helper，而不是本地複製 canonical 規則。
- 當 fixture literal 有助於閱讀時，優先保留可讀性，而不是過早抽象。
- 只有當 constant 具有共用語意或重複維護成本時，才優先抽出。
- 面向使用者的行為優先用 CLI-visible tests；演算法行為優先放在 core tests。
- 優先做能減少未來修改點數量的重構，而不是只減少目前行數。

## 5. 最低審查寫法

當我們指出一個程式碼品質問題時，通常至少應回答：

- 表面：
- 為什麼難維護：
- 它屬於可讀性問題、drift 風險，還是邊界問題：
- 這個問題是局部的，還是在其他地方也重複出現：
- 最小且安全的改善方案：
- 驗證計畫：

## 6. 建議的起始檢查

這些不是硬性規則，但很適合作為預設提示：

- 檔案大小警訊：大約 `300-500` 行
- 函式大小警訊：大約 `40-60` 行
- 重複 literal 警訊：同一個非 trivial literal 出現 `3+` 次
- drift 警訊：測試或 CLI helper 重做 canonicalization、signatures、hashing、replay 或 selector 邏輯
- 邊界警訊：同一模組混合 parsing、領域決策與 rendering

起始檢查可用工具 / 模組：

- 大小與熱點掃描：`wc -l`、`rg --files`、編輯器 outline
- 重複 literals：`rg`
- 結構重複或本地重實作：`ast-grep`
- 廣義結構搜尋 / 重寫實驗：`comby`
- 複雜度與 lint 訊號：`clippy`

## 7. 與其他表面的關係

請搭配下列文件一起使用這份檢查清單：

- [ROADMAP.zh-TW.md](../ROADMAP.zh-TW.md)
- [RUST-WORKSPACE.md](../RUST-WORKSPACE.md)
- [IMPLEMENTATION-CHECKLIST.zh-TW.md](../IMPLEMENTATION-CHECKLIST.zh-TW.md)
- [docs/FEATURE-REVIEW-CHECKLIST.zh-TW.md](./FEATURE-REVIEW-CHECKLIST.zh-TW.md)
- [AI-CO-WORKING-MODEL.md](./AI-CO-WORKING-MODEL.md)

如果這些表面彼此不一致，請依照
[PLANNING-SYNC-PLAN.zh-TW.md](./PLANNING-SYNC-PLAN.zh-TW.md) 的目前同步流程處理。
