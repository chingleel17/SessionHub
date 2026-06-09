## ADDED Requirements

### Requirement: 啟動時自動補算缺失的 session stats

系統 SHALL 在 `get_sessions` 掃描完成後，於背景對所有「已加入 `sessions_cache` 但缺少 `session_stats`」且「非 live、有 `events.jsonl`」的 Copilot session 自動解析並寫入 `session_stats`，單次執行上限為 50 個 session。

#### Scenario: 有待補算的完成 session

- **WHEN** app 啟動後 `get_sessions` 完成，且存在未計算 stats 的 Copilot session（有 events.jsonl、無 lock 檔）
- **THEN** 系統在背景非同步補算，不阻塞 UI 回應，補算完成後前端可查詢到完整統計

#### Scenario: 超過 50 個待補算 session

- **WHEN** 未計算 stats 的 session 數量超過 50
- **THEN** 僅補算最新 50 個（依 `updated_at` DESC），其餘留待下次啟動

#### Scenario: session 仍在執行（live）

- **WHEN** session 目錄下存在 `inuse.*.lock` 檔案
- **THEN** 跳過此 session，不補算也不快取

#### Scenario: events.jsonl 不存在

- **WHEN** session 在 sessions_cache 但無 events.jsonl
- **THEN** 跳過，不補算

#### Scenario: 補算失敗

- **WHEN** 解析 events.jsonl 發生 IO 或 parse error
- **THEN** 記錄警告但繼續處理其他 session，不中斷整體補算流程

### Requirement: 背景補算不重複計算已有快取的 session

系統 SHALL 在選取補算目標前，先排除 `session_stats` 已有記錄的 session，避免重複解析。

#### Scenario: 已有快取的 session 不重新解析

- **WHEN** `session_stats` 已有對應 session_id 的記錄
- **THEN** 跳過此 session，不重新讀取 events.jsonl
