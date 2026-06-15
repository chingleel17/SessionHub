## MODIFIED Requirements

### Requirement: hook scripts 以嵌入方式部署

Rust provider 模組 SHALL 以 `include_str!` 嵌入 `.sh` 與 `.ps1` 腳本，並於對應的 `ensure_*_hook_scripts_installed` 時一併寫出到 hook scripts 根目錄，版本號升為 `2`。

對於 Codex 與 Copilot，hook scripts 根目錄 SHALL 為該 provider 設定根目錄下的 `hooks/`（`~/.codex/hooks/`、`~/.copilot/hooks/`），且 SHALL 與整合檔（`hooks.json` 等）中 hook 指令所引用的腳本路徑為同一來源，不得使用指向 `%APPDATA%\SessionHub` 的 bundled 目錄作為寫出位置。

#### Scenario: 安裝後 sh 腳本存在於正確路徑
- **WHEN** 呼叫 `ensure_claude_hook_scripts_installed`
- **THEN** hook scripts 目錄下包含 `on-session-start.sh` 等五個 .sh 檔案與 `modules/record-event.sh` 等三個模組

#### Scenario: Codex 安裝後腳本根目錄與整合檔指令一致
- **WHEN** 呼叫 `ensure_codex_hook_scripts_installed`
- **THEN** 腳本寫出於 `~/.codex/hooks/`
- **AND** `hooks.json` 內 hook 指令引用的腳本路徑指向同一目錄下實際存在的檔案
