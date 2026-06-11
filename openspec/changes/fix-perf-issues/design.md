## Context

SessionHub 前端使用 React Query 管理所有伺服器狀態，後端 Tauri 以 Rust 處理 IPC 命令。目前三個效能瓶頸：

1. **PlansSpecsView 的 task toggle** 在 `handleToggleTask` 成功後手動呼叫 `onRefresh()`（觸發 plans/specs 重新掃描），同時 watcher 偵測到磁碟變更後也會 emit `project-files-changed` 再觸發一次，導致雙重 refetch 覆蓋 optimistic update，畫面出現閃爍。
2. **冷啟動** 時 React Query 沒有任何持久化層，必須等 `get_settings` → `get_sessions`（含全量 fs 掃描）串行完成後 UI 才能顯示，期間畫面完全空白（最長 2 秒）。後端 SQLite 已有 `sessions_cache` 表但前端未利用。
3. **Session stats** 採 N 個並行 `useQueries`，每個 session 各自發一個 IPC；加上 stats backfill 後廣播 invalidate 全部查詢、bridge 事件無前端節流，在 session 數量多時造成 Tauri IPC queue 飽和與 React 連續 setState 阻塞 JS 主執行緒。

## Goals / Non-Goals

**Goals:**
- 消除 task checkbox toggle 的閃爍（不修改 Rust watcher 邏輯）
- 冷啟動後 ≤100ms 顯示上次 session 列表，完整掃描後無縫替換
- 將 session stats IPC 呼叫從 N 次降為 1 次
- bridge event 前端每秒渲染次數 ≤ 5 次（200ms throttle）

**Non-Goals:**
- 修改 watcher debounce 常數或 Rust 端 debounce 邏輯
- 前端快取持久化到 localStorage（React Query placeholderData 已足夠）
- 修改 OpenCode stats 的讀取方式
- 任何 UI 外觀變更

## Decisions

### 決策一：task toggle 閃爍用純前端 `selfWrittenFilesRef` 解決，不修改 Rust

**選擇**：在 `PlansSpecsView.tsx` 加入 `useRef<Set<string>>`，toggle 寫入成功後 add(filePath)；監聽 `refreshToken` 的 useEffect 若命中則 delete 後跳過重新讀取。同時移除手動 `void onRefresh()` 呼叫。

**替代方案**：在 Rust watcher 端加入 suppress 視窗（watcher 忽略自己發出寫入後 N ms 的事件）。但需要從 `write_openspec_file` command 更新 `WatcherState`，跨 Tauri command 傳遞狀態複雜度高，且存在 race condition 風險。

**理由**：watcher 有 400ms debounce，`onWriteOpenspecFile` Promise 在 debounce 結束前已 resolve，標記時機安全。純前端改動範圍小、可測試、不影響 Rust 端。

---

### 決策二：冷啟動用 `placeholderData` + 新 `get_sessions_cached` command

**選擇**：後端新增 `get_sessions_cached`（只讀 `sessions_cache` SQLite 表，不觸發掃描），前端新增 `sessionsCachedQuery`，將其作為原 `sessionsQuery` 的 `placeholderData`。

**替代方案 A**：使用 React Query 的 `persister` plugin（如 `createSyncStoragePersister`）持久化全部 QueryClient 到 localStorage。缺點：序列化大量 SessionInfo 資料到 localStorage 效能差，且與後端 SQLite 快取重複。

**替代方案 B**：修改 `get_sessions` 讓後端「快取先回、掃描後推」（streaming 回傳兩次）。缺點：Tauri IPC 無原生 streaming，需改用 event 機制，架構改動較大。

**理由**：`placeholderData` 是 React Query 的標準用法，零學習成本，下游所有使用 `sessionsQuery.data` 的地方不需修改。後端新增 command 只需讀 SQLite，實作簡單。

---

### 決策三：session stats 改單一批量 command `get_all_session_stats`

**選擇**：後端新增 `get_all_session_stats(session_dirs: Vec<String>)` 回傳 `HashMap<String, SessionStats>`；前端用 `useMemo` 計算 `allSessionDirs` 並以單一 `useQuery` 取代原 `useQueries` 迴圈。

**替代方案**：分批查詢（每次 50 個），限制並行度。缺點：仍有多次 IPC，實作複雜度高於單一 batch。

**理由**：SQLite 快取命中率高（非 live session 全走快取），500 個 session 的批量查詢預計 < 50ms，遠快於 500 個並行 IPC 的隊列等待時間。

---

### 決策四：bridge event 用 `useRef` buffer + setTimeout throttle

**選擇**：在 `listen("provider-bridge-event-logged")` callback 中，將事件推入 `bridgeEventBufferRef`，首次推入時 setTimeout 200ms 後批次 flush 並 setState 一次。

**替代方案**：引入第三方 throttle 函式庫（如 lodash throttle）。缺點：增加依賴。

**理由**：用 `useRef` + `setTimeout` 可精確控制批次行為，不引入額外依賴，符合專案現有風格。

## Risks / Trade-offs

- **selfWrittenFilesRef 競態**：若在極端情況下 watcher 事件在 400ms debounce 結束前且 Promise resolve 後、Set.add 前到達（理論上不可能，因 debounce 保護），則閃爍仍會發生一次。此風險可接受。
- **get_sessions_cached 顯示舊資料**：冷啟動後短暫顯示上次快取資料，若兩次啟動間 session 有增減，用戶會短暫看到過時列表。完整掃描完成後無縫替換，影響期間 ≤ 2 秒。此為合理取捨（空白畫面更差）。
- **allSessionDirs queryKey 穩定性**：`allSessionDirs.sort()` 確保同一組 sessions 產生相同 queryKey，但若 sessions 增減（常見場景），queryKey 變化 → 重新 fetch，為正確行為。
- **get_all_session_stats payload 大小**：500 個 sessionDir 字串序列化約 50KB，Tauri IPC 可承受，但若 session 數量極大（> 5000）需考慮分批。

## Migration Plan

無資料遷移需求。新增的兩個 Rust command 不修改現有資料結構，前端改動完全向下相容。直接發布即可。
