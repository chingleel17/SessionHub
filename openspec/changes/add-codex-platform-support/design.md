## Context

SessionHub 目前的 provider 管線分成三塊：

- 設定與啟用狀態由 `AppSettings`、`settings.rs`、`SettingsView.tsx` 管理。
- session 掃描由 `src-tauri/src/sessions/mod.rs` 聚合，每個 provider 各自有獨立掃描器與 `ScanCache`。
- 即時刷新由 `watcher.rs` 依 provider 建立不同的檔案監看策略。

目前 Copilot 與 OpenCode 都有固定資料布局：Copilot 掃目錄中的 `workspace.yaml`，OpenCode 掃 `storage/session/<projectId>/ses_*.json`。你提供的 Codex 範例則是單一 `jsonl` 檔，位於 `~/.codex/sessions/<YYYY>/<MM>/<DD>/`，其中第一筆 `session_meta` 記錄已包含 `payload.id`、`payload.timestamp`、`payload.cwd`，其餘互動事件與訊息則以後續 JSONL 行持續附加。

這表示 Codex 與既有 provider 的主要差異在於：

- session 儲存單位是單一 `jsonl` 檔，而不是 session 專屬資料夾或 metadata JSON。
- 目錄分層由日期決定，不是 project ID 或 session ID。
- 若要取得 `updatedAt`，需要從檔案最後一筆可解析事件時間或檔案 mtime 推導，而不是直接讀固定欄位。

## Goals / Non-Goals

**Goals:**

- 在不破壞既有 Copilot / OpenCode 掃描流程的前提下，新增第三個獨立的 Codex provider。
- 讓使用者可在設定頁配置 Codex root，並透過 `enabledProviders` 啟用或停用 Codex。
- 將 Codex JSONL session 映射到既有 `SessionInfo` 介面，讓列表、專案分組、排序、標籤與篩選沿用現有 UI。
- 為 Codex 建立符合其檔案形狀的快取與 watcher 策略，避免每次刷新都全量重掃整棵日期目錄。

**Non-Goals:**

- 本次不實作 Codex 專屬 provider bridge / integration 安裝流程。
- 本次不承諾完整解析 Codex 所有訊息內容、token 統計或 activity status；先以 session 發現與基本 metadata 呈現為主。
- 本次不變更現有 session 卡片互動模型以外的產品流程，例如專屬 Codex resume 指令或進階 actions。

## Decisions

### 1. 新增 `codex_root` 並把 Codex 視為與 Copilot / OpenCode 平行的第三 provider

`AppSettings`、前端 `AppSettings` 型別、`get_sessions` / `restart_session_watcher` command 參數都要補上 `codexRoot`。預設值使用 `%USERPROFILE%\.codex`，實際掃描根目錄則固定取其下的 `sessions` 子目錄。

原因：目前 Copilot 與 OpenCode 都有獨立 root 欄位與解析函式，Codex 若重用 `opencodeRoot` 或硬編碼路徑，會讓設定模型失衡，也會讓 watcher / query key 無法精準失效。

替代方案：

- 不新增欄位，直接硬編碼 `~/.codex`。缺點是無法支援自訂位置，也不符合目前 settings 架構。
- 直接把 `codexRoot` 存成 `sessions` 目錄。缺點是 UI 會和其他 provider 的 root 定義不一致。

### 2. 為 Codex 新增獨立掃描模組 `sessions/codex.rs`

Codex 的 JSONL 結構與 OpenCode / Copilot 差異過大，不適合塞進既有掃描器。新模組負責：

- 遞迴掃描 `codexRoot/sessions/YYYY/MM/DD/*.jsonl`
- 讀取首個 `type == "session_meta"` 的記錄作為 session 基礎資訊
- 以檔名或 `session_meta.payload.id` 作為 `SessionInfo.id`
- 以 `payload.cwd` 對映 `SessionInfo.cwd`
- 以最後一筆事件的 `timestamp`，若缺失則退回檔案 mtime，作為 `updatedAt`
- 將 `session_dir` 指向 JSONL 檔案絕對路徑，讓後續功能可以把它當成 session 來源定位

原因：這能延續目前「每個 provider 各自封裝掃描規則、由 `sessions/mod.rs` 聚合」的結構，避免在單一函式中混入 provider 分支。

替代方案：

- 在 `sessions/mod.rs` 直接加 Codex 掃描細節。缺點是聚合層會失去單一職責。
- 把 Codex 視為 OpenCode 變形格式處理。缺點是資料來源、目錄形狀、cursor 策略都不同。

