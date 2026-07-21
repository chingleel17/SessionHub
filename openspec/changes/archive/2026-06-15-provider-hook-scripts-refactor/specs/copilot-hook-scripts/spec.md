## ADDED Requirements

### Requirement: Copilot CLI 獨立 hook 腳本集（含 sh 版本）

SessionHub SHALL 提供 Copilot CLI 專用的獨立 hook 腳本（`.ps1` for Windows、`.sh` for Unix），涵蓋 6 個事件，並在應用程式啟動時自動安裝至 `~/.copilot/hooks/`。

#### Scenario: 啟動時自動安裝 Copilot hook 腳本

- **WHEN** SessionHub 應用程式啟動
- **THEN** 系統將 Copilot hook 腳本寫入 `%APPDATA%\SessionHub\.copilot\hooks\`（bundled 路徑）
- **AND** 寫入失敗時記錄錯誤但不中斷啟動

#### Scenario: Copilot 整合安裝時引用腳本路徑

- **WHEN** 使用者安裝 Copilot integration
- **THEN** 寫入 `~/.copilot/settings.json` 的 hook command 為腳本呼叫（`powershell` 欄位改呼叫腳本路徑，新增 `command` 欄位呼叫 sh 腳本）
- **AND** 不再將 PowerShell 邏輯內嵌為字串

#### Scenario: Copilot hook 涵蓋 6 個事件

- **WHEN** Copilot hook 腳本集安裝完成
- **THEN** 存在以下腳本：`on-session-start.ps1`、`on-session-start.sh`、`on-session-end.ps1`、`on-session-end.sh`、`on-user-prompt-submitted.ps1`、`on-user-prompt-submitted.sh`、`on-pre-tool-use.ps1`、`on-pre-tool-use.sh`、`on-post-tool-use.ps1`、`on-post-tool-use.sh`、`on-error-occurred.ps1`、`on-error-occurred.sh`
- **AND** 每個腳本均可獨立執行，接受 `--bridge-path` 與 `--provider` 參數

#### Scenario: Copilot hook 腳本讀取 stdin payload

- **WHEN** Copilot CLI 呼叫 hook 並透過 stdin 傳入 JSON payload
- **THEN** 腳本讀取 stdin；若 payload 為空白，以 exit 0 立即結束

#### Scenario: sessionStart 事件

- **WHEN** Copilot CLI 觸發 `sessionStart` hook
- **THEN** 腳本寫入 bridge record，eventType 為 `session.started`，timestamp 從 payload `timestamp`（毫秒）轉換

#### Scenario: sessionEnd 事件含 error 判斷

- **WHEN** Copilot CLI 觸發 `sessionEnd` hook，且 payload `reason` 為 `error`
- **THEN** 腳本寫入 bridge record，eventType 為 `session.ended`，error 欄位填入 `copilot session ended with error`
- **WHEN** `reason` 非 `error`
- **THEN** error 欄位為 null

#### Scenario: userPromptSubmitted 事件截斷 prompt

- **WHEN** Copilot CLI 觸發 `userPromptSubmitted` hook
- **THEN** 腳本寫入 bridge record，eventType 為 `prompt.submitted`，title 欄位為 `prompt` 欄位前 80 字元

#### Scenario: preToolUse 與 postToolUse 事件

- **WHEN** Copilot CLI 觸發 `preToolUse` 或 `postToolUse` hook
- **THEN** 腳本寫入 bridge record，eventType 分別為 `tool.pre`、`tool.post`，title 為 `toolName` 欄位
- **AND** postToolUse 腳本 SHALL 在 `toolResult.resultType` 為 `failure` 或 `denied` 時填入 error 欄位

#### Scenario: errorOccurred 事件

- **WHEN** Copilot CLI 觸發 `errorOccurred` hook
- **THEN** 腳本寫入 bridge record，eventType 為 `session.errored`，error 欄位為 `error.message`

### Requirement: Copilot hook 腳本共用模組

Copilot hook 腳本 SHALL 共用 `record-event.sh` 與 `record-event.psm1` 模組，安裝時將 `modules/` 目錄一併複製至 `~/.copilot/hooks/modules/`。

#### Scenario: 模組與腳本同目錄安裝

- **WHEN** Copilot hook 腳本安裝完成
- **THEN** `~/.copilot/hooks/modules/record-event.sh` 與 `record-event.psm1` 均存在
- **AND** 腳本以相對路徑引用模組
