## Purpose

定義 SessionHub 的多工具啟動入口與工具可用性偵測，包含 herdr 的安裝、服務狀態及快取更新行為。

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

- **WHEN** 使用者在設定頁選擇預設啟動工具（Terminal / VS Code / File Explorer / OpenCode / Claude / Codex / Copilot CLI / Gemini CLI）
- **THEN** 系統將選擇儲存至 AppSettings.defaultLauncher
- **AND** ProjectView 的「開啟專案」主按鈕以此工具開啟專案路徑

#### Scenario: 設定儲存並生效

- **WHEN** 使用者儲存設定
- **THEN** 專案層級開啟選單立即反映新的預設啟動工具（選單中標示「預設」）

### Requirement: 專案開啟選單工具清單與順序

系統 SHALL 在專案層級「開啟專案」選單提供以下工具，並依此固定順序排列：Terminal → VS Code → Explorer → OpenCode → Claude → Codex → Copilot → Gemini。

#### Scenario: 選單依指定順序渲染

- **WHEN** 使用者展開專案頁面的「開啟專案」選單
- **THEN** 選單依序顯示 Terminal、VS Code、Explorer、OpenCode、Claude、Codex、Copilot、Gemini 共八個工具項目

#### Scenario: 以 Claude Code 開啟專案

- **WHEN** 使用者在選單選擇 Claude
- **THEN** 系統以設定的終端機（`terminalPath`，預設 `pwsh`）於專案目錄開新終端視窗
- **AND** 在終端中執行 `claude`，並以保留模式啟動（pwsh `-NoExit` / cmd `/K`）

#### Scenario: 以 Codex 開啟專案

- **WHEN** 使用者在選單選擇 Codex
- **THEN** 系統以設定的終端機（`terminalPath`，預設 `pwsh`）於專案目錄開新終端視窗
- **AND** 在終端中執行 `codex`，並以保留模式啟動（pwsh `-NoExit` / cmd `/K`）

#### Scenario: 未知工具類型

- **WHEN** 後端收到不在支援清單內的工具類型
- **THEN** 後端回傳錯誤，前端顯示 toast 提示不支援

### Requirement: 啟動工具可用性偵測涵蓋 Codex 與 Claude

系統 SHALL 在偵測工具可用性時，額外檢查 PATH 中是否存在 `codex` 與 `claude` 可執行檔，並將結果納入可用性資料。

#### Scenario: 偵測 Codex 與 Claude 是否安裝

- **WHEN** 系統執行工具可用性偵測
- **THEN** 回傳的可用性資料 SHALL 包含 `codex` 與 `claude` 的布林值，分別代表兩者是否存在於 PATH

#### Scenario: 選單依可用性標示

- **WHEN** 「開啟專案」選單渲染 Claude 或 Codex 項目
- **AND** 對應工具不存在於 PATH
- **THEN** 選單依既有可用性標示規則標記該項目為不可用

### Requirement: herdr 可用性偵測

系統 SHALL 在偵測工具可用性時，額外檢查 PATH 中是否存在 `herdr` 可執行檔，並將結果納入可用性資料。

#### Scenario: 偵測 herdr 是否安裝

- **WHEN** 系統執行工具可用性偵測
- **THEN** 回傳的可用性資料 SHALL 包含 `herdr` 布林值，代表其是否存在於 PATH

#### Scenario: 未安裝時的可用性結果

- **WHEN** PATH 中不存在 `herdr` 可執行檔
- **THEN** 可用性資料中的 `herdr` 為 false

### Requirement: herdr 服務狀態偵測

系統 SHALL 區分「herdr 未安裝」與「herdr 已安裝但服務未執行」兩種狀態，因兩者的補救方式不同。

#### Scenario: 已安裝且服務執行中

- **WHEN** `herdr` 存在於 PATH 且查詢服務狀態回報執行中
- **THEN** 系統視 herdr 為可正常使用

#### Scenario: 已安裝但服務未執行

- **WHEN** `herdr` 存在於 PATH 但查詢服務狀態未回報執行中
- **THEN** 系統回報「已安裝但服務未執行」狀態
- **AND** 相關提示訊息指引使用者啟動 herdr，而非重新安裝

#### Scenario: 未安裝

- **WHEN** `herdr` 不存在於 PATH
- **THEN** 系統回報「未偵測到」狀態
- **AND** 相關提示訊息指引使用者安裝 herdr

### Requirement: herdr 可用性快取更新時機

系統 SHALL 讓使用者在安裝或啟動 herdr 後，無需重啟應用程式即可讓設定頁反映最新可用性。

#### Scenario: 重新偵測 herdr

- **WHEN** 使用者於設定頁觸發重新偵測
- **THEN** 系統重新查詢 herdr 可用性與服務狀態
- **AND** 設定頁依最新結果更新 launcher 選項的可用狀態

#### Scenario: 儲存設定後更新可用性

- **WHEN** 使用者儲存設定
- **THEN** 系統使工具可用性快取失效，使後續讀取取得最新結果