### 3. Codex 首版使用「檔案 mtime + 日期遞迴」的增量策略，而不是事件 cursor

OpenCode 可依資料中的 `time.updated` 與 SQLite cursor 做增量掃描；Codex JSONL 沒有等價的全域 cursor。首版採用與 Copilot 類似的 provider cache，但 `session_mtimes` 儲存的是 JSONL 檔案最後修改時間。重新整理時只重讀新增或 mtime 變化的檔案，並在必要時定期全掃。

原因：JSONL append-only 特性使 mtime 成為穩定且廉價的變更訊號，足以支撐 session 列表場景，不需要先引入更複雜的檔案尾端索引。

替代方案：

- 每次都全量讀取所有日期目錄。實作最簡單，但 session 累積後成本過高。
- 保存每個檔案最後讀取位移。精度更高，但會大幅增加快取格式與錯誤恢復複雜度。

### 4. 新增 Codex watcher，直接遞迴監看 `sessions` 根目錄

Codex 目錄是年/月/日分層，且 session 檔會持續 append，因此 watcher 應直接監看 `codexRoot/sessions` 遞迴樹。事件判斷只需關注 `.jsonl` 檔新增、修改、重新命名，並沿用現有 debounce / refresh dedup 機制。

原因：日期分層會每日產生新子目錄，若只監看當前日期資料夾，跨日後會漏資料。

替代方案：

- 啟動時只監看已存在的日期資料夾。缺點是新日期目錄需要額外重建 watcher。
- 完全依 bridge event 驅動刷新。Codex 目前沒有現成 integration，不可行。

### 5. UI 與 spec 僅擴充既有 provider-aware 行為，不新增 Codex 專屬頁面

前端只需把 provider 列舉、標籤、篩選按鈕、設定頁勾選與顯示文案擴充到 Codex。`buildProjectGroups`、Dashboard、ProjectView 與 SessionCard 應持續使用 `SessionInfo` 的共用欄位運作。

原因：這是最小可用路徑，並符合目前「provider 差異在掃描與少量顯示文案，主 UI 共享」的架構。

替代方案：

- 為 Codex 建立獨立頁面或專屬 project 視圖。缺點是需求未要求，且會放大範圍。

## Risks / Trade-offs

- [Codex JSONL 事件型別未完整掌握] -> 先只依賴 `session_meta` 與通用 `timestamp` 欄位；其餘欄位缺失時以檔名或檔案 metadata 補足。
- [遞迴掃描日期樹可能在大量歷史資料下偏慢] -> 導入 per-file mtime 快取，並保留既有定期全掃回退策略。
- [把 `session_dir` 指向 JSONL 檔案可能影響假設其為資料夾的功能] -> 在 implementation 階段逐一檢查 `read_plan`、stats、actions 等呼叫點，必要時對 Codex session 明確停用不適用功能。
- [新增第三 provider 可能遺漏排序或 label 常數] -> 將 provider 順序集中為共用常數，避免 `App.tsx`、`SettingsView.tsx` 各自硬編碼不同列表。
- [Watcher 遞迴監看根目錄在 Windows 上可能收到較多事件] -> 沿用現有 debounce 與 relevant-path 過濾，只接受 `.jsonl` 路徑變更。

## Migration Plan

1. 擴充設定模型與預設值，讓舊版 `settings.json` 在缺少 `codexRoot` 時自動補預設值。
2. 新增 Codex 掃描模組、快取欄位與 watcher，並把 `get_sessions` 聚合流程擴充為三 provider。
3. 更新前端型別、query key、設定頁與 provider UI 文案。
4. 以使用者提供的實際 Codex JSONL 範例建立 Rust 測試案例，驗證 session 解析、分組與增量掃描。
5. 上線後若發現部分 Codex 檔案缺少 `session_meta` 或 `cwd`，先以 parse error / fallback 行為呈現，不阻斷其他 provider。

## Open Questions

- Codex 是否有穩定的 resume / reopen 指令格式，可供 `session-actions` 後續整合？
- Codex JSONL 是否存在可直接推導摘要、訊息數或 token 使用量的標準事件，供未來 `session-stats` 擴充？
- `session_dir` 對 Codex 應維持指向 JSONL 檔案，還是改為其所在日期資料夾並新增額外 `sourcePath` 欄位？目前為了減少 schema 變更，設計上先採前者。
