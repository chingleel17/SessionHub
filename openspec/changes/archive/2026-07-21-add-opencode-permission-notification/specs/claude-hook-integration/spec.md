## MODIFIED Requirements

### Requirement: Claude hook 設定注入

系統 SHALL 能偵測 `~/.claude/settings.json` 是否已包含 SessionHub 管理的所有 Claude hook，包括 `SessionStart`、`PreToolUse`、`PostToolUse`、`UserPromptSubmit`、`Stop`、`Notification` 與 `PermissionRequest`，並允許使用者從 Settings 頁面一鍵安裝或更新；安裝時以 merge 策略寫入，不覆蓋使用者現有的 hooks 設定。

#### Scenario: 安裝 Claude hook integration

- **WHEN** 使用者在 Settings 頁對 Claude 點擊「安裝整合」
- **THEN** 系統讀取現有的 `~/.claude/settings.json`（若不存在則以空物件起始）
- **AND** 在各 managed event 陣列中 append 帶 SessionHub marker 的 bridge hook 命令
- **AND** 寫回 `~/.claude/settings.json`
- **AND** 整合狀態更新為 `installed`

#### Scenario: 已完整安裝時顯示已安裝狀態

- **WHEN** 使用者進入 Settings 頁
- **AND** `~/.claude/settings.json` 已包含所有 SessionHub managed hooks
- **THEN** 顯示整合狀態為 `installed`
- **AND** 提供「重新安裝」而非「安裝整合」按鈕

#### Scenario: settings.json 不可寫入

- **WHEN** SessionHub 無法寫入 `~/.claude/settings.json`
- **THEN** 整合狀態標示為 `manual_required`
- **AND** 顯示手動設定說明，包含需加入的 JSON 片段

#### Scenario: hook 觸發後 SessionHub 接收 bridge 事件

- **WHEN** Claude Code CLI session 結束，Stop hook 執行
- **THEN** SessionHub bridge 檔案收到包含 `provider=claude`、`eventType=session.stop`、`sessionId`、`cwd` 的事件
- **AND** SessionHub 對應 session 觸發 targeted refresh

#### Scenario: 安裝 Claude PermissionRequest hook

- **WHEN** 使用者在 Settings 頁安裝或更新 Claude integration
- **THEN** 系統在 `hooks.PermissionRequest` 陣列加入帶 SessionHub marker 的 managed hook group
- **AND** hook 執行 `on-permission-request.cjs`
- **AND** `Notification` managed hook matcher 保留 `permission_prompt|idle_prompt`

#### Scenario: 缺少 PermissionRequest 時判定 outdated

- **WHEN** 既有 Claude integration 包含其他 managed hooks，但缺少 `PermissionRequest`
- **THEN** 整合狀態不得顯示為 installed
- **AND** 應提示使用者更新整合
