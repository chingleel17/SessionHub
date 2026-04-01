## MODIFIED Requirements

### Requirement: 目錄變更即時偵測
系統 SHALL 使用 OS filesystem watch 監聽 `session-state/` 目錄（Copilot）及 OpenCode SQLite WAL 檔案的新增、刪除、修改事件，並分別透過 provider-specific Tauri event 通知前端。Copilot watcher SHALL 發送 `copilot-sessions-updated` 事件；OpenCode watcher SHALL 發送 `opencode-sessions-updated` 事件。兩個事件均 SHALL 在各自的防抖期間結束後才發送（Copilot 800ms、OpenCode 500ms）。

#### Scenario: 新 session 目錄建立
- **WHEN** Copilot CLI 在 `session-state/` 下建立新的 session 目錄
- **THEN** 前端 UI 在防抖結束後（最多 800ms + 掃描時間）自動更新 session 列表，總延遲 SHALL 不超過 3 秒

#### Scenario: 兩個事件名稱分離
- **WHEN** Copilot 目錄發生變更
- **THEN** 系統發送 `copilot-sessions-updated`，而非 `opencode-sessions-updated`，且反之亦然

#### Scenario: OpenCode session 新增
- **WHEN** OpenCode 在 SQLite 中建立新 session
- **THEN** 系統發送 `opencode-sessions-updated`，前端透過此事件 invalidate session query 並取得更新後的列表
