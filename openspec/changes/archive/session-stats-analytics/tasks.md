## 1. Rust 資料模型與 SQLite schema

- [x] 1.1 在 `src-tauri/src/lib.rs` 新增 `SessionStats` struct（含 `output_tokens`, `interaction_count`, `tool_call_count`, `duration_minutes`, `models_used: Vec<String>`, `reasoning_count`, `tool_breakdown: HashMap<String,u32>`, `is_live: bool`），加 `#[derive(Serialize)]` 與 `#[serde(rename_all = "camelCase")]`
- [x] 1.2 在 `init_db` 中新增 `CREATE TABLE IF NOT EXISTS session_stats` 語句，欄位：`session_id TEXT PRIMARY KEY, events_mtime INTEGER, output_tokens INTEGER, interaction_count INTEGER, tool_call_count INTEGER, duration_minutes INTEGER, models_used TEXT, reasoning_count INTEGER, tool_breakdown TEXT`
- [x] 1.3 新增 `get_session_stats_cache(connection, session_id) -> Option<(i64, SessionStats)>` 輔助函式，從 SQLite 讀取快取並回傳 `(events_mtime, stats)`
- [x] 1.4 新增 `upsert_session_stats_cache(connection, session_id, events_mtime, stats)` 輔助函式，UPSERT 快取至 SQLite

## 2. Rust 解析邏輯

- [x] 2.1 新增 `parse_session_stats_internal(session_dir: &Path) -> Result<SessionStats, String>` 輔助函式：偵測 `inuse.*.lock` 設定 `is_live`，讀取 `events.jsonl` 逐行解析，累積各指標
- [x] 2.2 在 `parse_session_stats_internal` 中處理 `session.start` 事件：記錄 `startTime` 與 `selectedModel`
- [x] 2.3 在 `parse_session_stats_internal` 中處理 `session.model_change` 事件：累積 `modelsUsed` 集合
- [x] 2.4 在 `parse_session_stats_internal` 中處理 `user.message` 事件：只計頂層（無 `parentToolCallId`）
- [x] 2.5 在 `parse_session_stats_internal` 中處理 `tool.execution_start` 事件：只計頂層，並累積 `toolBreakdown` map
- [x] 2.6 在 `parse_session_stats_internal` 中處理 `assistant.message` 事件：累積 `outputTokens`，計 `reasoningOpaque` 不為 null 的頂層訊息為 `reasoningCount`
- [x] 2.7 計算 `durationMinutes`：(最後一個事件 timestamp - startTime) / 60，若無 startTime 則為 0
- [x] 2.8 實作 mtime 快取邏輯：取得 `events.jsonl` 的 metadata mtime → 比對快取 → 命中則直接回傳，未命中則解析並寫入快取（`is_live` 時不寫快取）

## 3. Rust Tauri Command

- [x] 3.1 新增 `#[tauri::command] fn get_session_stats(session_dir: String) -> Result<SessionStats, String>`，呼叫 `parse_session_stats_internal` 並透過 `open_db_connection` 處理快取
- [x] 3.2 在 `invoke_handler![]` 中登記 `get_session_stats`
- [x] 3.3 撰寫 Rust 單元測試：`test_parse_stats_empty_dir`（無 events.jsonl 回傳零值）、`test_parse_stats_basic`（含 user.message、tool.execution_start、assistant.message 的簡單 JSONL）、`test_parse_stats_skips_subagent_events`（subagent 事件不計入頂層統計）

## 4. 前端型別與 Query

- [x] 4.1 在 `src/types/index.ts` 新增 `SessionStats` 型別，對應 Rust struct（camelCase）
- [x] 4.2 在 `src/App.tsx` 新增 `useSessionStats(sessionDir)` 的 React Query hook 模式（或在 App.tsx 統一管理 stats query map），Query key: `["session_stats", sessionDir]`
- [x] 4.3 在 `App.tsx` 統一載入 session stats query map，並將 stats / loading 狀態透過 props 傳入 `ProjectView`

## 5. Session 卡片統計徽章

- [x] 5.1 新增 `src/components/SessionStatsBadge.tsx` 元件，接收 `stats: SessionStats | undefined` 與 `isLoading: boolean`，渲染精簡徽章列（互動次數、token 數、時長）
- [x] 5.2 在 `SessionStatsBadge` 中加入數字格式化函式：`>= 1000` 顯示 `K`，`>= 1000000` 顯示 `M`
- [x] 5.3 在 `src/App.css` 新增 `.stats-badge-row`、`.stats-badge` 樣式
- [x] 5.4 在 `SessionCard.tsx` 底部加入 `<SessionStatsBadge>` 渲染（需新增 `stats` 與 `statsLoading` props）
- [x] 5.5 更新翻譯鍵：`zh-TW.ts` 和 `en-US.ts` 新增 `stats.turns`、`stats.tokens`、`stats.duration`、`stats.noData`

## 6. Session 詳情統計面板

- [x] 6.1 新增 `src/components/SessionStatsPanel.tsx` 元件，接收 `stats: SessionStats`，顯示：工具調用分組表、模型清單、reasoning 次數、每次互動平均 token
- [x] 6.2 在 `SessionCard.tsx` 新增展開/收折狀態（`useState`），新增詳情 icon-button（使用 `ChartIcon` 或類似 SVG）
- [x] 6.3 在 `Icons.tsx` 新增 `StatsIcon`（折線圖 SVG）
- [x] 6.4 在 `src/App.css` 新增 `.stats-panel`、`.stats-panel-row`、`.stats-tool-table` 樣式
- [x] 6.5 更新翻譯鍵：`stats.detail.*` 相關鍵值（工具調用、模型、reasoning、每次互動平均）

## 7. 專案統計 Banner

- [x] 7.1 在 `ProjectView.tsx` 新增 `sessionStats: Record<string, SessionStats>` prop，在 toolbar-card 頂部渲染 `<ProjectStatsBanner>`
- [x] 7.2 新增 `src/components/ProjectStatsBanner.tsx`，彙算所有 sessions 的 stats 並顯示 banner（N sessions · X turns · Y tokens）
- [x] 7.3 在 `src/App.css` 新增 `.project-stats-banner` 樣式
- [x] 7.4 更新 `App.tsx`：將已載入的 stats map 傳入 `ProjectView`
- [x] 7.5 更新翻譯鍵：`stats.projectBanner` 等

## 8. Dashboard 統計擴充

- [x] 8.1 在 `DashboardView.tsx` 新增 `totalOutputTokens` 與 `totalInteractions` props
- [x] 8.2 在 `App.tsx` 從 stats query 結果計算彙總值並傳入 `DashboardView`
- [x] 8.3 在 Dashboard UI 加入 token 與互動次數的統計卡，格式化顯示
- [x] 8.4 更新翻譯鍵：`dashboard.stats.totalTokens`、`dashboard.stats.totalInteractions`

## 9. 驗收

- [x] 9.1 執行 `cargo test` — 所有 Rust 單元測試通過
- [x] 9.2 執行 `bun run build` — TypeScript 無型別錯誤
- [ ] 9.3 手動驗證：SessionCard 顯示正確 badge 數值（對比手算 events.jsonl）
- [ ] 9.4 手動驗證：快取機制正常（第二次呼叫不重新解析、修改 events.jsonl 後觸發重新解析）
