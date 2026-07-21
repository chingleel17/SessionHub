## Why

SessionHub 在三個場景有明顯的使用體驗問題：OpenSpec task checkbox toggle 後畫面閃爍、每次冷啟動需等待全量掃描（最多 2 秒空白畫面）、以及當 session 數量多或事件頻繁時前端卡頓甚至白屏。這些問題影響日常使用流暢度，且根本原因已明確可修復。

## What Changes

- **移除 task toggle 後的雙重 refetch**：`handleToggleTask` 成功後不再呼叫 `onRefresh()`；加入 `selfWrittenFilesRef` 讓 watcher 觸發的 `refreshToken` effect 能跳過「自己剛寫入」的檔案，消除 optimistic update 被覆蓋造成的閃爍
- **新增快速啟動快取查詢**：後端新增 `get_sessions_cached` Tauri command，只讀 SQLite `sessions_cache` 表不觸發 fs 掃描；前端透過 React Query `placeholderData` 讓 sessions 列表在 ~100ms 內顯示上次的資料，背景繼續完整掃描後無縫替換
- **Session stats 改為批量查詢**：後端新增 `get_all_session_stats` command，一次 IPC 回傳所有 sessions 的 stats；前端將原本 `useQueries` 的 N 個 IPC 呼叫改為單一 `useQuery`
- **Bridge 事件前端節流**：`provider-bridge-event-logged` listener 加入 200ms throttle buffer，避免高頻事件觸發連續 setState 阻塞 JS 主執行緒
- **Backfill invalidate 精準化**：stats backfill 完成後只 invalidate `session_stats_all` 單一查詢，而非廣播觸發 N 個個別查詢

## Capabilities

### New Capabilities

- `sessions-cached-query`: 提供冷啟動快速回傳快取 session 列表的能力，不阻塞等待全量掃描
- `batch-session-stats`: 以單一 IPC 批量取得所有 sessions 統計資料的能力

### Modified Capabilities

（無 spec 層級的行為變更，以上皆為實作層最佳化）

## Impact

- `src/components/PlansSpecsView.tsx`：`handleToggleTask` 邏輯、refreshToken effect
- `src/App.tsx`：新增 `sessionsCachedQuery`、替換 `sessionStatsQueries`、bridge event listener、backfill invalidate
- `src-tauri/src/commands/sessions.rs`：新增兩個 command function
- `src-tauri/src/lib.rs`：invoke_handler 加入兩個新 command
