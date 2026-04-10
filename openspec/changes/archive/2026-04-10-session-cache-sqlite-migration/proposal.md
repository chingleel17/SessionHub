## Why

目前 session 清單在掃描完成後以 `session_cache.json`（`%APPDATA%\SessionHub\session_cache.json`）持久化，供下次啟動時立即呈現。這個機制有以下問題：

1. **雙重儲存機制**：`metadata.db`（SQLite）已用於備註、標籤、統計快取，再維護一個 JSON 平行檔案增加了複雜度。
2. **缺乏 mtime 失效**：JSON 快取只能全量取代，無法記錄各 session 目錄的 mtime，增量掃描仍需要讀取磁碟上的 mtime（`session_mtimes` 存於記憶體 `ScanCache`，重啟即清空）。
3. **重啟後冷啟動**：應用程式重啟後 `ScanCache` 清空，首次 `get_sessions` 總是全量掃描，完成前畫面以舊的 JSON 快取為準，兩者出現短暫不一致。
4. **schema 無法演進**：JSON array 無法輕易加欄位篩選，也無法快速查詢單筆記錄。

## What Changes

- 在 `metadata.db` 新增三個資料表：`sessions_cache`（session 清單快取）、`scan_state`（per-provider 掃描狀態）、`session_mtimes`（per-session mtime）。
- `save_session_cache` / `load_session_cache` 改為讀寫 `sessions_cache` 資料表。
- `ScanCache`（記憶體）中的 `session_mtimes` 與 `last_cursor` 在啟動時從 SQLite 恢復，不再每次冷啟動全掃。
- 應用程式啟動流程：先從 `sessions_cache` 表讀取上次快取 → 背景執行增量掃描 → 完成後更新畫面。
- 刪除 `session_cache_path()` 及相關 JSON 讀寫函式；若舊的 `session_cache.json` 存在，一次性遷移後刪除。

## Capabilities

### New Capabilities

- `session-cache`：SQLite 三表架構（sessions_cache, scan_state, session_mtimes）

### Modified Capabilities

- `startup-cache`（廢棄）：原 JSON 快取機制由 `session-cache` 完全取代
- `incremental-scan`：`ScanCache` 初始化改為從 SQLite 恢復，而非每次冷掃

## Impact

- `src-tauri/src/lib.rs`：新增 `init_sessions_cache_tables()`、`load_sessions_cache_from_db()`、`save_sessions_cache_to_db()`、`load_scan_state_from_db()`、`save_scan_state_to_db()`；移除 `session_cache_path()`、`save_session_cache()`、`load_session_cache()` 函式及 `#[tauri::command] load_session_cache`；`get_sessions_internal` 啟動時從 DB 恢復快取。
- `src/App.tsx`：移除 `invoke("load_session_cache")` 呼叫（改為後端在 `get_sessions` 回傳前先回傳快取）或改為 `get_sessions` 本身即回傳快取資料，前端不需感知差異。
- 無新外部相依套件（rusqlite 已存在）。
- 向下相容：舊 `session_cache.json` 自動遷移後刪除，使用者無感。
