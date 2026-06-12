## 1. Task Toggle 閃爍修復（PlansSpecsView.tsx）

- [x] 1.1 在 `PlansSpecsView.tsx` 頂部加入 `selfWrittenFilesRef = useRef<Set<string>>(new Set())`
- [x] 1.2 在 `handleToggleTask` 成功後移除 `void onRefresh()` 呼叫，改為 `selfWrittenFilesRef.current.add(filePath)`
- [x] 1.3 在監聽 `refreshToken` 的 `useEffect` 中加入跳過邏輯：若 `contentFilePath` 在 `selfWrittenFilesRef` 中則 `delete` 後 `return`

## 2. 後端新增快取查詢 Command

- [x] 2.1 在 `src-tauri/src/commands/sessions.rs` 新增 `get_sessions_cached` function，從 `sessions_cache` SQLite 表讀取資料，支援 `enabled_providers` 和 `show_archived` 參數
- [x] 2.2 在 `src-tauri/src/lib.rs` 的 `invoke_handler` 加入 `get_sessions_cached`

## 3. 前端冷啟動快取查詢

- [x] 3.1 在 `src/App.tsx` 新增 `sessionsCachedQuery`（`queryKey: ["sessions_cached", ...]`，`staleTime: Infinity`），settings 到位後立即執行
- [x] 3.2 在原 `sessionsQuery` 加入 `placeholderData: sessionsCachedQuery.data`

## 4. 後端新增批量 Stats Command

- [x] 4.1 在 `src-tauri/src/commands/sessions.rs` 新增 `get_all_session_stats` function，接受 `session_dirs: Vec<String>`，回傳 `HashMap<String, SessionStats>`
- [x] 4.2 在 `src-tauri/src/lib.rs` 的 `invoke_handler` 加入 `get_all_session_stats`

## 5. 前端 Session Stats 改批量查詢

- [x] 5.1 在 `src/App.tsx` 加入 `allSessionDirs = useMemo(...)` 計算排序後的 sessionDir 列表（排除 codex）
- [x] 5.2 將 `sessionStatsQueries = useQueries(...)` 替換為單一 `sessionStatsQuery = useQuery({ queryKey: ["session_stats_all", allSessionDirs], ... })`
- [x] 5.3 新增 `sessionStatsMap = useMemo(...)` 將 `HashMap<sessionDir, stats>` 轉換為 `Record<sessionId, stats>`，供下游使用
- [x] 5.4 將 backfill 後的 `invalidateQueries({ queryKey: ["session_stats"] })` 改為 `invalidateQueries({ queryKey: ["session_stats_all"] })`
- [x] 5.5 確認所有原本使用 `sessionStatsMap[session.id]` 的地方仍正常運作

## 6. Bridge Event 前端節流

- [x] 6.1 在 `src/App.tsx` 的 bridge event listener 區塊加入 `bridgeEventBufferRef` 和 `bridgeEventFlushTimerRef`
- [x] 6.2 將 `listen("provider-bridge-event-logged")` callback 改為 buffer + 200ms setTimeout 批次 flush 模式

## 7. 驗證

- [x] 7.1 執行 `cargo build` 確認 Rust 端編譯無誤
- [x] 7.2 執行 `bun run dev` 啟動開發環境，手動點擊 task checkbox 確認無閃爍
- [x] 7.3 關閉重開應用，確認 session 列表在 100ms 內出現（不再空白等待）
- [x] 7.4 開啟 DevTools Console，確認 `get_all_session_stats` 只呼叫一次而非 N 次
