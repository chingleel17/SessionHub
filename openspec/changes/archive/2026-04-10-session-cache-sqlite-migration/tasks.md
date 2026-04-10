## 1. SQLite Schema 新增

- [x] 1.1 在 `init_db` fn 中新增 `sessions_cache` 資料表（欄位：session_id, provider, cwd, summary, summary_count, created_at, updated_at, session_dir, parse_error, is_archived, has_plan, has_events；PK: session_id, provider）
- [x] 1.2 在 `init_db` fn 中新增 `scan_state` 資料表（provider TEXT PK, last_full_scan_at INTEGER, last_cursor INTEGER）
- [x] 1.3 在 `init_db` fn 中新增 `session_mtimes` 資料表（session_id TEXT PK, provider TEXT, mtime INTEGER）

## 2. sessions_cache 讀寫函式

- [x] 2.1 新增 `load_sessions_cache_from_db(connection: &Connection, provider: Option<&str>) -> Vec<SessionInfo>` fn：從 `sessions_cache` 讀取並映射為 `Vec<SessionInfo>`
- [x] 2.2 新增 `save_sessions_cache_to_db(connection: &Connection, sessions: &[SessionInfo]) -> Result<(), String>` fn：以 `INSERT OR REPLACE` 將 sessions 批次寫入 `sessions_cache`
- [x] 2.3 在 `save_sessions_cache_to_db` 中，先刪除對應 provider 的舊記錄，再批次插入新記錄（避免孤兒記錄）

## 3. scan_state 讀寫函式

- [x] 3.1 新增 `load_scan_state_from_db(connection: &Connection, provider: &str) -> (i64, i64)` fn：回傳 `(last_full_scan_at, last_cursor)`；若無記錄回傳 `(0, 0)`
- [x] 3.2 新增 `save_scan_state_to_db(connection: &Connection, provider: &str, last_full_scan_at: i64, last_cursor: i64) -> Result<(), String>` fn

## 4. session_mtimes 讀寫函式

- [x] 4.1 新增 `load_session_mtimes_from_db(connection: &Connection, provider: &str) -> HashMap<String, i64>` fn
- [x] 4.2 新增 `save_session_mtimes_to_db(connection: &Connection, provider: &str, mtimes: &HashMap<String, i64>) -> Result<(), String>` fn：以 `INSERT OR REPLACE` 批次寫入

## 5. ScanCache 啟動恢復

- [x] 5.1 修改 `get_sessions_internal`：在呼叫 `should_full_scan` 之前，若 `copilot_guard` 為 `None`，從 `scan_state` 與 `session_mtimes` 恢復 `ProviderCache`（`sessions` 欄位填入 `load_sessions_cache_from_db` 的結果）
- [x] 5.2 同上處理 `opencode_guard`
- [x] 5.3 `last_full_scan_at` 從 DB 讀取的 unix timestamp 轉換為等效的 `Instant`（以 `Instant::now() - Duration::from_secs(elapsed)` 方式）

## 6. 掃描完成後持久化

- [x] 6.1 在 `get_sessions_internal` 全量掃描完成後，呼叫 `save_sessions_cache_to_db`、`save_scan_state_to_db`、`save_session_mtimes_to_db` 寫入 DB
- [x] 6.2 在增量掃描完成後，同樣呼叫以上三個函式更新 DB（僅更新有變化的 provider）
- [x] 6.3 所有持久化失敗只記錄 `eprintln!` 警告，不中斷主流程

## 7. 移除舊 JSON 快取機制

- [x] 7.1 移除 `session_cache_path()` fn
- [x] 7.2 移除 `save_session_cache(sessions: &[SessionInfo]) -> Result<(), String>` fn
- [x] 7.3 移除 `#[tauri::command] fn load_session_cache() -> Vec<SessionInfo>` command
- [x] 7.4 從 `run()` 的 `invoke_handler![]` 中移除 `load_session_cache`
- [x] 7.5 移除 `get_sessions_internal` 結尾的 `save_session_cache` 呼叫，替換為步驟 6 的新持久化呼叫

## 8. 舊 JSON 一次性遷移

- [x] 8.1 在 `init_db` 或獨立的 `migrate_legacy_json_cache` fn 中，若 `session_cache.json` 存在，讀取並呼叫 `save_sessions_cache_to_db`
- [x] 8.2 遷移成功後刪除 `session_cache.json`
- [x] 8.3 遷移失敗（讀取或解析錯誤）時僅記錄警告，不影響啟動流程

## 9. 前端調整

- [x] 9.1 在 `src/App.tsx` 中移除 `invoke("load_session_cache")` 的呼叫及相關初始化邏輯
- [x] 9.2 確認 `get_sessions` 呼叫在應用啟動時正常觸發（已有邏輯，確認即可）

## 10. 測試

- [x] 10.1 新增 Rust unit test：`test_sessions_cache_roundtrip`（寫入 → 讀取 → 比對欄位）
- [x] 10.2 新增 Rust unit test：`test_scan_state_roundtrip`（寫入 cursor/timestamp → 讀取比對）
- [x] 10.3 新增 Rust unit test：`test_session_mtimes_roundtrip`（HashMap 寫入 → 讀取比對）
- [x] 10.4 在現有 `test_get_sessions` 系列測試中驗證：啟動後 ScanCache 從 DB 恢復，增量掃描不執行全掃
