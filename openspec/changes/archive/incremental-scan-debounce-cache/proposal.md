## Why

每次 FS watcher 觸發或前端主動查詢，後端都會對所有 session 目錄做完整掃描（讀取全部 `workspace.yaml`）並對 OpenCode SQLite 執行無限制 `SELECT`，即使只有一個檔案異動也如此。這在 session 數量龐大時會造成明顯延遲，且 Copilot 與 OpenCode 兩個資料來源的任何變動都會觸發對方的不必要重掃。

## What Changes

- **Debounce FS watchers**：Copilot watcher 加入 800ms 防抖、OpenCode WAL watcher 加入 500ms 防抖，避免寫入爆發時連續觸發。
- **Split events**：原本統一的 `sessions-updated` 事件拆分為 `copilot-sessions-updated` 與 `opencode-sessions-updated`，讓兩個資料來源彼此獨立。
- **Per-provider in-memory cache**：Rust AppState 加入 `ScanCache`（兩個 `Mutex<Option<ProviderCache>>`），儲存上次全掃結果、目錄 mtime 索引（Copilot）與更新游標（OpenCode）。
- **Incremental scan**：watcher 觸發後僅重新讀取 mtime 有變化的 Copilot session 目錄；OpenCode 僅查詢 `time_updated > last_cursor` 的新增/更新行。
- **30-minute full-scan threshold**：距上次全掃超過 30 分鐘時，下一次觸發自動執行全掃並重置快取，防止累積漂移。
- `get_sessions` command 新增 `force_full: Option<bool>` 參數，封存/刪除等操作可強制觸發全掃。

## Capabilities

### New Capabilities

- `scan-cache`: 提供 per-provider 的記憶體快取結構（`ScanCache` / `ProviderCache`）與全掃閾值邏輯
- `incremental-scan`: Copilot 與 OpenCode 的增量掃描邏輯（mtime 比對、cursor 查詢）
- `debounce-watcher`: FS watcher 的防抖機制（per-watcher debounce thread）

### Modified Capabilities

- `file-watcher`: 原「偵測到變更即通知前端」行為改為「防抖後分別發送 provider-specific 事件」

## Impact

- `src-tauri/src/lib.rs`：`AppState` 加入 `ScanCache`；`create_sessions_watcher` / `create_opencode_watcher` 加入 debounce；`get_sessions` 改用增量邏輯；新增 `ProviderCache`、`ScanCache` struct。
- `src/App.tsx`：`listen("sessions-updated")` 替換為兩個獨立 listener（`copilot-sessions-updated`、`opencode-sessions-updated`），兩者都 invalidate 同一個 React Query key `["sessions"]`，後端整合邏輯不外露至前端。
- 無新外部相依套件（debounce 以標準 thread + sleep 實作）。
- 無資料庫 schema 變更（快取為 runtime 記憶體，不持久化）。
- 無前端 UI 變更（使用者可見行為不變，僅回應速度提升）。
