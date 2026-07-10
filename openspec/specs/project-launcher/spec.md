## Requirements

### Requirement: 專案層級開啟方式選單

系統 SHALL 在 ProjectView 頂部（sticky header 區）提供「開啟專案」按鈕與下拉選單，讓使用者以指定工具開啟**專案路徑**。

#### Scenario: 顯示專案開啟選單

- **WHEN** 使用者點擊 ProjectView 頂部「開啟專案」按鈕旁的展開箭頭
- **THEN** 系統顯示下拉選單，選項包含：Terminal、VS Code、File Explorer、Copilot CLI、OpenCode、Gemini CLI
- **AND** `AppSettings.defaultLauncher` 對應的選項旁標示「預設」
- **AND** 選單以最上層渲染，不被其他內容遮蓋，點擊選單外部即關閉

#### Scenario: 以選定工具開啟專案路徑

- **WHEN** 使用者從選單選擇一種工具
- **THEN** 系統以該工具開啟專案路徑（`project.pathLabel`）：Terminal 開新終端於該路徑、VS Code 執行 `code <path>`、File Explorer 執行 `explorer <path>`、CLI 工具在新終端於該路徑啟動
- **AND** 成功時顯示 toast 確認

#### Scenario: 主按鈕以預設工具開啟

- **WHEN** 使用者直接點擊「開啟專案」主按鈕（未展開選單）
- **THEN** 系統以 `AppSettings.defaultLauncher` 指定的工具開啟專案路徑
- **AND** 未設定 defaultLauncher 時以 Terminal 開啟

#### Scenario: 工具未安裝

- **WHEN** `ToolAvailability` 顯示某工具不可用（copilot / opencode / gemini / vscode）
- **THEN** 該選項 SHALL 標示未安裝並停用

#### Scenario: 專案路徑不存在

- **WHEN** 專案路徑為空或目錄不存在
- **THEN** 系統顯示 toast 提示（`toast.cwdMissing`），不執行開啟
