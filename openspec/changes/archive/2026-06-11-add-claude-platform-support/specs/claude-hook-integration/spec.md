## ADDED Requirements

### Requirement: Claude hook 設定注入

系統 SHALL 能偵測 `~/.claude/settings.json` 是否已包含 SessionHub 的 stop hook 設定，並允許使用者從 Settings 頁面一鍵安裝或更新；安裝時以 merge 策略寫入，不覆蓋使用者現有的 hooks 設定。

#### Scenario: 安裝 Claude hook integration

- **WHEN** 使用者在 Settings 頁對 Claude 點擊「安裝整合」
- **THEN** 系統讀取現有的 `~/.claude/settings.json`（若不存在則以空物件起始）
- **AND** 在 `hooks.Stop` 陣列中 append SessionHub bridge 寫入命令
- **AND** 寫回 `~/.claude/settings.json`
- **AND** 整合狀態更新為 `installed`

#### Scenario: 已安裝時顯示已安裝狀態

- **WHEN** 使用者進入 Settings 頁
- **AND** `~/.claude/settings.json` 的 `hooks.Stop` 已包含 SessionHub 的 hook entry
- **THEN** 顯示整合狀態為 `installed`
- **AND** 提供「重新安裝」而非「安裝整合」按鈕

#### Scenario: settings.json 不可寫入

- **WHEN** SessionHub 無法寫入 `~/.claude/settings.json`
- **THEN** 整合狀態標示為 `manual_required`
- **AND** 顯示手動設定說明，包含需加入的 JSON 片段

#### Scenario: hook 觸發後 SessionHub 接收 bridge 事件

- **WHEN** Claude Code CLI session 結束，stop hook 執行
- **THEN** SessionHub bridge 檔案收到包含 `provider=claude`、`eventType=session.stop`、`sessionId`、`cwd` 的事件
- **AND** SessionHub 對應 session 觸發 targeted refresh

### Requirement: Claude hook bridge 事件格式

Claude stop hook 寫入的 bridge record SHALL 符合現有的 bridge 事件標準格式，包含 `provider`、`eventType`、`timestamp`，並在可取得時包含 `sessionId` 與 `cwd`。

#### Scenario: bridge 事件包含完整欄位

- **WHEN** Claude Code CLI stop hook 執行
- **THEN** bridge JSON 包含 `provider: "claude"`、`eventType: "session.stop"`
- **AND** `timestamp` 為 ISO 8601 格式
- **AND** `sessionId` 對應 JSONL 檔案名稱（不含副檔名）
