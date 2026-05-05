## ADDED Requirements

### Requirement: Application-level persistent DB connection
應用程式 SHALL 在啟動時建立唯一的 SQLite 連線，並透過 Tauri managed state（`DbState`）在整個 app 生命週期內共享，不得在 Tauri command 函式內自行開啟新連線。

#### Scenario: Connection initialised on startup
- **WHEN** Tauri app 執行 `run()` 建構應用程式
- **THEN** SQLite 連線被建立並以 `DbState` 形式 `.manage()` 到 Tauri builder
- **THEN** `init_db()` 被呼叫一次完成 schema 初始化（CREATE TABLE IF NOT EXISTS）
- **THEN** WAL journal mode 被啟用（`PRAGMA journal_mode=WAL`）

#### Scenario: Command acquires connection from state
- **WHEN** 任意 Tauri command 需要存取 SQLite
- **THEN** 該 command 透過 `State<'_, DbState>` 參數取得連線
- **THEN** 以 `db.conn.lock().map_err(|e| format!("db lock poisoned: {e}"))` 取得 `MutexGuard`
- **THEN** 將 `&*guard` 傳遞給對應的 `_internal` helper 函式

### Requirement: Internal functions remain connection-injectable
所有 `_internal` helper 函式 SHALL 繼續接受 `&Connection` 作為參數，不依賴 Tauri state，確保單元測試可直接注入測試用連線。

#### Scenario: Unit test injects in-memory connection
- **WHEN** 單元測試呼叫 `_internal` 函式
- **THEN** 測試可傳入 `Connection::open_in_memory()` 或測試專用路徑的連線
- **THEN** 測試不需要 Tauri app handle 或 managed state

### Requirement: Schema initialised exactly once per process
每個 app process 生命週期內，`init_db()`（包含 CREATE TABLE IF NOT EXISTS 及 migration）SHALL 只在 `DbState` 建立時執行一次，不得在每個 command 執行時重複執行。

#### Scenario: No redundant schema checks on repeated commands
- **WHEN** 用戶在 app 執行期間觸發多個 DB 相關 command
- **THEN** SQLite 的 `CREATE TABLE IF NOT EXISTS` 不會在每次 command 中重複執行
- **THEN** 每次 command 的 DB 操作延遲比短連線模式低 1~4ms

### Requirement: WAL journal mode enabled
SQLite 連線 SHALL 啟用 WAL（Write-Ahead Logging）journal mode，允許並發讀取不阻塞寫入。

#### Scenario: WAL enabled at init
- **WHEN** `init_db()` 被呼叫
- **THEN** `PRAGMA journal_mode=WAL` 在所有 CREATE TABLE 語句之前執行

#### Scenario: Concurrent read and write
- **WHEN** 一個 command 正在寫入 session stats
- **THEN** 另一個 command 可以同時讀取 session_meta 而不被阻塞（WAL 允許多讀者單寫者）
