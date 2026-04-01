## ADDED Requirements

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
