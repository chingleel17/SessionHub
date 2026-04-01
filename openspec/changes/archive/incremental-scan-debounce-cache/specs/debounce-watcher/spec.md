## ADDED Requirements

### Requirement: Copilot watcher 800ms 防抖
Copilot FS watcher 的 callback SHALL 實作 800ms 防抖：在最後一個 FS 事件發生後 800ms 內若有新事件，計時 SHALL 重置；只有在 800ms 靜止後，系統才執行 Copilot 掃描並發送 `copilot-sessions-updated` 事件。

#### Scenario: Copilot CLI 連續寫入爆發
- **WHEN** Copilot CLI 在 500ms 內連續觸發 10 個 FS 事件
- **THEN** 系統在最後一個事件後 800ms 才執行一次掃描，期間不發送任何 `copilot-sessions-updated` 事件

#### Scenario: 單一 FS 事件後靜止
- **WHEN** Copilot FS watcher 偵測到一個事件，之後 800ms 無新事件
- **THEN** 系統在 800ms 後執行一次掃描並發送 `copilot-sessions-updated`

### Requirement: OpenCode watcher 500ms 防抖
OpenCode WAL watcher 的 callback SHALL 實作 500ms 防抖：在最後一個 WAL 事件發生後 500ms 內若有新事件，計時 SHALL 重置；只有在 500ms 靜止後，系統才執行 OpenCode 增量查詢並發送 `opencode-sessions-updated` 事件。

#### Scenario: SQLite WAL checkpoint 觸發多個事件
- **WHEN** OpenCode WAL checkpoint 在 200ms 內觸發 3 個 FS 事件
- **THEN** 系統在最後一個事件後 500ms 才執行一次 OpenCode 查詢，期間不發送任何 `opencode-sessions-updated` 事件

#### Scenario: 單一 WAL 事件後靜止
- **WHEN** OpenCode WAL watcher 偵測到一個事件，之後 500ms 無新事件
- **THEN** 系統在 500ms 後執行一次 OpenCode 查詢並發送 `opencode-sessions-updated`
