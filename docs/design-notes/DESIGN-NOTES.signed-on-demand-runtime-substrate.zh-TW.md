# Mycel as a Signed On-Demand Runtime Substrate

狀態：design draft

這份筆記探索一種更進一步的 Mycel 解讀方式：Mycel 不只是可驗證文字與治理的協議，也可以被視為一種讓大部分 application logic 按需抓取、驗證與執行的 substrate。

核心想法是：

- 本地只保留很小的 trusted local runtime
- 把可執行部分建模成已簽章、content-addressed 的 modules
- 只在需要時抓取缺少的 modules
- 沒有使用中的 modules 可以完全不存在於本機

這不是要把 Mycel 變成一個 monolithic operating system kernel。

這比較像是把 Mycel 視為一個供高層 application behavior 使用的 signed、distributed runtime substrate。

相關文件：

- `DESIGN-NOTES.dynamic-module-loading.*`：較窄版的 module-loading 模型
- `DESIGN-NOTES.mycel-app-layer.*`：app state 與 side-effect execution 的分層
- `DESIGN-NOTES.app-signing-model.*`：object signing、artifact signing 與 runtime evidence 的差異

## 0. 目標

讓一個 Mycel-based system 可以做到：

- 可執行功能在需要前不必存在本機
- 缺少的功能可以在當下即時抓取
- 每一個被抓取的可執行 artifact 都有簽章且可 content-address
- 本地只保留驗證與執行這些 artifacts 所需的最小 trusted runtime

同時保留：

- 明確的 trust boundaries
- 對實際執行程式碼的 auditability
- 對 code artifacts 的 deterministic identity
- 即使 artifact 有效，本地仍保有拒絕執行的能力

## 1. 這是什麼，不是什麼

這種模型比較接近：

- signed application substrate
- distributed runtime environment
- content-addressed module host

它不一定是：

- 完整取代硬體導向 OS kernel 的方案
- 聲稱本地 literally 可以沒有任何常駐 code
- 保證所有 execution 都能完全 network-transparent

某種程度的 local bootstrap 仍然必須存在。

## 2. 小型 Trusted Local Base

如果本地完全是空的，就不可能安全地執行抓下來的 modules，因為總要先有某部分常駐來負責：

- 開機
- 建立網路連線
- 驗證 signatures 與 hashes
- 執行 local policy
- 提供 sandboxing
- 提供 execution runtime

所以實際模型會是：

- 一個很小的 trusted local base 常駐
- 更高層的 logic 則變成可抓取、可替換

這個 trusted base 越小，系統就越接近真正的 on-demand runtime substrate。

## 3. 執行模型

系統應把 code 分成三類。

### 3.1 Resident Base

永遠存在本地。

責任：

- bootstrapping
- verification
- module cache management
- policy enforcement
- sandbox runtime hosting

### 3.2 On-Demand Modules

在需要時抓取。

例如：

- renderers
- transformers
- app-specific logic
- policy helpers
- protocol-adjacent extension logic

### 3.3 Optional Cached Artifacts

只為了速度或 offline continuity 才保留在本地。

這些 artifacts 應該可以被刪除，而不改變它們的 identity，因為 identity 來自 content hash 與 signature，不是來自本機安裝狀態。

## 4. 為什麼所有可抓取 code 都要簽章

如果系統會在 runtime 抓取可執行 artifact，那 signature checks 就不能只是附帶 metadata。

它們本身就是 execution boundary 的一部分。

因此每個可抓取的可執行部分都應具備：

- 穩定的 artifact identity
- content hash
- 有效 signature
- 宣告清楚的 runtime target
- 宣告清楚的 capability request

這能降低 runtime fetch 淪為任意 remote-code-execution 通道的風險。

## 5. Content Addressing 與預設不存在

理想的儲存規則是：

- 當下不需要的 code，不必存在於本地

這意味著：

- modules 應以 identity 參照，而不是只靠 installation path
- 本地缺少 module 是正常狀態，不是錯誤
- fetch 是標準的 resolution 步驟

host 應能自然地表達：

-「我現在需要這個 module」
-「它目前不在本地」
-「現在抓下來並驗證」

而不是把這個流程視為異常狀況。

## 6. Runtime Fetch Flow

建議流程：

