# batch-session-stats Specification

## Purpose
TBD - created by archiving change fix-perf-issues. Update Purpose after archive.
## Requirements
### Requirement: 後端提供批量 session stats 查詢
系統 SHALL 提供 `get_all_session_stats` Tauri command，接受 `session_dirs: Vec<String>` 並一次回傳所有 sessions 的統計資料，回傳型別為 `HashMap<String, SessionStats>`（key 為 sessionDir）。

#### Scenario: 批量查詢多個 sessions
- **WHEN** 前端傳入 N 個 sessionDir 字串
- **THEN** 後端回傳包含 N 筆資料的 HashMap，key 為 sessionDir

#### Scenario: 部分 session 查詢失敗不影響整批
- **WHEN** 某個 sessionDir 對應的 stats 查詢失敗（如 session 不存在）
- **THEN** 該 session 不出現在回傳的 HashMap 中，其他 session 正常回傳

### Requirement: 前端改用單一批量查詢取代 N 個個別查詢
系統 SHALL 將 `sessionStatsQueries = useQueries(...)` 替換為單一 `useQuery`，以 `allSessionDirs` 為參數，避免 N 個並行 IPC 呼叫造成 Tauri IPC queue 飽和。

#### Scenario: 單次 IPC 取得所有 session stats
- **WHEN** sessions 列表載入完成
- **THEN** 前端發出 1 次 `get_all_session_stats` IPC，而非 N 次個別呼叫

#### Scenario: Live session 定期刷新
- **WHEN** 批量 stats 中有任一 `isLive: true` 的 session
- **THEN** 整批查詢每 30 秒自動重新 fetch 一次

### Requirement: Bridge event 前端節流
系統 SHALL 對 `provider-bridge-event-logged` 事件的前端處理加入 200ms throttle，避免高頻事件連續觸發 React setState。

#### Scenario: 高頻 bridge 事件批次處理
- **WHEN** 200ms 內收到多個 `provider-bridge-event-logged` 事件
- **THEN** 前端合併為一次 setState，只重渲染一次

#### Scenario: 低頻事件不受影響
- **WHEN** 兩個 bridge 事件間隔 > 200ms
- **THEN** 每個事件都正常觸發 setState 更新

### Requirement: Backfill 後精準 invalidate
系統 SHALL 在 stats backfill 完成後只 invalidate `session_stats_all` 單一批量查詢，不廣播 invalidate N 個個別查詢。

#### Scenario: Backfill 完成後觸發單一批量刷新
- **WHEN** `triggerStatsBackfill` 回傳 count > 0
- **THEN** 只 invalidate `queryKey: ["session_stats_all"]`，觸發一次批量查詢

