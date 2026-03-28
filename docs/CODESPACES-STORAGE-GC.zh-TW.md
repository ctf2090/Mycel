# Codespaces 儲存空間 GC

這份文件定義一套安全、可重複執行的 Mycel Codespaces 儲存空間垃圾清理計畫。

適用情境：

- Codespace 提示磁碟空間不足
- `/workspaces` 被可重建產物塞滿
- 我們想用固定流程處理，而不是每次臨時手動清理

## 目標

- 回收空間時不碰 tracked source files
- 優先清理可重建輸出與快取，而不是使用者工作樹內容
- 先把清理計畫列出來，再決定是否刪除

## 工具

請使用 `scripts/codespaces_storage_gc.py`。

預設行為是 dry-run，只列出目前 workspace 內可回收的路徑。

```bash
python3 scripts/codespaces_storage_gc.py
```

若要把 Cargo 與 npm 這類 home 目錄快取也納入：

```bash
python3 scripts/codespaces_storage_gc.py --include-home-caches
```

若要真的刪除所選目標：

```bash
python3 scripts/codespaces_storage_gc.py --apply
python3 scripts/codespaces_storage_gc.py --apply --include-home-caches
```

若只想處理特定目標：

```bash
python3 scripts/codespaces_storage_gc.py --target repo-target
python3 scripts/codespaces_storage_gc.py --apply --target cargo-registry-cache --target npm-cache
```

若要和其他工具串接：

```bash
python3 scripts/codespaces_storage_gc.py --json
```

## 預設 GC 計畫

建議依這個順序執行：

1. 先對 workspace-only targets 做 dry-run，確認最大宗可回收路徑。
2. 優先刪除 workspace build outputs，尤其是 `target/`。
3. 重新確認剩餘可用空間。
4. 若壓力仍在，再用 `--include-home-caches` 把 home 目錄快取納入。
5. 清理後再視需要重建或重新下載快取。

## 執行節奏

Codespaces storage GC review 至少每 `400` commits 執行一次。

以 `2026-03-26` 這個時間點、目前這份 Mycel clone 的 commit 節奏來看：

- 在 `2026-03-08 17:54:36 UTC+8` 到 `2026-03-26 17:18:27 UTC+8` 之間共有 `1334` commits
- 平均約 `74.21 commits/day`
- 因此每 `400` commits 大約是 `5.39 days`，也就是約 `5 天 9 小時`

請把 `400 commits` 視為固定觸發條件，而把天數估算視為會隨專案節奏變動的操作參考值；如果 commit cadence 改變，就重新計算。

## 目前支援的目標類別

- `repo-target`：`target/` 下的 Cargo 編譯輸出
- `repo-tmp`：`tmp/` 下的 workspace 暫存資料
- `repo-pytest-cache`：`.pytest_cache/`
- `repo-node-cache`：`node_modules/.cache/`
- `cargo-registry-cache`：`~/.cargo/registry/cache/`
- `cargo-git-db`：`~/.cargo/git/db/`
- `npm-cache`：`~/.npm/_cacache/`
- `pip-cache`：`~/.cache/pip/`

這個工具只會處理這份 allowlist，並略過 symlink、缺少的路徑與非目錄目標。

## 備份注意事項

有些 Codespace 狀態存在 repo 外面；如果我們想保留本機 shell 或 agent 設定，做手動備份時要把這些檔案一起納入。

請明確備份以下檔案：

- `/home/codespace/.codex/skills/boot-agent/agents/openai.yaml`
- `/home/codespace/.bashrc`

## 操作備註

- 預設只處理 workspace targets，因為它們最不容易讓人意外，而且通常能回收最多空間。
- Home 快取列為可選項，因為清掉之後可能讓後續安裝或編譯變慢。
- 使用 `--apply` 時，工具會同時回報清理前與清理後的可用空間。
- 如果 allowlisted cleanup 做完後空間還是太少，再手動檢查大型非標準目錄，不要直接刪其他內容。