1. accepted app state 或某個 local action 需要一個 module
2. local runtime 解析出所需 module identity
3. 如果本地 cache 裡沒有，runtime 就從被允許的來源抓取
4. runtime 驗證 signature、hash、runtime target 與 local policy
5. runtime 在 sandbox 中載入它
6. execution metadata 被記錄下來，供之後 audit

如果驗證失敗，artifact 就算語法正確，也應維持不可執行。

## 7. 為什麼這很像分散式作業模型

這種模型會開始看起來像 distributed operating model，因為：

- 可執行 behavior 不是預設永遠安裝在本地
- host 需要透過網路或 content graph 解析 code
- execution 依賴遠端 artifact availability
- 本地 storage 更像 verified cache，而不是完整 install image

但它和傳統 distributed OS 仍有一個重要差異：

- trust 與 artifact verification 在這裡是 first-class design 元件，不是次要的 packaging 問題

## 8. 建議的 Artifact 模型

這個 substrate 預設不應抓取 raw native code fragments。

比較好的做法是採用結構化模型：

- signed module metadata object
- signed 或至少 hash-bound 的 module blob
- 明確的 runtime target
- 明確的 capability request

最安全的第一版應是：

- `WASM` modules
- content-addressed blobs
- host-mediated capability APIs

這樣可讓 substrate 保持可攜性，也更容易 audit。

## 9. Capability 與 Policy 邊界

有效 signature 的意思應該是：

-「這確實是 signer 想發佈的 artifact」

它不應自動代表：

-「本機就必須執行它」

host 仍然需要 local policy checks：

- 這個 signer 值得信任嗎？
- 這個 module family 被允許嗎？
- 這些 requested capabilities 能接受嗎？
- 在當前 local state 下允許執行嗎？

這能保留本地對 execution 的主權。

## 10. Cache，而不是 Installation

在這個模型裡，本地 storage 的角色比較像 verified execution cache，而不是傳統軟體安裝。

建議屬性：

- cache entries 以 content hash 為 key
- 未使用 modules 可安全 eviction
- 下次需要時可重用 exact version
- 只有已快取的 artifacts 才能支援 offline execution

這表示系統可以在大部分時候保持 stateless，同時又不必讓每次重複執行都完全依賴網路。

## 11. 哪些地方仍然需要 Determinism

不是所有抓取來的 modules 都有相同語義重量。

至少可分三類：

### 11.1 Pure Presentation Modules

例如：

- renderers

Determinism 很有幫助，但對 accepted-state derivation 的影響較小。

### 11.2 State-Interpreting Modules

例如：

- policy helpers
- transformation logic

這時 determinism 重要得多，因為不同輸出可能改變 app behavior。

### 11.3 Side-Effecting Modules

例如：

- 透過 host capabilities 觸發 HTTP calls 或本地動作的 modules

這類需要最強的 audit 與 policy controls。

host 應區分這些類別，而不是把所有 modules 視為同一種東西。

## 12. 為什麼 Native Code 不該是預設

如果這個 substrate 預設抓取 native binaries 或 dynamic libraries，它就會繼承：

- platform-specific packaging 問題
- 更困難的 sandboxing
- 更寬的 privilege surface
- 更差的 portability

這會讓設計滑向高風險的 remote-install 模式，而不是保守的 signed runtime substrate。

因此預設方向應保持在：

- 很小的 local host
- sandbox runtime
- signed portable modules

## 13. 建議的第一個實務形態

這個想法第一個較實際的形態可以是：

1. 一個 local Mycel host runtime
2. 按需載入的 signed `WASM` modules
3. content-addressed module blobs
4. local capability policy
5. execution audit logs
6. 在 module 未使用時允許本地 cache eviction

這樣已足以逼近「signed、fetch-on-demand 的 runtime substrate」，同時不必假裝 Mycel 已經取代完整 OS stack。

## 14. 開放問題

- trusted local base 最小可以縮到什麼程度，才不會讓系統變得不實用？
- module signer policy 應該 purely local、綁 profile，還是可由 governance 輔助？
- 哪些 module 類別若會影響 accepted-state derivation，就必須有更嚴格的 determinism 要求？
- offline mode 是否應要求顯式 pin 住 critical modules？
- 抓下來的 module 什麼時候應保留在 cache，什麼時候應在執行後立刻丟棄？
