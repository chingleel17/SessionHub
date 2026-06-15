## 1. 後端：掃描改為非阻塞背景執行

- [x] 1.1 `lib.rs` 將 `ScanCache::default()` 改以 `std::sync::Arc::new(...)` 形式 `.manage()`
- [x] 1.2 `get_sessions` 改為 `async fn`，state 型別改為 `State<'_, Arc<ScanCache>>`
- [x] 1.3 在 `get_sessions` 內 `Arc::clone(scan_cache.inner())` 後以 `tauri::async_runtime::spawn_blocking` 執行 `get_sessions_internal`
- [x] 1.4 背景閉包內以 `open_db_connection()` 另開 DB 連線，不沿用 `DbState` guard
- [x] 1.5 `get_session_activity_statuses` 的 `scan_cache` 參數型別對齊為 `State<'_, Arc<ScanCache>>`
- [x] 1.6 `cargo check` 通過（僅既有無關警告）

## 2. 前端：修復「卡在掃描中」

- [x] 2.1 `forceFull` 由 `useState` 改為 `useRef<boolean>`（`forceFullRef`）
- [x] 2.2 自 `sessionsQuery` 的 `queryKey` 移除 `forceFull`
- [x] 2.3 `queryFn` 讀取 `forceFullRef.current` 後立即重設為 `false`，不在 fetch 過程中變動 queryKey
- [x] 2.4 所有 `setForceFull(true)` 改為 `forceFullRef.current = true`（其後既有的 `invalidateQueries(["sessions"])` 負責觸發掃描）

## 3. 前端：狀態列計數範圍對齊看板週期

- [x] 3.1 將計數 useMemo 移至 `filteredDashboardSessions` 定義之後
- [x] 3.2 計數迴圈改以 `filteredDashboardSessions` 為來源，依賴陣列改為 `[filteredDashboardSessions, activityStatusMap]`

## 4. 驗證

- [x] 4.1 `tsc --noEmit` 型別檢查通過
- [x] 4.2 手動重新掃描期間 UI 保持可操作，不再白屏
- [x] 4.3 掃描完成後狀態列「掃描中」正常結束，不再卡住
- [x] 4.4 Session 卡片更新時間正常顯示
- [x] 4.5 狀態列計數與看板當前週期一致
