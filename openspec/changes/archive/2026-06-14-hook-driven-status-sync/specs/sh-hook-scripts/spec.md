## ADDED Requirements

### Requirement: sh 版本 hook 模組結構
`.claude/hooks/modules/` 下 SHALL 新增三個 `.sh` 模組檔案，對應現有 `.psm1`：
- `record-event.sh`：提供 `write_bridge_event_record`、`read_hook_payload`、`get_hook_string_value` 函式
- `db-ops.sh`：提供 `invoke_with_retry` 函式
- `task-queue.sh`：（輕量佔位，供後續擴充）

#### Scenario: 模組可被 source 引入
- **WHEN** 入口腳本執行 `source "$SCRIPT_DIR/modules/record-event.sh"`
- **THEN** `write_bridge_event_record` 函式可被呼叫，不報錯

#### Scenario: jq 不存在時模組輸出錯誤並安全退出
- **WHEN** sh 腳本執行環境中 `jq` 不可用（`command -v jq` 失敗）
- **THEN** 腳本輸出錯誤訊息至 stderr，以 exit 0 結束（不阻斷 Claude 工作流程）

### Requirement: sh 版本入口腳本
`.claude/hooks/` 下 SHALL 新增五個 `.sh` 入口腳本，功能與現有 `.ps1` 一一對應：
`on-session-start.sh`、`on-pre-tool-use.sh`、`on-post-tool-use.sh`、`on-user-prompt-submit.sh`、`on-stop.sh`

每支腳本 SHALL：
- 接受 `--bridge-path` 與 `--provider` 參數
- 從 stdin 讀取 JSON payload
- 呼叫 `write_bridge_event_record` 寫入 bridge JSONL

#### Scenario: on-session-start.sh 正確寫入事件
- **WHEN** 腳本以有效 JSON payload 透過 stdin 呼叫，並提供合法 `--bridge-path`
- **THEN** bridge 檔案中新增一行 JSON，包含 `eventType="session.started"` 和正確的 `sessionId`、`cwd`

#### Scenario: on-stop.sh 正確寫入停止事件
- **WHEN** 腳本以含 `stop_reason` 的 JSON payload 呼叫
- **THEN** bridge 檔案中新增一行 JSON，包含 `eventType="session.stop"`

### Requirement: claude.rs 同時產生 command 與 commandWindows
`managed_hook_group` 產生的 hook JSON SHALL 同時包含：
- `command`：使用 sh 腳本路徑（`sh "<path>/on-xxx.sh" --bridge-path "<path>" --provider claude`）
- `commandWindows`：使用 ps1 腳本路徑（維持現有 pwsh 指令格式）

#### Scenario: 重新安裝後 hook group 含兩個欄位
- **WHEN** 呼叫 `install_or_update_claude_integration`
- **THEN** Claude settings.json 中每個 hook group 的 `hooks[0]` 同時有 `command` 與 `commandWindows`

#### Scenario: 重複安裝不產生重複 hook group
- **WHEN** 對同一 Claude settings.json 執行兩次安裝
- **THEN** 每個事件下仍只有一個 SessionHub hook group（冪等）

### Requirement: hook scripts 以嵌入方式部署
Rust `claude.rs` SHALL 以 `include_str!` 嵌入 `.sh` 腳本，並於 `ensure_claude_hook_scripts_installed` 時一併寫出到 hook scripts 目錄，版本號升為 `2`。

#### Scenario: 安裝後 sh 腳本存在於正確路徑
- **WHEN** 呼叫 `ensure_claude_hook_scripts_installed`
- **THEN** hook scripts 目錄下包含 `on-session-start.sh` 等五個 .sh 檔案與 `modules/record-event.sh` 等三個模組
