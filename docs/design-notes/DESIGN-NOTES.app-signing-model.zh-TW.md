# Mycel App-signing Model

狀態：design draft

這份文件描述 Mycel-based system 應如何把 application signing（應用簽章）視為分層模型，而不是單一簽章決策。

核心設計原則是：

- application state signing 不等於 release signing
- release signing 不等於 execution-evidence signing
- 每一層 signing 保護的是不同類型的 trust
- 安全部署不應假設某一層會自動取代其他層

## 0. 目標

讓 Mycel-based application system 至少能分清楚三種不同的 signing 需求：

- 對 app-layer objects 與 governance state 的簽章
- 對 release software artifacts 的簽章
- 對 execution receipts 或 runtime attestations 的簽章

這份文件不定義單一強制的 signing toolchain。

它定義的是 signing model 應保留的 trust boundaries。

## 1. 為什麼單一 Signature 不夠

如果系統只說「the app is signed」，這句話本身其實很模糊。

它可能表示：

- app manifest 已簽章
- 可下載的 binary 已簽章
- runtime receipt 已簽章

這些保護的是不同風險。

因此 deployment 應把它們分開建模。

## 2. 第一層：App-layer Object Signing

這一層保護由 Mycel 承載的 application records。

常見被簽的 objects：

- `app_manifest`
- app governance records
- policy objects
- proposal 與 approval records
- effect requests
- 當 effect receipts 被建模成 Mycel objects 時，也應簽章

主要目的：

- 證明 authorship
- 保護 record integrity
- 保留 governance 與 state history

這一層屬於 Mycel object 與 profile model 的一部分。

它應對齊：

- canonical serialization
- object-level signature verification
- accepted-state derivation rules

但這一層本身，不能證明下載下來的 software artifact 就是可信的。

## 3. 第二層：Release Artifact Signing

這一層保護被發行出去的 software artifacts。

常見被簽的 artifacts：

- CLI binaries
- application packages
- installers
- container images
- release manifests

主要目的：

- 保護 software supply chain
- 證明 artifact origin
- 偵測被替換或竄改的 releases

即使所有 Mycel objects 都已簽章，這一層仍然重要。

原因：

- 使用者仍可能下載到惡意 client，而該 client 可能錯誤驗證 Mycel objects，或在本地洩漏 secrets

這一層屬於 build、release 與 distribution pipeline，而不是 protocol core。

## 4. 第三層：Execution-evidence Signing

這一層保護 runtime 實際做了什麼的證據。

常見被簽的 evidence：

- execution receipts
- settlement receipts
- runtime attestations
- external effect confirmations

主要目的：

- 證明是哪個 runtime 或 executor 做了某個動作
- 保留事後 auditability
- 區分 intended action 與 completed side effect

這一層對以下系統尤其重要：

- payment execution
- custody systems
- external effect systems
- disputes 與 incident review

這一層不應與 release signing 混為一談。

runtime 可能是真實的，但某一筆 execution receipt 仍需要自己可驗的 authorship。

## 5. Core vs App vs Runtime 邊界

protocol core 應提供一般性的 signature-verification capability。

app 與 profile layer 應定義：

- 哪些 Mycel objects 必須被簽
- 每種 object family 由誰簽才算有效
- 這些 signed records 如何參與 accepted-state selection

runtime 與 release pipeline 應定義：

- software artifacts 如何簽章
- effect receipts 如何簽章
- runtime identities 如何管理

這樣可以在保持 protocol core 穩定的同時，讓高層 signing model 繼續演進。

## 6. 常見 Failure Cases

### 6.1 只有 Object Signing，沒有 Release Signing

所有 governance records 都有簽章，但使用者下載的是未簽的 binaries。

結果：

- governance history 可能是真的
- client supply chain 仍可能被攻破

### 6.2 只有 Release Signing，沒有 Object Signing

發出去的 binary 是真的，但 governance 與 app-state objects 的 authorship 規則很弱。

結果：

- software origin 有保護
- app-layer authority 與 history 仍然薄弱

### 6.3 Runtime 可信，但 Receipt 沒簽

runtime 本身可信，但 execution evidence 沒有被簽章，也無法歸屬。

結果：

- 事後 audit 與 dispute handling 變弱

### 6.4 把某一層當成全部層的替代品

系統假設某一種 signature 類型就能解決全部 trust 問題。

結果：

- supply chain、governance history 或 effect auditability 仍留下隱藏缺口

## 7. 建議的最低基線

實務部署通常至少應提供：

1. 已簽的 app-layer governance 與 state objects
2. 已簽的 release artifacts
3. 對高風險 runtimes 產生已簽的 execution receipts

最小部署可以先從第一層開始。

但安全敏感部署不應停在那裡。

## 8. 實務判準

真正該問的不是：

- 「這個 app 有沒有簽章？」

而是：

- 「系統的哪一部分被誰簽了，而那個 signature 保護的是哪一條 trust boundary？」

如果 deployment 無法清楚回答，代表它的 signing model 很可能還沒有定義完整。
