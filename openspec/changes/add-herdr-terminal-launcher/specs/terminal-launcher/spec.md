## ADDED Requirements

### Requirement: 終端啟動器種類分派

系統 SHALL 依設定的終端啟動器種類（`terminal_launcher`）決定啟動方式，所有終端啟動入口（一般終端、AI coding CLI 啟動、session resume）SHALL 共用同一套分派規則。

#### Scenario: 預設為 shell 啟動器

- **WHEN** 設定中 `terminal_launcher` 為 `"shell"` 或未設定
- **THEN** 系統沿用既有 shell 啟動邏輯（`terminal_path` 搭配 file_stem 白名單，並以獨立 console 視窗開啟）

#### Scenario: 選用 herdr 啟動器

- **WHEN** 設定中 `terminal_launcher` 為 `"herdr"`
- **THEN** 系統改以 herdr 建立 tab／pane 承載該次啟動，不另開作業系統 console 視窗

#### Scenario: 未知啟動器種類

- **WHEN** `terminal_launcher` 的值不在支援清單內
- **THEN** 系統以 `"shell"` 行為啟動，不中斷使用者操作

### Requirement: herdr 兩段式啟動流程

在 herdr 啟動器模式下，系統 SHALL 先建立 tab 取得目標 pane 識別碼，再將啟動指令送入該 pane。

#### Scenario: 開啟純終端

- **WHEN** 使用者以 herdr 啟動器開啟終端且無初始指令
- **THEN** 系統建立一個工作目錄為目標 cwd 的 herdr tab
- **AND** 不再送出額外指令

#### Scenario: 啟動 AI coding CLI

- **WHEN** 使用者以 herdr 啟動器啟動某個 AI coding CLI（如 claude／codex／copilot／opencode／gemini）
- **THEN** 系統建立工作目錄為目標 cwd 的 herdr tab
- **AND** 自建立結果取得該 tab 的 pane 識別碼
- **AND** 將該 CLI 的啟動指令送入該 pane 執行

#### Scenario: 恢復既有 session

- **WHEN** 使用者以 herdr 啟動器恢復某個 session
- **THEN** 系統建立 tab 後，將對應 provider 的 resume 指令送入該 pane 執行

#### Scenario: tab 標示

- **WHEN** 系統以 herdr 建立 tab
- **THEN** 該 tab 帶有可辨識該次啟動目標的標籤，便於使用者在多個 tab 間識別

### Requirement: herdr 不可用時的錯誤處理

系統 SHALL 在 herdr 無法完成啟動時回傳明確錯誤訊息，不得靜默失敗或產生無回應的操作。

#### Scenario: herdr 未安裝

- **WHEN** 使用者選用 herdr 啟動器但系統找不到 herdr 可執行檔
- **THEN** 系統回傳指出 herdr 不可用的錯誤訊息
- **AND** 前端以 toast 呈現該錯誤

#### Scenario: herdr server 未執行或建立 tab 失敗

- **WHEN** herdr 回報非成功狀態，或其輸出無法解析出 pane 識別碼
- **THEN** 系統回傳包含失敗原因的錯誤訊息，且不再嘗試送出後續指令

## MODIFIED Requirements

### Requirement: 終端執行檔驗證

系統 SHALL 在儲存設定前驗證使用者指定的終端執行檔路徑，驗證方式依終端啟動器種類而異。

#### Scenario: 有效的終端路徑

- **WHEN** 啟動器為 shell 且使用者輸入終端路徑並儲存
- **THEN** 系統驗證路徑對應的可執行檔存在

#### Scenario: herdr 啟動器允許 PATH 解析的指令名

- **WHEN** 啟動器為 herdr 且指定的啟動指令為不含目錄的裸指令名
- **THEN** 系統以 PATH 解析該指令是否可用，不要求其為可瀏覽的檔案路徑
- **AND** 解析成功即視為通過驗證

#### Scenario: herdr 指令無法解析

- **WHEN** 啟動器為 herdr 且指定的指令在 PATH 與檔案系統中皆無法解析
- **THEN** 系統回報驗證失敗，並提示 herdr 不可用

### Requirement: 終端類型白名單與啟動參數

