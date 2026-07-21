## ADDED Requirements

### Requirement: Dashboard 依啟用清單過濾 quota 顯示

系統 SHALL 讓 Dashboard 的 Quota 卡片（`QuotaOverview`）只顯示使用者於設定頁 `quotaEnabledProviders` 中勾選啟用的 provider，行為 SHALL 與 StatusBar 的 quota chip 顯示範圍一致。

#### Scenario: 使用者停用某 provider 的 quota 監控

- **WHEN** 使用者在設定頁取消勾選某個 provider（例如 OpenCode）的 quota 監控並儲存
- **THEN** Dashboard 的 Quota 卡片 SHALL 不再顯示該 provider 的用量資訊
- **AND** 即使後端快取或資料庫中仍留有該 provider 過去的 snapshot 資料

#### Scenario: 重新開啟設定頁或 Dashboard 後仍保持過濾

- **WHEN** 使用者停用某 provider 的 quota 監控後，重新導覽至設定頁或 Dashboard
- **THEN** 該 provider SHALL 不會重新出現在 Dashboard 的 Quota 卡片中
- **AND** 直到使用者重新勾選啟用該 provider 為止

#### Scenario: 使用者重新啟用某 provider 的 quota 監控

- **WHEN** 使用者重新勾選啟用某 provider 的 quota 監控並儲存
- **THEN** Dashboard 的 Quota 卡片 SHALL 在下次刷新後顯示該 provider 的用量資訊

### Requirement: 儲存設定時清除已停用 provider 的 quota 快取

系統 SHALL 在使用者儲存設定時，依最新的 `quotaEnabledProviders` 清單清除記憶體快取與資料庫中已停用 provider 的 quota snapshot，而不僅限於使用者手動觸發全量「重新整理」時才清除。

#### Scenario: 儲存設定觸發快取清除

- **WHEN** 使用者將某 provider 從 `quotaEnabledProviders` 移除並儲存設定
- **THEN** 後端 SHALL 立即從記憶體快取與資料庫移除該 provider 的既有 quota snapshot
- **AND** 後續 `get_quota_snapshots` 呼叫不再回傳該 provider 的資料

#### Scenario: 應用程式啟動時載入快取遵循目前啟用清單

- **WHEN** 應用程式啟動並從資料庫載入既有 quota snapshot 至記憶體快取
- **THEN** 系統 SHALL 排除目前 `quotaEnabledProviders` 中未啟用的 provider 資料
