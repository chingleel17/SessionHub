## MODIFIED Requirements

### Requirement: 終端執行檔驗證

系統 SHALL 在儲存設定前驗證使用者指定的終端執行檔路徑是否存在且可執行。

#### Scenario: 有效的終端路徑

- **WHEN** 使用者輸入終端路徑並儲存
- **THEN** 系統驗證路徑對應的可執行檔存在

### Requirement: 終端類型白名單與啟動參數

系統 SHALL 依終端類型（pwsh / cmd / bash）選用對應的啟動參數，以 file_stem 白名單識別終端類型。

#### Scenario: 識別 PowerShell

- **WHEN** terminal_path 的 file_stem（不含副檔名）為 `pwsh` 或 `powershell`
- **THEN** 以 `-NoExit -Command Set-Location -Path <cwd>` 啟動並切換目錄

#### Scenario: 識別 cmd

- **WHEN** terminal_path 的 file_stem 為 `cmd`
- **THEN** 以 `/K cd /d <cwd>` 啟動並切換目錄

#### Scenario: 識別 bash / sh / zsh

- **WHEN** terminal_path 的 file_stem 為 `bash`、`sh` 或 `zsh`
- **THEN** 以 `--init-file <(echo "cd <cwd>")` 或 `--rcfile` 方式啟動

#### Scenario: 未知終端類型

- **WHEN** file_stem 不在白名單內
- **THEN** 直接以 cwd 作為工作目錄啟動終端，不附加額外參數

### Requirement: 依 provider 類型提供複製指令

系統 SHALL 在 session 操作中提供「複製啟動指令」功能，指令格式依 session 的 provider 而異。

#### Scenario: Copilot session 複製指令

- **WHEN** 使用者點擊 Copilot session 的「複製指令」
- **THEN** 複製 `gh copilot session resume <session-id>` 至剪貼簿

#### Scenario: OpenCode session 複製指令

- **WHEN** 使用者點擊 OpenCode session 的「複製指令」
- **THEN** 複製 `opencode --session <session-id>` 至剪貼簿

## ADDED Requirements

### Requirement: 多工具啟動指令路由

系統 SHALL 在 `open_in_tool` command 中依 tool_type 決定啟動邏輯，統一處理所有工具的啟動參數。

#### Scenario: terminal 類型路由

- **WHEN** open_in_tool 收到 tool_type 為 `terminal`
- **THEN** 套用現有終端啟動邏輯（settings.terminal_path + file_stem 白名單）

#### Scenario: opencode 類型路由

- **WHEN** open_in_tool 收到 tool_type 為 `opencode`
- **THEN** 在終端中執行 `opencode --cwd <cwd>`，以 settings.terminal_path 開啟終端

#### Scenario: gh-copilot 類型路由

- **WHEN** open_in_tool 收到 tool_type 為 `gh-copilot` 且 session_id 不為空
- **THEN** 在終端中執行 `gh copilot session resume <session_id>`

#### Scenario: gemini 類型路由

- **WHEN** open_in_tool 收到 tool_type 為 `gemini`
- **THEN** 在終端中執行 `gemini`，工作目錄設為 cwd

#### Scenario: explorer 類型路由

- **WHEN** open_in_tool 收到 tool_type 為 `explorer`
- **THEN** 直接 spawn `explorer.exe <cwd>`，不開啟終端視窗