在 shell 啟動器模式下，系統 SHALL 依終端類型（pwsh / cmd / bash）選用對應的啟動參數，以 file_stem 白名單識別終端類型。此白名單在 herdr 啟動器模式下不適用。

#### Scenario: 識別 PowerShell

- **WHEN** 啟動器為 shell 且 terminal_path 的 file_stem（不含副檔名）為 `pwsh` 或 `powershell`
- **THEN** 以 `-NoExit -Command Set-Location -Path <cwd>` 啟動並切換目錄

#### Scenario: 識別 cmd

- **WHEN** 啟動器為 shell 且 terminal_path 的 file_stem 為 `cmd`
- **THEN** 以 `/K cd /d <cwd>` 啟動並切換目錄

#### Scenario: 識別 bash / sh / zsh

- **WHEN** 啟動器為 shell 且 terminal_path 的 file_stem 為 `bash`、`sh` 或 `zsh`
- **THEN** 以 `--init-file <(echo "cd <cwd>")` 或 `--rcfile` 方式啟動

#### Scenario: 未知終端類型

- **WHEN** 啟動器為 shell 且 file_stem 不在白名單內
- **THEN** 直接以 cwd 作為工作目錄啟動終端，不附加額外參數

#### Scenario: herdr 模式不套用白名單

- **WHEN** 啟動器為 herdr
- **THEN** 系統不依 `terminal_path` 的 file_stem 推導啟動參數

### Requirement: 多工具啟動指令路由

系統 SHALL 在 `open_in_tool` command 中依 tool_type 決定啟動邏輯，統一處理所有工具的啟動參數，並在決定啟動參數後依當前啟動器種類送出。

#### Scenario: terminal 類型路由

- **WHEN** open_in_tool 收到 tool_type 為 `terminal`
- **THEN** 依當前啟動器開啟終端（shell 模式套用 terminal_path + file_stem 白名單；herdr 模式建立 tab）

#### Scenario: opencode 類型路由

- **WHEN** open_in_tool 收到 tool_type 為 `opencode`
- **THEN** 在啟動器提供的終端環境中執行 `opencode --cwd <cwd>`

#### Scenario: gh-copilot 類型路由

- **WHEN** open_in_tool 收到 tool_type 為 `gh-copilot` 且 session_id 不為空
- **THEN** 在啟動器提供的終端環境中執行 `gh copilot session resume <session_id>`

#### Scenario: gemini 類型路由

- **WHEN** open_in_tool 收到 tool_type 為 `gemini`
- **THEN** 在啟動器提供的終端環境中執行 `gemini`，工作目錄設為 cwd

#### Scenario: explorer 類型路由

- **WHEN** open_in_tool 收到 tool_type 為 `explorer`
- **THEN** 直接 spawn `explorer.exe <cwd>`，不開啟終端視窗，且不受啟動器種類影響

### Requirement: Windows 終端與 CLI 啟動環境一致性

系統 SHALL 讓一般終端啟動、多工具啟動及 session resume 共用相同的 Windows MSYS stackdump 緩解環境組態，避免任一啟動入口遺漏。此要求適用於 shell 啟動器所建立的 console 程序。

#### Scenario: 開啟一般終端
- **WHEN** `open_terminal` 在 Windows 以 shell 啟動器啟動使用者設定的終端
- **THEN** 新程序套用 MSYS stackdump 緩解環境

#### Scenario: 開啟或恢復 AI coding CLI
- **WHEN** `open_in_tool` 或 `resume_session_in_terminal` 在 Windows 以 shell 啟動器啟動受支援的 AI coding CLI
- **THEN** 新程序套用與一般終端相同的 MSYS stackdump 緩解環境

#### Scenario: 啟動參數維持不變
- **WHEN** 系統套用 MSYS stackdump 緩解環境
- **THEN** 各 terminal 與 provider 原有的命令、參數、工作目錄及 Windows console creation flags 維持不變

#### Scenario: herdr 模式的環境套用範圍
- **WHEN** 啟動器為 herdr
- **THEN** 系統不需為該次啟動建立新的 Windows console 程序，因此不套用 console creation flags
