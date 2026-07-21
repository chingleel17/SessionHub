## ADDED Requirements

### Requirement: opencode 權限請求觸發介入通知

系統 SHALL 在 opencode session 因權限請求事件（`permission.updated`、`permission.asked` 或 `permission.v2.asked`）進入 `waiting` 狀態時，透過既有的介入通知路徑發送 Windows Toast 通知，使 opencode 與其他 provider 享有一致的「需介入」通知行為。

#### Scenario: opencode 權限請求發送介入通知

- **WHEN** opencode 因 Bash、檔案讀寫、跨專案／跨目錄存取或其他受管制操作發出任一相容版本的權限請求事件
- **AND** 經 bridge 轉譯後該 session 的 `SessionActivityStatus.status` 由非 `waiting` 轉為 `waiting`
- **AND** `AppSettings.enable_intervention_notification` 為 `true`
- **THEN** 系統呼叫 `send_intervention_notification`，`notificationType` 為 `waiting`
- **AND** 通知內容包含該 opencode session 的專案名稱（取自 `cwd` 最後一段路徑）

#### Scenario: opencode 通知沿用去重與點擊聚焦

- **WHEN** opencode session 觸發 waiting 介入通知
- **THEN** 通知去重與點擊聚焦行為與其他 provider 一致（同 session 不重複通知、點擊聚焦至對應 project tab）
