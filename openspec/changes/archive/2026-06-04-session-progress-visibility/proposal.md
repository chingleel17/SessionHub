## Why

Copilot CLI 近期更新後，agent 在執行任務時改用每個 session 的 `session.db` 追蹤 todos，不再預設建立 `plan.md`；同時，SessionHub 的統計儀表板對大量已完成 session 顯示空白，因為 `session_stats` 只在使用者手動點開 session 時才被計算，造成 dashboard 數字嚴重失真。

## What Changes

- **新增**：在 session 詳細頁面顯示 agent 的 todos 清單（從 `session.db` 的 `todos` 表讀取），讓使用者追蹤 agent 任務進度
- **新增**：將 Plan 與任務入口整合到 session 卡片的摘要 badge 區，點擊 Plan 直接開編輯分頁、點擊任務摘要開啟任務分頁
- **新增**：在 session 卡片摘要 badge 區顯示任務總數與各狀態數量（僅在 count > 0 時顯示）
- **新增**：SessionHub 啟動及 file watcher 觸發時，自動對所有「已完成但缺少 `session_stats`」的 session 進行背景補算，不需使用者手動開啟每個 session

## Capabilities

### New Capabilities

- `session-todos-viewer`: 在 Copilot session 的詳細頁面中讀取並顯示 `session.db` 的 todos 表，呈現 agent 的任務清單（id、title、status、description），提供進度可見性
- `stats-background-backfill`: 背景掃描找出 `sessions_cache` 中存在但 `session_stats` 缺失的已完成 session（有 `events.jsonl` + `session.shutdown`），自動解析並寫入 `session_stats`

### Modified Capabilities

- `session-todos-viewer`: 補充統一入口 badge、任務摘要 badge 與任務子分頁的互動規格
- `session-stats`: 補充 backfill trigger 時機的規格（app 啟動時、file watcher 偵測到 session 目錄更新時）

## Impact

- **Rust 後端**：新增 `read_session_todos` command（讀取 `session.db`）；新增 `backfill_missing_session_stats` command 或在現有掃描流程中加入 backfill 邏輯
- **前端**：Session 卡片摘要 badge 區新增 Plan / 任務快捷入口、任務摘要 badge，以及 ProjectView 任務子分頁
- **SQLite**：只讀取 `session.db` 的 `todos` 表，不修改；`metadata.db` 的 `session_stats` 寫入行為不變，僅增加觸發時機
- **無 breaking changes**：現有 plan.md viewer 邏輯保持不變
