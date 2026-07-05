## MODIFIED Requirements

### Requirement: 設定頁顯示 provider integration 狀態

系統 SHALL 在設定頁顯示每個已支援 provider 的 integration 狀態、設定檔位置，以及最後檢查結果。設定檔位置欄位的標籤 SHALL 依該 provider 實際的整合機制顯示對應語意的名稱，不得對所有 provider 一律使用同一個籠統標籤。

#### Scenario: 顯示 provider 狀態

- **WHEN** 使用者開啟設定頁
- **THEN** 系統顯示 Copilot、OpenCode、Codex 與 Claude Code 各自的 integration 狀態
- **AND** 顯示其設定檔、plugin 或 hook 路徑（若可解析）

#### Scenario: 重新檢查 integration 狀態

- **WHEN** 使用者點擊 provider 的「重新檢查」
- **THEN** 系統重新偵測 integration 安裝狀態
- **AND** 更新畫面中的狀態與錯誤訊息

#### Scenario: Claude Code 顯示 Hook 路徑標籤

- **WHEN** 設定頁顯示 Claude Code 的 provider integration 卡片
- **THEN** 設定檔位置欄位標籤 SHALL 顯示「Hook 路徑」（而非「設定 / plugin 路徑」）
- **AND** 欄位值為 Claude Code hooks 設定目錄（例如 `%USERPROFILE%\.claude\hooks`）

#### Scenario: 其他 provider 沿用既有標籤

- **WHEN** 設定頁顯示 Copilot、OpenCode 或 Codex 的 provider integration 卡片
- **THEN** 設定檔位置欄位標籤沿用「設定 / plugin 路徑」
