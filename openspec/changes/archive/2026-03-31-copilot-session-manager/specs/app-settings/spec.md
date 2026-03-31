## ADDED Requirements

### Requirement: 設定持久化
系統 SHALL 將應用程式設定儲存於 `%APPDATA%\CopilotSessionManager\settings.json`，應用程式重啟後設定保持不變。

#### Scenario: 儲存設定
- **WHEN** 使用者修改設定並點擊儲存
- **THEN** 系統將設定寫入 `settings.json`
