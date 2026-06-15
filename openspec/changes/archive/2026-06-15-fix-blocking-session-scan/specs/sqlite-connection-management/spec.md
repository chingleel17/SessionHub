## MODIFIED Requirements

### Requirement: Application-level persistent DB connection
應用程式 SHALL 在啟動時建立唯一的 SQLite 連線，並透過 Tauri managed state（`DbState`）在整個 app 生命週期內共享。一般 Tauri command 不得在函式內自行開啟新連線；惟長時間執行的背景掃描 / backfill 類 command（於 `spawn_blocking` 中執行）為例外，得於背景執行緒內以 `open_db_connection()` 另開連線，以避免跨 `await` 持有 `DbState` 的 MutexGuard 而阻塞主執行緒或其他 command。

#### Scenario: Connection initialised on startup
- **WHEN** Tauri app 執行 `run()` 建構應用程式
- **THEN** SQLite 連線被建立並以 `DbState` 形式 `.manage()` 到 Tauri builder
- **THEN** `init_db()` 被呼叫一次完成 schema 初始化（CREATE TABLE IF NOT EXISTS）
- **THEN** WAL journal mode 被啟用（`PRAGMA journal_mode=WAL`）

#### Scenario: Command acquires connection from state
- **WHEN** 一般（短時間、同步）Tauri command 需要存取 SQLite
- **THEN** 該 command 透過 `State<'_, DbState>` 參數取得連線
- **THEN** 以 `db.conn.lock().map_err(|e| format!("db lock poisoned: {e}"))` 取得 `MutexGuard`
- **THEN** 將 `&*guard` 傳遞給對應的 `_internal` helper 函式

#### Scenario: 背景掃描 command 於 spawn_blocking 內另開連線
- **WHEN** 長時間執行的 command（如 `get_sessions`、`trigger_stats_backfill`）需在背景執行緒存取 SQLite
- **THEN** 該 command 標記為 `async`，並於 `tauri::async_runtime::spawn_blocking` 閉包內以 `open_db_connection()` 取得獨立連線
- **THEN** 不在背景工作期間持有 `DbState` 的 MutexGuard，主執行緒與其他 command 不被阻塞
- **AND** 因 WAL 模式允許多讀者單寫者，獨立連線與 `DbState` 連線可並行存取
