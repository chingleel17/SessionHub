## ADDED Requirements

### Requirement: 目錄變更即時偵測
系統 SHALL 使用 OS filesystem watch 監聽 `session-state/` 目錄的新增、刪除、修改事件，並透過 Tauri event 通知前端。

#### Scenario: 新 session 目錄建立
- **WHEN** Copilot CLI 在 `session-state/` 下建立新的 session 目錄
- **THEN** 前端 UI 在 3 秒內自動更新 session 列表
