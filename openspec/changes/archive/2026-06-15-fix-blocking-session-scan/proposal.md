## Why

`get_sessions` 原本是同步 `#[tauri::command]`。在 Tauri v2 中，未標記 `async` 的 command 會在主執行緒（IPC / 事件循環）上執行，而 `get_sessions_internal` 會進行大量耗時的同步工作（多 provider 磁碟掃描、逐一執行 git 子程序 `enrich_sessions_with_git_metadata`）。只要主執行緒被佔住，視窗就無法重繪與回應點擊，整個 UI 變白卡死。手動掃描與自動掃描走同一條路徑，因此兩者都會卡，與 hook 無關。

修正掃描阻塞後，又暴露兩個被原本「同步且極快」行為掩蓋的前端問題：

1. **狀態列一直卡在「掃描中」**：`sessionsQuery` 把 `forceFull` 放進 `queryKey`，又在 `queryFn` 完成時 `setForceFull(false)`，導致每次 fetch 完成就因 queryKey 變動再觸發第二次 fetch。掃描變慢（async 化）後，雙重 fetch 疊加 watcher 的 invalidate，使 `isFetching` 永遠為 `true`，狀態列卡在「掃描中」。
2. **狀態列計數範圍與看板不一致**：狀態列的 idle/active/waiting/done 計數原本以「全部 session」為基礎，顯示歷史所有 session（例如 idle 800 多），但看板僅顯示當前週期（本周 / 本月）的 session，造成數字嚴重落差。

這次改動已直接修復程式碼，但未透過 OpenSpec 流程，需補記入規格，避免後續重構依現有規格把非阻塞掃描改回同步、或把 `forceFull` 改回 queryKey，導致卡住問題重現。

## What Changes

- `get_sessions` 改為 `async` command，整個掃描透過 `tauri::async_runtime::spawn_blocking` 移至背景執行緒，不阻塞主執行緒。
- `ScanCache` 改以 `Arc<ScanCache>` 形式 `.manage()`，使其能 clone 進 `spawn_blocking` 閉包。
- 背景掃描執行緒內以 `open_db_connection()` 另開 DB 連線（與既有的 `trigger_stats_backfill` 一致），不持有 `DbState` 的 MutexGuard 跨 `await`。
- 前端 `forceFull` 由 `useState` 改為 `useRef`，移出 `sessionsQuery` 的 `queryKey`；`queryFn` 讀取後立即清除，掃描完成不再改變 queryKey。
- 狀態列 active/waiting/idle/done 計數改以「當前看板週期內的 session（`filteredDashboardSessions`）」為計算範圍，與看板顯示一致。

## Capabilities

### Modified Capabilities
- `sessions-cached-query`: `get_sessions` 掃描須非阻塞執行；`forceFull` 不得進入 `queryKey`。
- `sqlite-connection-management`: 補上「長時間背景掃描 command 可於 `spawn_blocking` 內另開連線」的例外。
- `unified-session-status-count`: 狀態列計數範圍限定於當前看板週期。

## Impact

- **Backend**: `src-tauri/src/commands/sessions.rs`（`get_sessions` 改 async + spawn_blocking、`get_session_activity_statuses` state 型別對齊）、`src-tauri/src/lib.rs`（`ScanCache` 以 `Arc` manage）。
- **Frontend**: `src/App.tsx`（`forceFull` 改 ref、狀態列計數範圍對齊 `filteredDashboardSessions`）。
- **行為**: 掃描期間 UI 保持回應；狀態列「掃描中」會在掃描完成後正常結束；狀態列計數與看板週期一致。
