## MODIFIED Requirements

### Requirement: 應用程式設定結構
系統 SHALL 讀寫 `%APPDATA%\SessionHub\settings.json`，設定結構包含 `copilotRoot`、`terminalPath`、`externalEditorPath`、`showArchived`，以及新增的 `pinnedProjects`（string[] 型別，預設空陣列）。

#### Scenario: 讀取不含 pinnedProjects 的舊設定
- **WHEN** settings.json 不含 `pinnedProjects` 欄位（舊版設定）
- **THEN** 系統以空陣列 `[]` 作為預設值，不報錯

#### Scenario: 儲存釘選專案設定
- **WHEN** 使用者釘選或取消釘選專案後呼叫 save_settings
- **THEN** settings.json 中的 `pinnedProjects` 更新為最新的 project key 陣列

#### Scenario: 讀取含 pinnedProjects 的設定
- **WHEN** settings.json 包含 `pinnedProjects: ["C:\\projects\\foo", "C:\\projects\\bar"]`
- **THEN** 系統正確回傳此陣列至前端，前端據以還原釘選狀態
