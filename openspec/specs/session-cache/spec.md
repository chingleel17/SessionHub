## ADDED Requirements

### Requirement: SQLite session cache 取代 JSON 檔案

系統 SHALL 以 SQLite 資料表取代 `session_cache.json`，統一快取機制至 `metadata.db`。

#### Scenario: session_cache.json 廢棄

- **WHEN** 系統啟動時發現 `%APPDATA%\SessionHub\session_cache.json` 存在
- **THEN** 若版本未遷移，一次性讀取後寫入 SQLite，並刪除 JSON 檔案

### Requirement: SQLite 三表快取架構

`metadata.db` SHALL 包含以下三個快取相關資料表：

#### Scenario: sessions_cache 資料表

- 欄位：`session_id TEXT PK`、`provider TEXT`、`cwd TEXT`、`summary TEXT`、`summary_count INT`、`created_at INT`、`updated_at INT`、`has_plan BOOL`、`has_events BOOL`、`parse_error TEXT`、`raw_json TEXT`

#### Scenario: scan_state 資料表

- 欄位：`provider TEXT PK`、`last_full_scan_at INT`、`last_cursor INT`

#### Scenario: session_mtimes 資料表

- 欄位：`session_id TEXT PK`、`provider TEXT`、`mtime INT`

### Requirement: 快取讀取策略

系統 SHALL 優先使用 SQLite 快取，僅在 mtime 變更時重新解析。

#### Scenario: 命中快取

- **WHEN** session 的 workspace.yaml mtime 未變更且 sessions_cache 中有對應記錄
- **THEN** 直接回傳快取的 SessionInfo，不重新讀取檔案

#### Scenario: 快取失效

- **WHEN** session 的 workspace.yaml mtime 已變更
- **THEN** 重新解析 workspace.yaml，更新 sessions_cache 與 session_mtimes
