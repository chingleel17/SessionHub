# sh-hook-scripts Specification

## Purpose
TBD - created by archiving change hook-driven-status-sync. Update Purpose after archive.
## Requirements
### Requirement: sh 版本 hook 模組結構（已降級為 fallback）

> **Note**: sh 由主軌降為 fallback，主邏輯改由 `node-hook-scripts` 的 `record-event.js` 承載；sh 模組行為維持現狀但不再是受規範的主結構。主軌邏輯參見 `node-hook-scripts` 規格的「共用 record-event.js 模組」需求；既有 `record-event.sh` 等 sh 模組檔案保留作 fallback，無需移除。

`.claude/hooks/modules/` 下 MAY 提供 `.sh` 模組檔案作為 fallback：
- `record-event.sh`：提供 `write_bridge_event_record`、`read_hook_payload`、`get_hook_string_value` 函式
- `db-ops.sh`：提供 `invoke_with_retry` 函式
- `task-queue.sh`：（輕量佔位，供後續擴充）

#### Scenario: 模組可被 source 引入
- **WHEN** 入口腳本執行 `source "$SCRIPT_DIR/modules/record-event.sh"`
- **THEN** `write_bridge_event_record` 函式可被呼叫，不報錯

#### Scenario: jq 不存在時模組輸出錯誤並安全退出
- **WHEN** sh 腳本執行環境中 `jq` 不可用（`command -v jq` 失敗）
- **THEN** 腳本輸出錯誤訊息至 stderr，以 exit 0 結束（不阻斷 Claude 工作流程）

### Requirement: sh 版本入口腳本（已降級為 fallback）

> **Note**: sh 入口腳本由主軌降為 fallback，主軌入口改由 `node-hook-scripts` 的 `.js` 入口腳本定義。既有 `.sh` 入口腳本保留作 fallback。

`.claude/hooks/` 下 MAY 提供五個 `.sh` 入口腳本作為 fallback：
`on-session-start.sh`、`on-pre-tool-use.sh`、`on-post-tool-use.sh`、`on-user-prompt-submit.sh`、`on-stop.sh`

#### Scenario: on-session-start.sh 正確寫入事件
- **WHEN** 腳本以有效 JSON payload 透過 stdin 呼叫，並提供合法 `--bridge-path`
- **THEN** bridge 檔案中新增一行 JSON，包含 `eventType="session.started"` 和正確的 `sessionId`、`cwd`

#### Scenario: on-stop.sh 正確寫入停止事件
- **WHEN** 腳本以含 `stop_reason` 的 JSON payload 呼叫
- **THEN** bridge 檔案中新增一行 JSON，包含 `eventType="session.stop"`

### Requirement: claude.rs 同時產生 command 與 commandWindows

`managed_hook_group` 產生的 hook JSON SHALL 以 `node <script.js>` 作為主軌命令，並以 sh 腳本作為 fallback；SHALL NOT 再產生 PowerShell / `commandWindows` 命令：
- 主軌命令使用 `.js` 入口腳本（`node "<path>/on-xxx.js" --bridge-path "<path>" --provider claude`）
- fallback 使用 sh 腳本路徑（`sh "<path>/on-xxx.sh" --bridge-path "<path>" --provider claude`）

#### Scenario: 重新安裝後 hook group 主軌為 node

- **WHEN** 呼叫 `install_or_update_claude_integration`
- **THEN** Claude settings.json 中每個 hook group 的主命令指向 `node <script.js>`
- **AND** 不含任何 PowerShell / `commandWindows` 欄位

#### Scenario: hook group 保留 sh fallback

- **WHEN** 呼叫 `install_or_update_claude_integration`
- **THEN** 每個 hook group 仍包含指向 `.sh` 腳本的 fallback 命令欄位

#### Scenario: 重複安裝不產生重複 hook group

- **WHEN** 對同一 Claude settings.json 執行兩次安裝
- **THEN** 每個事件下仍只有一個 SessionHub hook group（冪等）

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

