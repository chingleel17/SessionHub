## ADDED Requirements

### Requirement: 多工具啟動下拉選單

系統 SHALL 在 SessionCard 與 ProjectCard 提供多工具啟動下拉選單，讓使用者選擇以哪種工具開啟。

#### Scenario: 顯示工具選單

- **WHEN** 使用者點擊 SessionCard 或 ProjectCard 上的啟動按鈕
- **THEN** 系統顯示包含以下選項的下拉選單：Terminal、OpenCode、Copilot CLI、Gemini CLI、File Explorer
- **AND** 預設啟動工具的選項旁標示「預設」標籤

#### Scenario: 以 Terminal 開啟

- **WHEN** 使用者從下拉選單選擇「Terminal」
- **THEN** 系統以設定的終端機路徑及 cwd 啟動終端（現有邏輯）

#### Scenario: 以 OpenCode 開啟

- **WHEN** 使用者從下拉選單選擇「OpenCode」
- **THEN** 系統在終端中執行 `opencode --cwd <cwd>`

#### Scenario: 以 Copilot CLI 開啟

- **WHEN** 使用者從下拉選單選擇「Copilot CLI」
- **AND** session 有有效的 session_id
- **THEN** 系統在終端中執行 `copilot session resume <session_id>`（使用 `copilot` standalone CLI）
- **AND** 若指令失敗，顯示 toast 提示錯誤訊息

#### Scenario: 以 Gemini CLI 開啟

- **WHEN** 使用者從下拉選單選擇「Gemini CLI」
- **THEN** 系統在終端中執行 `gemini`，工作目錄設為 cwd

#### Scenario: 以 File Explorer 開啟

- **WHEN** 使用者從下拉選單選擇「File Explorer」
- **THEN** 系統執行 `explorer.exe <cwd>`，不開啟終端視窗

### Requirement: Platform-Aware 預設啟動工具

系統 SHALL 依照 session 的 provider 決定啟動按鈕的預設工具，而非僅依全域設定。

#### Scenario: Copilot session 預設以 Copilot CLI 開啟

- **WHEN** 使用者點擊 Copilot session 的主啟動按鈕
- **THEN** 系統預設使用 `copilot` 工具啟動（而非全域 defaultLauncher）

#### Scenario: OpenCode session 預設以 OpenCode 開啟

- **WHEN** 使用者點擊 OpenCode session 的主啟動按鈕
- **THEN** 系統預設使用 `opencode` 工具啟動

#### Scenario: 退回全域設定

- **WHEN** session provider 無法對應特定工具，或 platform-aware 工具不可用
- **THEN** 系統退回 `AppSettings.defaultLauncher`，再退回 `terminal`

#### Scenario: 全域設定作為覆蓋

- **WHEN** 使用者在設定頁明確設定「預設啟動工具」
- **THEN** 該設定覆蓋 platform-aware 邏輯，所有 session 均使用此工具



#### Scenario: 使用預設工具啟動

- **WHEN** 使用者點擊 SessionCard 的主啟動按鈕（非展開下拉）
- **THEN** 系統以 AppSettings.defaultLauncher 設定的工具啟動
- **AND** 若未設定 defaultLauncher，則預設使用 Terminal

### Requirement: 設定頁預設啟動工具選項

系統 SHALL 在設定頁提供「預設啟動工具」選項，讓使用者選擇點擊啟動按鈕時的預設行為。

#### Scenario: 設定預設工具

- **WHEN** 使用者在設定頁選擇預設啟動工具（Terminal / OpenCode / Copilot CLI / Gemini CLI / File Explorer）
- **THEN** 系統將選擇儲存至 AppSettings.defaultLauncher
- **AND** SessionCard 主啟動按鈕的 icon 更新為對應工具的 icon

#### Scenario: 設定儲存並生效

- **WHEN** 使用者儲存設定
- **THEN** 所有 SessionCard 立即反映新的預設啟動工具
