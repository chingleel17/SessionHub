## ADDED Requirements

### Requirement: Codex 獨立 hook 腳本集

SessionHub SHALL 提供 Codex 專用的獨立 hook 腳本（`.ps1` for Windows、`.sh` for Unix），涵蓋 5 個事件，並在應用程式啟動時自動安裝至 `~/.codex/hooks/`。

#### Scenario: 啟動時自動安裝 Codex hook 腳本

- **WHEN** SessionHub 應用程式啟動
- **THEN** 系統將 Codex hook 腳本寫入 `%APPDATA%\SessionHub\.codex\hooks\`（bundled 路徑）
- **AND** 寫入失敗時記錄錯誤但不中斷啟動

#### Scenario: Codex 整合安裝時引用腳本路徑

- **WHEN** 使用者安裝 Codex integration
- **THEN** 寫入 `~/.codex/hooks.json` 的 hook command 為 `sh /path/to/on-session-start.sh --bridge-path ...`（Unix）或 `pwsh -File on-session-start.ps1 -BridgePath ...`（Windows）
- **AND** 不再將 PowerShell 邏輯內嵌為字串

#### Scenario: Codex hook 涵蓋 5 個事件

- **WHEN** Codex hook 腳本集安裝完成
- **THEN** 存在以下腳本：`on-session-start.ps1`、`on-session-start.sh`、`on-pre-tool-use.ps1`、`on-pre-tool-use.sh`、`on-post-tool-use.ps1`、`on-post-tool-use.sh`、`on-user-prompt-submit.ps1`、`on-user-prompt-submit.sh`、`on-stop.ps1`、`on-stop.sh`
- **AND** 每個腳本均可獨立執行，接受 `--bridge-path` 與 `--provider` 參數

#### Scenario: Codex hook 腳本讀取 stdin payload

- **WHEN** Codex 呼叫 hook 並透過 stdin 傳入 JSON payload
- **THEN** 腳本讀取 stdin 並解析 `session_id`、`cwd`、`transcript_path` 欄位
- **AND** 若 stdin 為空或 payload 為空白，腳本以 exit 0 立即結束，不 block

#### Scenario: SessionStart 事件對應 source 欄位

- **WHEN** Codex 觸發 `SessionStart` hook（source 為 `startup`、`resume`、`clear` 或 `compact`）
- **THEN** 腳本寫入 bridge record，eventType 為 `session.started`，title 欄位填入 `source` 值

#### Scenario: Stop 事件對應 stop_reason 欄位

- **WHEN** Codex 觸發 `Stop` hook
- **THEN** 腳本寫入 bridge record，eventType 為 `session.stop`，title 欄位填入 `stop_reason`（若存在）

#### Scenario: PreToolUse 與 PostToolUse 事件對應 tool_name 欄位

- **WHEN** Codex 觸發 `PreToolUse` 或 `PostToolUse` hook
- **THEN** 腳本寫入 bridge record，eventType 分別為 `tool.pre`、`tool.post`，title 欄位填入 `tool_name`

#### Scenario: UserPromptSubmit 事件對應 prompt 欄位

- **WHEN** Codex 觸發 `UserPromptSubmit` hook
- **THEN** 腳本寫入 bridge record，eventType 為 `prompt.submitted`，title 欄位填入 `prompt` 的前 80 字元

#### Scenario: Codex hook 設定檔路徑

- **WHEN** 系統安裝 Codex integration
- **THEN** hook 設定寫入 `~/.codex/hooks.json`（非 `config.json`）
- **AND** 格式為 `{ "hooks": { "EventName": [ { "hooks": [{ "type": "command", "command": "...", "commandWindows": "..." }] } ] } }`

### Requirement: Codex hook 腳本共用模組

Codex hook 腳本 SHALL 共用 `record-event.sh` 與 `record-event.psm1` 模組，安裝時將 `modules/` 目錄一併複製至 `~/.codex/hooks/modules/`。

#### Scenario: 模組與腳本同目錄安裝

- **WHEN** Codex hook 腳本安裝完成
- **THEN** `~/.codex/hooks/modules/record-event.sh` 與 `record-event.psm1` 均存在
- **AND** 腳本以相對路徑 `"$SCRIPT_DIR/modules/record-event.sh"` 引用模組
