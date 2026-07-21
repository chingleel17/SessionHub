## MODIFIED Requirements

### Requirement: SQLite 快取統計結果

系統 SHALL 將計算完成的統計結果快取至 `session_stats` 資料表，以 mtime 失效。快取寫入的觸發時機 SHALL 包含：
1. 使用者點開 session 時（原有行為）
2. 背景 backfill 流程（新增）：app 啟動後 `get_sessions` 完成時，對缺少快取的已完成 session 批次寫入

#### Scenario: 快取命中

- **WHEN** session 非 live 且 events_mtime 未變更
- **THEN** 直接回傳快取統計，不重新解析

#### Scenario: 背景 backfill 寫入快取

- **WHEN** backfill 流程解析完一個 session 的 events.jsonl 且 session 非 live
- **THEN** 統計結果寫入 `session_stats`，後續查詢直接命中快取
