# 規劃同步計畫

狀態：目前有效的 planning surfaces 同步工作約定

這份文件定義 Mycel 如何同步以下幾類介面：

- repo 層級的 planning Markdown，特別是 `ROADMAP.md`、`ROADMAP.zh-TW.md` 與 `IMPLEMENTATION-CHECKLIST.*`
- GitHub Issues
- GitHub Pages 上的 planning 摘要頁面

它的目的，是避免權威建置順序、可執行任務池、以及公開進度頁之間發生漂移。

## 1. 適用範圍

這份計畫適用於：

- [`ROADMAP.md`](../ROADMAP.md)
- [`ROADMAP.zh-TW.md`](../ROADMAP.zh-TW.md)
- [`IMPLEMENTATION-CHECKLIST.en.md`](../IMPLEMENTATION-CHECKLIST.en.md)
- [`IMPLEMENTATION-CHECKLIST.zh-TW.md`](../IMPLEMENTATION-CHECKLIST.zh-TW.md)
- [`docs/PROGRESS.md`](./PROGRESS.md)
- [`docs/progress.html`](./progress.html)
- [`README.md`](../README.md) 與 [`README.zh-TW.md`](../README.zh-TW.md) 中精選的 contributor-entry issue 連結
- GitHub Issues，尤其是 `ai-ready` 類任務

這份計畫不適用於：

- 不會改變實作順序或 closure 狀態的 protocol wording 微調
- 純視覺性的首頁或 support page 調整
- 不影響專案 planning 狀態的 issue triage

## 2. Source of Truth 順序

當不同介面彼此不一致時，採用以下權威順序：

1. `ROADMAP.md` 與 `ROADMAP.zh-TW.md`
2. `IMPLEMENTATION-CHECKLIST.*`
3. GitHub Issues
4. `docs/PROGRESS.md`
5. `docs/progress.html`
6. landing page 或 support page 上的摘要說法
7. README 中精選的 contributor-entry issue 連結

解讀方式：

- `ROADMAP.md` 與 `ROADMAP.zh-TW.md` 共同擁有 milestone 順序、phase 邊界、以及 build sequence 的權威性。
- `IMPLEMENTATION-CHECKLIST.*` 擁有 section-level closure 狀態與 readiness gate 的權威性。
- GitHub Issues 代表剩餘缺口的可執行切片。
- `docs/PROGRESS.md` 與 `docs/progress.html` 是衍生摘要，不得自行發明專案狀態。
- `README.*` 中精選的 issue 連結也是給 contributor 看的衍生摘要，必須指向目前仍有效的 `ai-ready` 任務。

## 3. 各表面的角色

### 3.1 `ROADMAP.*`

用 `ROADMAP.md` 與 `ROADMAP.zh-TW.md` 回答：

- 我們現在在哪個 phase
- 下一個 lane 是什麼
- milestone 順序是什麼
- 目前 lane 刻意排除哪些東西

以下情況應更新 `ROADMAP.*`：

- milestone 重心改變
- repo 從一個 phase 邊界移動到另一個
- 目前 lane 的主要缺口已實質改變

### 3.2 `IMPLEMENTATION-CHECKLIST.*`

用 checklist 回答：

- 什麼已經實作
- 什麼仍未完成
- 哪些是 partial、哪些可以 close
- 哪些 readiness gate 仍被卡住

以下情況應更新 checklist：

- 某個具體實作項目完成
- 原本 open 的項目變成 partial 或可 close
- 某個 readiness gate 的狀態改變

### 3.3 GitHub Issues

用 GitHub Issues 回答：

- 接下來有哪些窄而可執行的工作
- 哪些 checklist gaps 已被切成任務
- 哪些工作可以委派給 bot 或平行 contributor

Issues 不應取代 roadmap 或 checklist 的專案狀態，只應反映它們。

### 3.4 Pages 進度表面

用 `docs/PROGRESS.md` 與 `docs/progress.html` 回答：

- 讀者應該快速理解什麼
- 現在活躍的是哪個 milestone lane
- 哪些 checklist sections 是 mostly done、partial、或 not started

Pages 必須維持 summary-first。它們應該壓縮 planning 狀態，而不是定義 planning 狀態。

## 4. 同步規則

### 4.1 若 milestone 狀態改變

依這個順序更新：

1. `ROADMAP.md`
2. `IMPLEMENTATION-CHECKLIST.*`
3. `docs/PROGRESS.md`
4. `docs/progress.html`
5. 關閉或新增相關 GitHub Issues

例子：

- `M1` 從「late partial」進到「足以開始收尾」
- `M2` 成為更明確的 active lane

### 4.2 若某個 checklist 項目完成，但 phase 沒變

依這個順序更新：

1. `IMPLEMENTATION-CHECKLIST.*`
2. 相關 GitHub Issue 狀態
3. 若 section-level 狀態改變，再更新 `docs/PROGRESS.md`
4. 若公開摘要用語改變，再更新 `docs/progress.html`

例子：

- `Implement snapshot parsing` 完成
- 但 roadmap 的 active lane 沒變

### 4.3 若發現新的可執行缺口

依這個順序更新：

1. 若這個缺口是真實且持久的，先更新 `IMPLEMENTATION-CHECKLIST.*`
2. 若它夠窄、可執行，再開 GitHub Issue
3. 除非 milestone 重心改變，否則不要更新 `ROADMAP.md`
4. 只有在 section 狀態實質改變時才更新 progress 摘要

### 4.4 若 issue triage 只改變執行形狀

只更新：

1. GitHub Issues

下列情況不應更新 roadmap、checklist、或 pages：

- 專案底層狀態沒變
- 只是把工作拆得更細
- 只是改 labels 或 ownership，沒有影響 closure 狀態

