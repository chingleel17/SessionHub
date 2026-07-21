## 1. Rust：Session Todos 讀取

- [x] 1.1 在 `src-tauri/src/types.rs` 新增 `SessionTodo` struct（id, title, status, description, updatedAt）並加 `#[serde(rename_all = "camelCase")]`
- [x] 1.2 在 `src-tauri/src/commands/` 新增 `session_todos.rs`，實作 `read_session_todos_internal(session_dir: &Path) -> Result<Vec<SessionTodo>, String>`，以 `rusqlite` 開啟 `session.db`，查詢 `todos` 表；`session.db` 不存在或 `todos` 表不存在時回傳 `Ok(vec![])`
- [x] 1.3 新增 `#[tauri::command] fn read_session_todos(session_dir: String) -> Result<Vec<SessionTodo>, String>`，呼叫 internal 函式
- [x] 1.4 在 `lib.rs` 的 `invoke_handler![]` 登記 `read_session_todos`
- [x] 1.5 在 `src-tauri/src/commands/mod.rs` 加入 `session_todos` 模組

## 2. Rust：Stats Background Backfill

- [x] 2.1 在 `src-tauri/src/stats.rs` 新增 `backfill_missing_stats_internal(conn: &Connection, copilot_root: &Path) -> Result<usize, String>`：查 `sessions_cache` 中缺少 `session_stats` 的 session，過濾 live session，取最新 50 個，依序解析 `events.jsonl` 並寫入 `session_stats`
- [x] 2.2 新增 `#[tauri::command] async fn trigger_stats_backfill(db: State<DbState>, settings: State<SettingsState>) -> Result<usize, String>`，在 `spawn_blocking` 內呼叫 internal 函式
- [x] 2.3 在 `lib.rs` 的 `invoke_handler![]` 登記 `trigger_stats_backfill`

## 3. 前端：型別與 IPC

- [x] 3.1 在 `src/types/index.ts` 新增 `SessionTodo` 型別（id, title, status, description, updatedAt）與 status 聯合型別 `"pending" | "in_progress" | "done" | "blocked"`
- [x] 3.2 在 `src/App.tsx` 新增 `readSessionTodos(sessionDir: string): Promise<SessionTodo[]>` invoke wrapper
- [x] 3.3 在 `src/App.tsx` 新增 `triggerStatsBackfill()` invoke call，於 sessions query 成功後（`onSuccess`）呼叫一次（fire-and-forget）

## 4. 前端：Todos UI 元件

- [x] 4.1 新增 `src/components/SessionTodosPanel.tsx`，接收 `todos: SessionTodo[]` props，顯示每個 todo 的 status badge 與 title；status 使用對應顏色（pending=灰、in_progress=藍、done=綠、blocked=紅）
- [x] 4.2 在 SessionView 或 ProjectView 的 session 詳細區塊中，對 `provider === "copilot"` 的 session 載入並顯示 `SessionTodosPanel`（呼叫 `readSessionTodos`）
- [x] 4.3 若 `isLive === true`，隨 stats refetch 一起重新呼叫 `readSessionTodos`（保持同步）
- [x] 4.4 在 `src/locales/zh-TW.ts` 與 `en-US.ts` 新增 todos 相關翻譯 key（「無任務資料」、status 標籤等）

## 5. 測試與驗證

- [x] 5.1 在 `src-tauri/src/commands/session_todos.rs` 撰寫單元測試：測試 `session.db` 不存在、`todos` 表不存在、正常讀取三個情境
- [x] 5.2 在 `src-tauri/src/stats.rs` 撰寫 `backfill_missing_stats_internal` 的單元測試：驗證已有快取的 session 不重複計算、live session 被跳過
- [x] 5.3 執行 `bun run build` 確認前端無型別錯誤
- [x] 5.4 執行 `cargo test` 確認所有 Rust 測試通過

## 6. 前端：Session 卡片 badge 入口整合

- [x] 6.1 將 `hasPlan` 從 session header chip 移至 `SessionStatsBadge` 區，改為可點擊的 Plan badge，點擊後開啟既有 Plan 編輯分頁
- [x] 6.2 在 `SessionStatsBadge` 區新增任務總數 badge 與各狀態 badge，僅在對應 count > 0 時顯示，並支援未知狀態名稱
- [x] 6.3 讓任務總數 badge 與狀態 badge 點擊後開啟該 session 的 todos 子分頁，不重複建立相同 session 的 todos 分頁
- [x] 6.4 調整 `ProjectView` sub-tab state，使同一 session 的 plan / todos 分頁可共存且互不衝突
- [x] 6.5 新增 todos 分頁內容元件，提供與 Plan 分頁一致的查看體驗
- [x] 6.6 更新對應翻譯與樣式，補上 badge 文案與任務分頁標題
- [x] 6.7 重新執行 `bun run build` 與 `cargo test` 驗證上述 UI 調整
