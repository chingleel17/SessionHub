## ADDED Requirements

### Requirement: Hook 腳本安裝路徑與整合檔指令一致

系統在安裝或更新 Codex / Copilot integration 時，寫出 hook 腳本檔案（`.sh` 與 `.ps1`）的根目錄 SHALL 與寫入 `hooks.json`（或對應 integration 檔）內 hook 指令所引用的腳本路徑為同一目錄，且兩者 SHALL 由同一個解析函式提供，避免路徑分歧。

該腳本根目錄 SHALL 為 provider 設定根目錄下的 `hooks/`（Codex 為 `~/.codex/hooks/`，Copilot 為 `~/.copilot/hooks/`），而非 `%APPDATA%\SessionHub` 下的 bundled 目錄。

#### Scenario: 安裝後腳本存在於整合檔所引用的路徑
- **WHEN** 使用者安裝 Codex integration
- **THEN** `hooks.json` 中每個 hook 指令引用的 `.sh` / `.ps1` 路徑實際存在對應檔案
- **AND** Codex 觸發 hook 時不會因找不到腳本而以非零碼結束

#### Scenario: 安裝與寫入 config 使用同一解析來源
- **WHEN** `ensure_codex_hook_scripts_installed` 寫出腳本，且 `render_codex_integration` 產生指令
- **THEN** 兩者解析出的 hook 腳本根目錄為相同絕對路徑

### Requirement: 清除舊版內嵌 hook group

系統在安裝或更新 Codex integration 時 SHALL 移除既有的 SessionHub hook group，包含新版以 `# sessionhub-provider-event-bridge` 標記的 group，以及舊版 v4 PowerShell 內嵌指令 group（特徵為 `commandWindows` 內含 `provider = 'codex'` 之類的內嵌記錄指令），避免同一事件下新舊 group 並存。

#### Scenario: 升級時清除舊版內嵌 group
- **WHEN** `hooks.json` 中既有舊版 v4 內嵌 PowerShell hook group
- **AND** 使用者重新安裝或更新 Codex integration
- **THEN** 每個事件下只保留一個 SessionHub hook group
- **AND** 舊版內嵌 group 被移除

#### Scenario: 重複安裝維持冪等
- **WHEN** 對同一 `hooks.json` 連續執行兩次安裝
- **THEN** 每個事件下仍只有一個 SessionHub hook group