### 4.5 若 Pages 只是為了可讀性調整

更新：

1. `docs/PROGRESS.md`
2. `docs/progress.html`

除非底層狀態真的改變，否則不要去動 roadmap 或 checklist。

## 5. GitHub Issue 對應規則

### 5.1 Issue 來源

每一張 planning-oriented 實作 issue 都應對應到下列其中之一：

- 一個 checklist item
- 一個 checklist 子缺口
- 一個 milestone-close proof point

避免一張 issue 跨越多個無關的 checklist sections。

### 5.2 建議 issue metadata

每張 issue 應該包含：

- 它支援哪個 checklist section 或 roadmap milestone
- start files
- acceptance criteria
- verification commands
- non-goals

對 bot-friendly 任務，建議使用：

- `ai-ready`
- `well-scoped`
- `tests-needed`
- `fixture-backed`
- 適用時加 `spec-follow-up`

### 5.3 Issue lifecycle

採用以下生命周期：

1. 缺口真實且可執行時開 issue
2. 只要 checklist 對應缺口仍未實質關閉，就保持 open
3. 當該 issue 的窄 acceptance criteria 已完成時關閉
4. 若更大的 checklist closure 仍未完成，就開 follow-up issues，而不是讓原 issue 保持模糊

## 6. Pages 衍生規則

### 6.1 `docs/PROGRESS.md`

這是公開 progress view 的 Markdown 摘要來源。

它應該：

- 重述 active lane
- 壓縮 milestone 狀態
- 壓縮 checklist section 狀態
- 連回 roadmap 與 checklist 權威來源

它不應該：

- 引入 `ROADMAP.md` 沒有的 milestone 名稱
- 在 checklist 沒有對應依據時把某個區塊標成完成
- 推測 roadmap 尚未定義的未來 phase

### 6.2 `docs/progress.html`

這個檔案應被視為 `docs/PROGRESS.md` 的展示層。

當 planning 狀態改變時：

- 先更新 `docs/PROGRESS.md`
- 再更新 `docs/progress.html`

若兩者不一致，應該把 HTML 修回和 Markdown 摘要一致，而不是反過來。

## 7. 更新節奏

### 7.1 事件驅動更新

在下列情況下，立即更新 planning surfaces：

- milestone 有實質前進
- checklist section 狀態改變
- active implementation lane 改變

### 7.2 Commit-count refresh

使用：

```bash
scripts/check-doc-refresh.sh
```

若它回報 `due`，則在下一個 docs-sync batch 更新：

- `ROADMAP.md`
- `ROADMAP.zh-TW.md`
- `IMPLEMENTATION-CHECKLIST.en.md`
- `IMPLEMENTATION-CHECKLIST.zh-TW.md`
- 對齊的 GitHub Issues
- GitHub Pages 上的 planning 摘要層，例如 `docs/PROGRESS.md` 與 `docs/progress.html`

即使沒有任何單一 commit 強迫你更新，也要做這次 refresh。

## 8. 建議同步流程

對一個有實質意義的 implementation batch：

1. 先落 code 與 tests
2. 判斷 checklist 狀態是否改變
3. 判斷 roadmap 重心是否改變
4. 更新 GitHub Issues
5. 更新 `docs/PROGRESS.md`
6. 更新 `docs/progress.html`
7. 跑相關驗證與 doc-refresh 檢查

對一個 docs-only planning refresh：

1. refresh `ROADMAP.md`
2. refresh `IMPLEMENTATION-CHECKLIST.*`
3. 對齊 issues
4. 再更新 `docs/PROGRESS.md`
5. 再更新 `docs/progress.html`
6. 確認 GitHub Pages 上的 planning 摘要與刷新後的 roadmap/checklist/issues 狀態一致
7. 若目前建議起手的 issue 已改變，同步刷新 `README.*` 中精選的 contributor-entry 連結

## 9. Anti-Drift 規則

不要讓以下狀況長期存在：

1. roadmap 說 lane 已經改變，但 progress page 還顯示舊 lane
2. checklist 把某項標成完成，但相關 issue 沒關，也沒有 follow-up split
3. progress page 說某區塊 mostly done，但 checklist 仍大多未勾
4. issue 標題漂移成 roadmap 尚未支持的推測性工作
5. Pages 自行發明 roadmap 或 checklist 沒有的專案狀態說法
6. README 的 contributor-entry 連結指向過期、已關閉、或已不具代表性的 issue

## 10. 最低完成條件

當以下條件都成立時，可視為 planning surfaces 已同步：

- roadmap 的 milestone wording 符合目前 active lane
- checklist boxes 反映目前實作 closure
- open issues 對應到真實剩餘缺口
- `docs/PROGRESS.md` 的摘要與 roadmap/checklist 一致
- `docs/progress.html` 與 `docs/PROGRESS.md` 一致
- `README.*` 中精選的 contributor-entry 連結仍指向具代表性的 open starter issues

## 11. 對 Mycel 目前的實務指引

目前請採用以下具體規則：

1. 把 `ROADMAP.md` 當成 milestone 與 lane 的權威來源
2. 把 `IMPLEMENTATION-CHECKLIST.*` 當成 closure 狀態的權威來源
3. 把 open 的 `ai-ready` issues 視為 checklist gaps 的窄執行切片
4. 把 `docs/PROGRESS.md` 與 `docs/progress.html` 視為公開摘要層
5. 把 `README.*` 中精選的 contributor issue 連結視為窄的公開入口，並在 planning sync 時一起刷新

這樣可以讓 roadmap、implementation closure、task queue、以及公開進度頁保持一致，而不會讓任何一個表面被迫承擔過多角色。
