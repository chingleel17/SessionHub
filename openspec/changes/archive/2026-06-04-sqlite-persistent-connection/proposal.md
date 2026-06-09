## Why

目前每個 Tauri command 都會自行呼叫 `open_db_connection()` 建立新的 SQLite 連線，並在許多地方重複執行 `init_db()`（CREATE TABLE IF NOT EXISTS），每次造成 1~4ms 額外開銷，且 SQLite 的頁面快取（page cache）隨連線關閉而失效。將連線改為應用程式生命週期內的長連線，可消除這些重複開銷，並讓頻繁查詢受益於熱快取。

## What Changes

- 在 Tauri `AppState` 中加入 `db: Mutex<Connection>`，應用程式啟動時建立並初始化一次
- 移除所有 Tauri command 內的 `open_db_connection()` + `init_db()` 呼叫，改從 app state 取得連線
- `DB_SCHEMA_INITIALIZED` AtomicBool 不再需要，一併移除
- `open_db_connection()` 與 `open_db_connection_and_init()` 改為只在測試及啟動用途保留
- 受影響的 command 涵蓋：`get_sessions`、`get_session_stats`、`upsert_session_meta`、`delete_session_meta`、`get_analytics_data` 等所有存取 DB 的路徑

## Capabilities

### New Capabilities

- `sqlite-connection-management`：定義應用程式層級的 SQLite 長連線管理規格，包含 Tauri managed state 的初始化、跨 command 的 Mutex 存取模式，以及測試時的連線注入方式

### Modified Capabilities

<!-- 此變更為純後端實作優化，不改變任何 user-facing 規格行為 -->

## Impact

- `src-tauri/src/lib.rs`：AppState 新增 `db` 欄位；`run()` 初始化時建立連線
- `src-tauri/src/db.rs`：`open_db_connection_and_init()` 保留但僅供測試；移除 `DB_SCHEMA_INITIALIZED`
- `src-tauri/src/commands/`：所有存取 DB 的 command 改用 `State<AppState>` 取得連線
- `src-tauri/src/sessions/`：`scan_session_dir`、`scan_copilot_incremental_internal` 等接受 `&Connection` 參數，無需自行開啟連線
- 測試：維持使用 `open_db_connection()` 或 in-memory connection，確保 isolation
