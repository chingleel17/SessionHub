## ADDED Requirements

### Requirement: 從 session.shutdown 擷取 modelMetrics

系統 SHALL 在解析 Copilot `events.jsonl` 時，讀取 `session.shutdown` 事件中的 `modelMetrics` 欄位，提取每個 model 的 `requests.count`（整數）、`requests.cost`（f64，Copilot 計費點數，支援小數）、`inputTokens`（u64）、`outputTokens`（u64），並存入 `SessionStats.modelMetrics`。

#### Scenario: shutdown 事件存在且含 modelMetrics

- **WHEN** `events.jsonl` 中有 `session.shutdown` 事件且 `data.modelMetrics` 非空
- **THEN** `SessionStats.modelMetrics` 應包含各 model key 及對應的 requestsCount、requestsCost、inputTokens、outputTokens

#### Scenario: shutdown 事件不存在或被中斷

- **WHEN** session 被強制終止，`events.jsonl` 中無 `session.shutdown` 事件
- **THEN** `SessionStats.modelMetrics` SHALL 為空物件（`{}`），其餘統計欄位不受影響

#### Scenario: 多個 model 在同一 session 使用

- **WHEN** `modelMetrics` 中有多個 model key（如 `claude-sonnet-4.6`、`gpt-4o`）
- **THEN** 所有 model 的 metrics 都 SHALL 被保留在 `modelMetrics` 中

### Requirement: model_metrics 欄位持久化至 SQLite

系統 SHALL 在 `session_stats` 表新增 `model_metrics TEXT` 欄位（JSON 格式），在 upsert 時儲存、在查詢時回傳。

#### Scenario: 首次建立 stats 快取

- **WHEN** session_stats 中不存在該 session_id 的記錄，且系統計算出 modelMetrics
- **THEN** model_metrics 欄位 SHALL 儲存序列化後的 JSON 字串

#### Scenario: 更新已有 stats 快取

- **WHEN** session 的 events.jsonl 被修改（mtime 改變），系統重新解析
- **THEN** model_metrics 欄位 SHALL 被更新為最新 modelMetrics JSON

#### Scenario: 既有資料庫無 model_metrics 欄位（schema migration）

- **WHEN** 應用程式啟動，`session_stats` 表尚無 `model_metrics` 欄位
- **THEN** 系統 SHALL 自動執行 `ALTER TABLE session_stats ADD COLUMN model_metrics TEXT`，不影響既有資料
