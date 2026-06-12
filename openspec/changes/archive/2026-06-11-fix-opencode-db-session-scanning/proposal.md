## Why

SessionHub 已能收到 OpenCode plugin 發出的 bridge 事件，但最新 OpenCode session 並未正確出現在 SessionHub 的 session 列表中。實際檢查發現，OpenCode 已將新 session 主資料寫入 `opencode.db`，而 SessionHub 仍以舊的 `storage/project/*.json` 與 `storage/session/<projectId>/*.json` 為主要掃描來源，導致 refresh 後仍掃不到最新 session。

## What Changes

- 將 OpenCode session 掃描主路徑改為讀取 `opencode.db` 的 `session`、`project` 與相關資料表，而不是只依賴舊 JSON storage。
- 更新 OpenCode fallback watcher，使其以 `opencode.db` / WAL 變化作為主要 fallback 監看來源。
- 檢查並修正 OpenCode session stats / message 解析邏輯，確保與最新 DB 儲存模型相容。
- 補上測試案例，驗證 bridge 通知後，DB-only 的 OpenCode session 能正確進入 SessionHub 列表與快取。

## Capabilities

### New Capabilities
- `opencode-db-storage`: 定義 SessionHub 如何從 OpenCode 的 SQLite storage 讀取 session 與相關 metadata。

### Modified Capabilities
- `opencode-provider`: 將 OpenCode session 掃描來源由舊 JSON storage 擴充為以 `opencode.db` 為主。
- `file-watcher`: 更新 OpenCode fallback watcher 要求，改以資料庫與 WAL 變更為主要來源。
- `session-stats`: 檢查 OpenCode stats 來源與最新 storage 模型的相容性。

## Impact

- Rust backend: `src-tauri/src/sessions/opencode.rs`、`src-tauri/src/watcher.rs`、`src-tauri/src/stats.rs` 與相關型別、測試。
- Local compatibility: 需要同時考慮舊 JSON storage 與新版 DB storage 的過渡相容。
- OpenSpec: 新增 `opencode-db-storage` capability，並更新 OpenCode provider、watcher、stats 規格。
