## MODIFIED Requirements

### Requirement: 多工具啟動下拉選單

系統 SHALL 在 SessionCard 與 ProjectCard 提供多工具啟動下拉選單，讓使用者選擇以哪種工具開啟。

#### Scenario: 顯示工具選單

- **WHEN** 使用者點擊 SessionCard 或 ProjectCard 上的啟動按鈕
- **THEN** 系統顯示包含以下選項的下拉選單：Terminal、OpenCode、Copilot CLI、Gemini CLI、File Explorer
- **AND** 預設啟動工具的選項旁標示「預設」標籤
- **AND** 選單 SHALL 以 `position: fixed` 渲染於最上層，不被其他卡片遮蓋

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

#### Scenario: 同時只能展開一個選單

- **WHEN** 使用者已展開某個 SessionCard 的工具選單
- **AND** 使用者點擊另一個 SessionCard 的啟動按鈕
- **THEN** 原先展開的選單 SHALL 自動關閉
- **AND** 新選單 SHALL 展開

#### Scenario: 點擊外部關閉選單

- **WHEN** 工具選單已展開
- **AND** 使用者點擊選單外部任意區域
- **THEN** 選單 SHALL 自動關閉
