# sessions-cached-query Specification

## Purpose
TBD - created by archiving change fix-perf-issues. Update Purpose after archive.
## Requirements
### Requirement: get_sessions 掃描非阻塞執行

`get_sessions` Tauri command SHALL 為 `async`，並將整個 session 掃描（多 provider 磁碟掃描與 git metadata 查詢）透過 `tauri::async_runtime::spawn_blocking` 移至背景執行緒，不得在 Tauri 主執行緒上同步執行，以避免掃描期間 UI 白屏無回應。

#### Scenario: 掃描期間 UI 保持回應

- **WHEN** 使用者手動觸發重新掃描，或 watcher 事件觸發自動掃描
- **THEN** 掃描在背景執行緒進行，主執行緒不被阻塞
- **AND** 視窗持續可重繪、可回應點擊與操作，不出現白屏卡死

#### Scenario: 背景執行緒取得掃描所需資源

- **WHEN** `get_sessions` 進入 `spawn_blocking` 閉包
- **THEN** `ScanCache` 以 `Arc<ScanCache>` clone 後移入閉包共享狀態
- **AND** DB 連線於閉包內以 `open_db_connection()` 另開，不跨 `await` 持有 `DbState` 的 MutexGuard

### Requirement: forceFull 旗標不得進入 sessionsQuery 的 queryKey

前端控制「下一次掃描是否強制全掃」的 `forceFull` 旗標 SHALL 以 `useRef` 保存，不得納入 `sessionsQuery` 的 `queryKey`，且不得在 `queryFn` 執行過程中改變 queryKey，以避免每次 fetch 完成因 queryKey 變動而連續觸發第二次 fetch，導致 `isFetching` 永遠為 `true`、狀態列卡在「掃描中」。

#### Scenario: 強制全掃透過 ref 傳遞

- **WHEN** 封存 / 解封存 / 刪除 session 或清除空 session 後需要全掃
- **THEN** 設定 `forceFullRef.current = true`，再呼叫 `invalidateQueries(["sessions"])` 觸發掃描
- **AND** `queryFn` 讀取 `forceFullRef.current` 後立即重設為 `false`

#### Scenario: 掃描完成後 isFetching 正常歸零

- **WHEN** 一次 `get_sessions` 掃描完成且無待處理的 invalidate
- **THEN** queryKey 未因本次掃描而變動，不觸發額外 fetch
- **AND** `sessionsQuery.isFetching` 回到 `false`，狀態列「掃描中」指示結束

### Requirement: 後端提供快取 session 列表查詢
系統 SHALL 提供 `get_sessions_cached` Tauri command，從 SQLite `sessions_cache` 表直接回傳上次掃描結果，不執行任何檔案系統掃描。

#### Scenario: 冷啟動時回傳快取資料
- **WHEN** 前端 settings 載入完成後立即呼叫 `get_sessions_cached`
- **THEN** 後端在 ≤100ms 內回傳 `Vec<SessionInfo>`，內容來自 `sessions_cache` SQLite 表

#### Scenario: 快取為空時回傳空列表
- **WHEN** `sessions_cache` 表尚無資料（首次啟動）
- **THEN** 回傳空 `Vec<SessionInfo>`，前端繼續等待完整掃描

#### Scenario: show_archived 過濾
- **WHEN** 呼叫時傳入 `show_archived: false`
- **THEN** 回傳結果中不包含 `is_archived: true` 的 session

### Requirement: 前端冷啟動顯示快取資料
系統 SHALL 在完整 session 掃描完成前，以 SQLite 快取資料填充 session 列表，讓用戶在 ≤100ms 內看到上次的資料。

#### Scenario: 快取資料顯示後背景掃描
- **WHEN** `sessionsCachedQuery` 回傳資料且 `sessionsQuery` 尚未完成
- **THEN** UI 顯示快取 session 列表，不顯示空白畫面

#### Scenario: 完整掃描完成後無縫替換
- **WHEN** `sessionsQuery`（完整掃描）完成
- **THEN** session 列表更新為最新掃描結果，無閃爍過渡

