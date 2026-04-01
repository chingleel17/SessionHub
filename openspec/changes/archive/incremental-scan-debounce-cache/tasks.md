## 1. Rust 資料結構

- [x] 1.1 在 `lib.rs` 頂部新增 `ProviderCache` struct（`sessions`, `session_mtimes`, `last_full_scan_at`, `last_cursor`）
- [x] 1.2 新增 `ScanCache` struct（`copilot: Mutex<Option<ProviderCache>>`, `opencode: Mutex<Option<ProviderCache>>`）
- [x] 1.3 在 `AppState` struct 加入 `scan_cache: ScanCache` 欄位
- [x] 1.4 在 `run()` 中初始化 `ScanCache`（兩個 provider 均為 `Mutex::new(None)`）

## 2. Debounce Watcher — Copilot

- [x] 2.1 定義常數 `COPILOT_DEBOUNCE_MS: u64 = 800`
- [x] 2.2 在 `create_sessions_watcher` 中加入 `Arc<Mutex<Instant>>` 作為 `last_event` 追蹤器
- [x] 2.3 將 watcher callback 改為：更新 `last_event` 後 spawn thread，sleep 800ms，比對 elapsed，穩定後觸發掃描
- [x] 2.4 將 emit 事件由 `sessions-updated` 改為 `copilot-sessions-updated`

## 3. Debounce Watcher — OpenCode

- [x] 3.1 定義常數 `OPENCODE_DEBOUNCE_MS: u64 = 500`
- [x] 3.2 在 `create_opencode_watcher` 中加入 `Arc<Mutex<Instant>>` 作為 `last_event` 追蹤器
- [x] 3.3 將 watcher callback 改為：更新 `last_event` 後 spawn thread，sleep 500ms，比對 elapsed，穩定後觸發掃描
- [x] 3.4 將 emit 事件由 `sessions-updated` 改為 `opencode-sessions-updated`

## 4. Copilot 增量掃描邏輯

- [x] 4.1 新增 `scan_copilot_incremental_internal` fn：`read_dir` 取得所有 session 目錄的 mtime
- [x] 4.2 比對 `cache.session_mtimes`：找出 mtime 變化/新增的 session_id 列表
- [x] 4.3 只對變化的 session 呼叫現有的 `parse_workspace_yaml`（或等效讀取函式）
- [x] 4.4 從快取結果中移除已消失的 session 目錄
- [x] 4.5 更新 `cache.session_mtimes` 與 `cache.sessions`，回傳合併後的完整 `Vec<SessionInfo>`

## 5. OpenCode 增量掃描邏輯

- [x] 5.1 新增 `scan_opencode_incremental_internal` fn：執行 `SELECT * FROM sessions WHERE time_updated > {last_cursor}`
- [x] 5.2 將查詢結果 upsert 進 `cache.sessions`（by session_id）
- [x] 5.3 更新 `cache.last_cursor` 為新結果中最大的 `time_updated`（若無結果則不變）
- [x] 5.4 回傳更新後的 `cache.sessions`

## 6. get_sessions Command 整合

- [x] 6.1 為 `get_sessions` command 加入 `force_full: Option<bool>` 參數
- [x] 6.2 新增 `should_full_scan(cache: &Option<ProviderCache>, force_full: bool) -> bool` 輔助 fn（檢查 None / 30min threshold / force_full）
- [x] 6.3 定義常數 `FULL_SCAN_THRESHOLD_SECS: u64 = 1800`
- [x] 6.4 在 `get_sessions_internal` 中：lock `scan_cache.copilot`，依 `should_full_scan` 決定全掃或增量，更新快取
- [x] 6.5 同上處理 `scan_cache.opencode`
- [x] 6.6 合併兩個 provider 快取的 `sessions` 並回傳統一的 `Vec<SessionInfo>`

## 7. 強制全掃呼叫點

- [x] 7.1 找出封存 session 的 command（`archive_session` 或等效），在其後呼叫 `get_sessions` 時確保傳入 `force_full: true`
- [x] 7.2 找出刪除 session 的 command，同上處理

## 8. 前端 App.tsx 調整

- [x] 8.1 將 `listen("sessions-updated", ...)` 改為 `listen("copilot-sessions-updated", ...)`，callback 不變（invalidate `["sessions"]` query）
- [x] 8.2 新增 `listen("opencode-sessions-updated", ...)`，callback 同上
- [x] 8.3 確認兩個 listener 都在 `useEffect` cleanup 中 `unlisten`
- [x] 8.4 確認前端呼叫 `get_sessions` 時，封存/刪除後的呼叫傳入 `{ forceFullScan: true }`（camelCase，Tauri serde 自動對應 `force_full`）

## 9. 驗證

- [x] 9.1 執行 `cargo test`，確認所有現有 Rust 單元測試通過（8/8 通過）
- [x] 9.2 執行 `bun run build`，確認 TypeScript 無型別錯誤（build 成功）
- [ ] 9.3 手動測試：啟動 app，確認 session 列表正常載入（全掃路徑）
- [ ] 9.4 手動測試：在 Copilot session 目錄新增檔案，確認 UI 在 ~800ms 後自動更新
- [ ] 9.5 手動測試：封存一個 session，確認 session 列表正確反映（強制全掃）
- [ ] 9.6 手動測試：等待 30 分鐘（或暫時調低閾值），確認下次觸發執行全掃
