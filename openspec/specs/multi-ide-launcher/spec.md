## Requirements

### Requirement: Platform-Aware 預設啟動工具

系統 SHALL 以 session 的 provider 對應的 CLI resume 指令開啟 session，不提供其他工具選擇。

#### Scenario: 以 provider CLI resume session

- **WHEN** 使用者點擊 SessionCard 或看板卡片的開啟按鈕
- **THEN** 系統以設定的終端機（`terminalPath`，預設 `pwsh`）開啟新終端視窗，工作目錄為 session cwd
- **AND** 在終端中執行該 provider 對應的 resume 指令：
  - `claude` → `claude --resume=<session_id>`
  - `codex` → `codex resume <session_id>`
  - `copilot` → `copilot --resume=<session_id>`
  - `opencode` → `opencode session <session_id>`
- **AND** 終端以保留模式啟動（pwsh `-NoExit` / cmd `/K`），指令結束後視窗不自動關閉

#### Scenario: 不支援的 provider

- **WHEN** session 的 provider 無對應 resume 指令
- **THEN** 後端回傳錯誤，前端顯示 toast 提示不支援

#### Scenario: cwd 不存在

- **WHEN** session 的 cwd 為空或目錄不存在
- **THEN** 系統顯示 toast 提示（沿用 `toast.cwdMissing`），不開啟終端

#### Scenario: 開啟按鈕標示 provider

- **WHEN** SessionCard 渲染開啟按鈕
- **THEN** 按鈕 tooltip / aria-label SHALL 標明將以該 session 的 provider 工具開啟

### Requirement: 設定頁預設啟動工具選項

系統 SHALL 在設定頁提供「預設啟動工具」選項，作為**專案層級開啟選單**的預設行為；此設定不影響 session 的開啟方式。

#### Scenario: 設定預設工具

- **WHEN** 使用者在設定頁選擇預設啟動工具（Terminal / OpenCode / Copilot CLI / Gemini CLI / VS Code / File Explorer）
- **THEN** 系統將選擇儲存至 AppSettings.defaultLauncher
- **AND** ProjectView 的「開啟專案」主按鈕以此工具開啟專案路徑

#### Scenario: 設定儲存並生效

- **WHEN** 使用者儲存設定
- **THEN** 專案層級開啟選單立即反映新的預設啟動工具（選單中標示「預設」）
