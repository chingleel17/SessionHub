## ADDED Requirements

### Requirement: Copilot 增量掃描
系統 SHALL 在快取存在且未超過全掃閾值時，透過比對目錄 mtime 進行增量掃描：僅重新讀取 `workspace.yaml` 的目錄為 mtime 有變化或新增的 session；mtime 未變的 session SHALL 直接從快取中取得；快取中存在但目錄已消失的 session SHALL 從結果中移除。

#### Scenario: 只有一個 session 目錄更新
- **WHEN** Copilot watcher 觸發增量掃描，且 100 個 session 中只有 1 個目錄的 mtime 有變化
- **THEN** 系統只讀取該 1 個目錄的 `workspace.yaml`，其餘 99 個從快取取得，合併後回傳完整列表

#### Scenario: 新增一個 session 目錄
- **WHEN** Copilot CLI 建立新的 session 目錄
- **THEN** 該目錄的 session_id 不存在於 `session_mtimes` 快取，系統讀取其 `workspace.yaml` 並加入快取

#### Scenario: 刪除一個 session 目錄
- **WHEN** session 目錄被移除
- **THEN** 該 session_id 從 `session_mtimes` 與 `sessions` 快取中移除，回傳的列表不包含該 session

### Requirement: OpenCode 增量掃描
系統 SHALL 在快取存在且未超過全掃閾值時，使用 `last_cursor` 對 SQLite 執行增量查詢：`SELECT * FROM sessions WHERE time_updated > {last_cursor}`；查詢結果 SHALL upsert 進 `cache.sessions`；`last_cursor` SHALL 更新為新結果中最大的 `time_updated` 值；若查詢結果為空，快取 SHALL 維持不變。

#### Scenario: OpenCode 有新 session
- **WHEN** OpenCode watcher 觸發增量掃描，且 SQLite 有 `time_updated > last_cursor` 的新行
- **THEN** 系統只查詢新增/更新的行，將其 upsert 進快取，並更新 `last_cursor`

#### Scenario: OpenCode 無新變動
- **WHEN** OpenCode watcher 觸發增量掃描，且 SQLite 無 `time_updated > last_cursor` 的行
- **THEN** 系統回傳現有快取，不執行額外磁碟 I/O，`last_cursor` 不變

### Requirement: get_sessions 支援 force_full 參數
`get_sessions` command SHALL 接受可選的 `force_full: Option<bool>` 參數。當 `force_full` 為 `Some(true)` 時，SHALL 對所有 provider 執行全掃並重置快取，忽略 `last_full_scan_at`。

#### Scenario: 呼叫時不傳 force_full
- **WHEN** `get_sessions` 被呼叫且未傳入 `force_full`
- **THEN** 系統依照正常快取邏輯決定全掃或增量掃描

#### Scenario: 封存後呼叫帶 force_full
- **WHEN** `get_sessions` 被呼叫且 `force_full: true`
- **THEN** 系統執行全掃，快取重置為最新狀態
