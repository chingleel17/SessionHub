# session-cache Specification

## Purpose
規範 SQLite session 快取、校正後專案資訊與掃描進度的持久化順序，確保冷啟動可直接使用一致的顯示資料。

## Requirements

### Requirement: SQLite session cache 取代 JSON 檔案

系統 SHALL 以 SQLite 資料表取代 `session_cache.json`，統一快取機制至 `metadata.db`。

#### Scenario: session_cache.json 廢棄

- **WHEN** 系統啟動時發現 `%APPDATA%\SessionHub\session_cache.json` 存在
- **THEN** 若版本未遷移，一次性讀取後寫入 SQLite，並刪除 JSON 檔案

### Requirement: SQLite 三表快取架構

`metadata.db` SHALL 包含以下三個快取相關資料表：

#### Scenario: sessions_cache 資料表

- **WHEN** 系統初始化 `sessions_cache` 資料表
- **THEN** 以 `(session_id, provider)` 為複合主鍵，保存 `cwd`、`repo_root`、`repo_name`、`git_branch`、`summary`、`created_at`、`updated_at`、`session_dir` 等文字欄位，以及 `summary_count` 整數欄位
- **AND** `parse_error`、`is_archived`、`has_plan`、`has_events` 以整數表示布林值，預設為 0

#### Scenario: scan_state 資料表

- **WHEN** 系統初始化 `scan_state` 資料表
- **THEN** 建立 `provider TEXT PK`、`last_full_scan_at INTEGER` 與 `last_cursor INTEGER` 欄位，時間與游標預設為 0

#### Scenario: session_mtimes 資料表

- **WHEN** 系統初始化 `session_mtimes` 資料表
- **THEN** 建立 `session_id TEXT PK`、`provider TEXT` 與 `mtime INTEGER` 欄位，mtime 預設為 0

### Requirement: 快取讀取策略

系統 SHALL 優先使用 SQLite 快取，僅在 mtime 變更時重新解析。

#### Scenario: 命中快取

- **WHEN** session 的 workspace.yaml mtime 未變更且 sessions_cache 中有對應記錄
- **THEN** 直接回傳快取的 SessionInfo，不重新讀取檔案

#### Scenario: 快取失效

- **WHEN** session 的 workspace.yaml mtime 已變更
- **THEN** 重新解析 workspace.yaml 並更新記憶體中的掃描結果
- **AND** 依下述持久化順序，在校正完成的顯示快取保存成功後，才更新磁碟中的掃描進度與 session_mtimes

### Requirement: 顯示快取保存校正後的專案資訊

系統 SHALL 在 `sessions_cache` 保存回傳前端的 session 顯示快照，包含 `cwd`、`repo_root`、`repo_name` 與 `git_branch`。完整掃描 SHALL 先套用路徑對應，再解析 Git 資訊及套用 session metadata，最後才保存顯示快取。原始 provider 掃描資料與記憶體 `ProviderCache` SHALL NOT 在掃描途中覆寫此快照。

#### Scenario: 重啟直接沿用已確認的專案名稱
- **WHEN** 背景掃描已將舊專案路徑對應至新目錄，且解析得到名稱 `docs`
- **THEN** 保存的快取包含校正後的路徑與 `repo_name` 值 `docs`
- **AND** 下次冷啟動在完整掃描完成前即可使用該名稱，不必再次查詢 Git

#### Scenario: 掃描途中仍可讀取上次完成的顯示快照
- **WHEN** 某 provider 的原始 session 資料仍包含舊路徑或舊名稱，且新一輪掃描尚未完成校正
- **THEN** `sessions_cache` 保留上次完成校正的結果
- **AND** 同時發生的快取查詢不會讀到本次尚未校正的中間結果

### Requirement: 掃描進度不得超前於已保存的顯示快取

系統 SHALL 將顯示快取的保存與 provider 掃描進度的保存分開。僅在校正後的 `sessions_cache` 保存成功後，才可寫入對應 provider 的 `scan_state` 與 `session_mtimes`。保存掃描進度 SHALL NOT 重寫 session 顯示資料；記憶體 provider 快取仍維持掃描用途，不因保存進度而被替換成顯示快照。

#### Scenario: 保存游標與 mtime 不覆寫名稱
- **WHEN** 顯示快取已保存 `docs`，記憶體 provider 快取仍包含原始舊名稱
- **AND** 系統保存該 provider 的 `last_full_scan_at`、`last_cursor` 與 session mtimes
- **THEN** 掃描進度成功保存，顯示快取中的名稱仍為 `docs`

#### Scenario: 校正或快取保存失敗不得推進磁碟游標
- **WHEN** 掃描在專案資訊校正期間中斷，或校正後的顯示快取保存失敗
- **THEN** 本次掃描不得先將新的游標或 mtime 寫入磁碟
- **AND** 下次啟動不會因掃描進度超前於快取而跳過尚未保存的 session
