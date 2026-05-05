## 1. 基礎建設

- [x] 1.1 `types.rs`：新增 `DbState` struct，包含 `pub(crate) conn: Mutex<Connection>` 欄位，並實作 `DbState::new(db_path) -> Result<Self, String>`（開啟連線、執行 `init_db`）
- [x] 1.2 `db.rs`：在 `init_db()` 最前面加入 `PRAGMA journal_mode=WAL` 執行
- [x] 1.3 `db.rs`：將 `DB_SCHEMA_INITIALIZED` AtomicBool 及 `open_db_connection_and_init()` 移除；`open_db_connection()` 加上文件說明「僅供測試或特殊啟動用途」
- [x] 1.4 `lib.rs`（`run()`）：在 `.manage(ScanCache::default())` 旁邊加入 `DbState::new()?` 初始化並 `.manage(db_state)`；import 相關型別

## 2. commands/ 模組遷移

- [x] 2.1 `commands/sessions.rs`：`get_session_stats_command`、`upsert_session_meta`、`delete_session_meta` 改用 `State<'_, DbState>`，移除內部 `open_db_connection()` + `init_db()` 呼叫
- [x] 2.2 `commands/analytics.rs`：`get_analytics_data` 改用 `State<'_, DbState>`，移除內部 `open_db_connection()` + `init_db()` 呼叫

## 3. sessions/ 模組遷移

- [x] 3.1 `sessions/mod.rs`（`get_sessions_internal`）：改從外部接收 `&Connection` 參數，移除內部 `open_db_connection_and_init()`；更新 `get_sessions` command 傳入 `State<'_, DbState>`
- [x] 3.2 `sessions/copilot.rs`（`scan_session_dir`、`find_session_by_cwd_internal`、刪除空 session 相關函式）：移除內部 `open_db_connection()` + `init_db()`，改接受 `&Connection` 參數或從呼叫端傳入

## 4. lib.rs command 遷移

- [x] 4.1 遷移 lib.rs 中存取 DB 的所有 Tauri command（`get_session_stats`、`batch_get_session_stats`、`delete_empty_sessions`、`get_session_activity_statuses` 等）改用 `State<'_, DbState>`
- [x] 4.2 移除 lib.rs 中所有零散的 `open_db_connection()` + `init_db()` 呼叫，確認無遺漏

## 5. 驗證

- [x] 5.1 執行 `cargo test`，確認全部 50 個測試通過（測試仍使用 `open_db_connection()` 或 in-memory connection）
- [x] 5.2 執行 `bun run build`，確認前端 TypeScript 無錯誤
- [ ] 5.3 手動驗證：啟動 app，瀏覽 sessions、編輯備註、查看 stats，確認功能正常
