## MODIFIED Requirements

### Requirement: 終端執行檔驗證
系統 SHALL 在儲存設定前驗證使用者指定的終端執行檔路徑，檢查條件包含：
1. 路徑對應的檔案必須存在。
2. 檔案的 `file_stem()` 必須為以下之一（不區分大小寫）：`pwsh`、`powershell`、`cmd`、`bash`、`sh`。

#### Scenario: 有效的終端路徑（pwsh）
- **WHEN** 使用者輸入指向 `pwsh.exe` 的路徑並儲存
- **THEN** 系統驗證通過，設定儲存成功

#### Scenario: 有效的終端路徑（cmd）
- **WHEN** 使用者輸入指向 `cmd.exe` 的路徑並儲存
- **THEN** 系統驗證通過，設定儲存成功

#### Scenario: 無效的終端路徑（不在白名單）
- **WHEN** 使用者輸入指向 `notepad.exe` 的路徑並儲存
- **THEN** 系統回傳驗證錯誤，設定不儲存

#### Scenario: 路徑不存在
- **WHEN** 使用者輸入不存在的路徑並儲存
- **THEN** 系統回傳驗證錯誤，設定不儲存

## ADDED Requirements

### Requirement: 依終端機類型帶入啟動參數
系統 SHALL 依偵測到的終端機類型（從 `file_stem()` 判斷）帶入對應的啟動參數以開啟終端機至指定工作目錄：
- `pwsh` / `powershell`：使用 `-NoExit -Command "cd '<dir>'"`
- `cmd`：使用 `/K "cd /d <dir>"`
- `bash` / `sh`：以 `current_dir` 設定工作目錄，使用 `-i` 啟動互動 shell

#### Scenario: 以 pwsh 開啟終端
- **WHEN** 使用者設定終端為 `pwsh.exe` 並觸發「開啟終端」
- **THEN** 系統以 `-NoExit -Command "cd '<sessionDir>'"` 啟動 pwsh 並切換至 session 工作目錄

#### Scenario: 以 cmd 開啟終端
- **WHEN** 使用者設定終端為 `cmd.exe` 並觸發「開啟終端」
- **THEN** 系統以 `/K "cd /d <sessionDir>"` 啟動 cmd 並切換至 session 工作目錄

### Requirement: 複製指令依 Session Provider 分支
系統 SHALL 依 session 的 `provider` 欄位輸出對應的 resume 指令：
- `copilot` provider：`copilot --resume=<sessionId>`
- `opencode` provider：`opencode session <sessionId>`

#### Scenario: 複製 Copilot session 指令
- **WHEN** 使用者對 `provider = "copilot"` 的 session 點擊「複製指令」
- **THEN** 剪貼簿內容為 `copilot --resume=<sessionId>`

#### Scenario: 複製 OpenCode session 指令
- **WHEN** 使用者對 `provider = "opencode"` 的 session 點擊「複製指令」
- **THEN** 剪貼簿內容為 `opencode session <sessionId>`
