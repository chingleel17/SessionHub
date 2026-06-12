# sessions-cached-query Specification

## Purpose
TBD - created by archiving change fix-perf-issues. Update Purpose after archive.
## Requirements
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

