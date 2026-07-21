## MODIFIED Requirements

### Requirement: 依事件語意觸發通知

系統 SHALL 由各 provider hook 在「完成」「等待回應／需決策」「需授權」事件點直接觸發通知，不依賴 SessionHub 應用內的 activity 狀態判定；Claude 的工具授權 SHALL 以 `PermissionRequest` 為主要訊號，並以 `Notification.notification_type` 為相容備援。

#### Scenario: Copilot 觸發點

- **WHEN** Copilot `on-session-end` 執行
- **THEN** 發送「完成」類通知
- **WHEN** Copilot `on-pre-tool-use` 執行（即將執行工具、可能需授權）
- **THEN** 發送「需授權」類通知

#### Scenario: Codex 觸發點

- **WHEN** Codex `on-stop` 執行
- **THEN** 發送「完成」類通知
- **WHEN** Codex `PermissionRequest` 事件觸發
- **THEN** 發送「需授權」類通知

#### Scenario: Claude 完成觸發點

- **WHEN** Claude `Stop` hook 觸發
- **THEN** 發送「完成」類通知

#### Scenario: Claude 工具授權觸發點

- **WHEN** Claude `PermissionRequest` hook 因 Bash、Read、Edit、Write、跨專案／跨目錄存取或其他工具權限觸發
- **THEN** 發送「需授權」類通知
- **AND** 單純 `PreToolUse` 不得被視為需授權

#### Scenario: Claude Notification 備援觸發點

- **WHEN** Claude `Notification` hook payload 的 `notification_type` 為 `permission_prompt`
- **THEN** 發送「需授權」類通知
- **WHEN** `notification_type` 為 `idle_prompt`
- **THEN** 發送「等待回應」類通知
