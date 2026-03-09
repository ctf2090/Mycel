# Mycel Signature Role Matrix

狀態：design draft

這份文件把目前 repo 裡的主要 object families 對到應該簽它們的角色。

核心設計原則是：

- 某個 object 的簽署者，應對應到該 object 所聲稱表達的 authority
- 一個角色不應靜默繼承另一個角色的 signing power
- runtime authorship、governance authorship 與 signer consent 應保持可區分
- 第一版實作應先定義狹窄的預設矩陣，再逐步加入例外

相關文件：

- `DESIGN-NOTES.signature-priority.*`：哪些 objects 應最先要求簽章
- `DESIGN-NOTES.app-signing-model.*`：三層 signing model
- `DESIGN-NOTES.policy-driven-threshold-custody.*`：custody 專用 object families
- `DESIGN-NOTES.auto-signer-consent-model.*`：signer consent 邊界

## 0. 目標

給出第一版對這些問題的回答：

- 誰簽什麼
- 哪些角色可以 co-sign
- 哪些角色預設不應簽某類 object

這份文件聚焦 Mycel 承載的 objects，不處理 release artifacts。

## 1. 角色

這份文件使用以下角色標籤：

- `app-author`
- `app-maintainer`
- `governance-maintainer`
- `editor-maintainer`
- `view-maintainer`
- `signer`
- `signer-runtime`
- `runtime`
- `executor`
- `operator`

不是每個 app 都需要每一種角色。

某些 deployment 可能把多個角色收斂到同一把 key，但那應是明確的 profile 選擇，而不是隱含假設。

## 2. 矩陣

### 2.1 `app_manifest`

- 主要簽署者：`app-author` 或 `app-maintainer`
- 可 co-sign：`governance-maintainer`
- 預設不應簽：`runtime`、`executor`、`operator`

原因：

- 這個 object 定義的是 app identity 與 scope，不是 runtime execution

### 2.2 Governance proposal

- 主要簽署者：`governance-maintainer`
- app 專用變體可使用：`editor-maintainer` 或 `view-maintainer`，依 profile 而定
- 預設不應簽：`runtime`、`executor`

原因：

- proposal 表達的是 governance intent，不是 side-effect execution

### 2.3 Governance approval 或 selector signal

- 主要簽署者：`governance-maintainer` 或 `view-maintainer`
- 可 co-sign：profile 定義的其他 governance-authorized roles
- 預設不應簽：`runtime`、`executor`、`operator`

原因：

- selector authority 應與 runtime 行為保持分離

### 2.4 `signer_enrollment`

- 主要簽署者：`signer`
- 可 co-sign：`governance-maintainer` 或 `operator` 作為確認
- 預設不應簽：不相干的 `runtime`、`executor`

原因：

- enrollment 必須能證明 signer 的確知情加入

### 2.5 `consent_scope`

- 主要簽署者：`signer`
- 可 co-sign：當 profile 要求顯式 acknowledgement 時，可由 `governance-maintainer`
- 預設不應簽：`runtime`、`executor`

原因：

- consent 必須來自其 key 或 key share 被綁定的 signer 本人

### 2.6 `signer_set`

- 主要簽署者：`governance-maintainer`
- 可 co-sign：被授權的 custody-governance role
- 預設不應簽：`signer-runtime`、`runtime`、`executor`

原因：

- signer-set membership 是 governance fact，不是 runtime observation

### 2.7 `policy_bundle`

- 主要簽署者：`governance-maintainer`
- 可 co-sign：profile 定義的 policy-authorizing role
- 預設不應簽：`runtime`、`executor`

原因：

- policy authorization 不應靜默委派給 executors

### 2.8 `pause_or_revoke_record`

- 主要簽署者：`governance-maintainer`
- 在 profile 允許 signer-local emergency control 時，也可由 `signer` 簽
- 預設不應簽：`runtime`、`executor`、`operator`

原因：

- 這類 object 會改變未來 execution eligibility，不能退化成 local runtime override

### 2.9 `trigger_record`

- 主要簽署者：擁有 trigger source 的角色
- 常見情況：
  - governance-approved trigger 由 `governance-maintainer`
  - trusted runtime-derived trigger 由 `runtime`
- 預設不應簽：不相干的 `executor`

原因：

- trigger 必須可歸屬到真正觀察或授權該觸發條件的系統元件

### 2.10 `execution_intent`

- 主要簽署者：被授權的 `runtime` 或 governance-derived execution authority
- 可 co-sign：若 profile 想在 settlement 前要求 executor acknowledgement，則可由 `executor`
- 預設不應簽：一般 `operator`

原因：

- intent 綁定的是可執行上下文，應由能從 accepted state 合法導出它的權限來源簽署

### 2.11 `signer_attestation`

- 主要簽署者：`signer-runtime` 或 `signer`
- 預設不應簽：`runtime`、`executor`、`operator`

原因：

- attestation 是 signer 端對「檢查通過且已產生簽章結果」的聲明

### 2.12 `execution_receipt`

- 主要簽署者：`executor` 或 execution `runtime`
- 可 co-sign：若 profile 把 execution 與 observation 分開，則可由 settlement-observer runtime 共簽
- 預設不應簽：`governance-maintainer`、`signer`

原因：

- receipt 應證明 execution layer 實際發生了什麼

### 2.13 一般性的 `effect_receipt`

- 主要簽署者：`runtime`
- 可 co-sign：當 runtime 委派給特定 executor 時，可由其共簽
- 預設不應簽：governance roles

原因：

- effect receipt 是 runtime evidence，不是 governance decision

## 3. 預設分離規則

第一版矩陣應遵守以下規則：

1. governance roles 簽 governance records
2. signers 簽 enrollment 與 consent
3. signer runtimes 簽 signer attestations
4. runtimes 與 executors 簽 effect 與 settlement receipts
5. operators 不應只因為他們在跑基礎設施，就變成預設 authority signer

## 4. 危險的角色坍縮

以下角色合併雖然可能可行，但若沒有明確 profile 定義，風險很高：

- `governance-maintainer` + `executor`
- `signer` + `governance-maintainer`
- `runtime` + `selector authority`
- `operator` + 所有 signing roles

這些合併雖然方便，但會削弱 attribution clarity，也容易隱藏權力集中。

## 5. 目前 Repo 的最小第一版矩陣

如果現在要在 repo 裡先定一個狹窄預設矩陣，最穩妥的起點是：

- `app_manifest` -> `app-author` 或 `app-maintainer`
- governance proposal / approval -> `governance-maintainer`
- `signer_enrollment` -> `signer`
- `consent_scope` -> `signer`
- `signer_set` -> `governance-maintainer`
- `policy_bundle` -> `governance-maintainer`
- `trigger_record` -> 擁有 trigger 的 authority
- `execution_intent` -> 被授權的 `runtime`
- `signer_attestation` -> `signer-runtime`
- `execution_receipt` -> `executor` 或 execution `runtime`

## 6. 實務判準

對任何 object family，都先問：

- 這個 object 聲稱表達的是誰的 authority？

那個角色就應該先簽。

如果預設改由別的角色來簽，系統就應明確解釋為什麼這種替代仍然安全且可見。
