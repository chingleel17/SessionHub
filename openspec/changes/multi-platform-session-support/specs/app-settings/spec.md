## MODIFIED Requirements

### Requirement: 設定持久化
系統 SHALL 將應用程式設定儲存於 `%APPDATA%\SessionHub\settings.json`，應用程式重啟後設定保持不變。新增 `opencodeRoot`（預設 `~/.local/share/opencode/`）與 `enabledProviders`（預設 `["copilot", "opencode"]`）欄位。

#### Scenario: 儲存設定
- **WHEN** 使用者修改設定並點擊儲存
- **THEN** 系統將設定寫入 `settings.json`，包含 `opencodeRoot` 與 `enabledProviders` 欄位

#### Scenario: 讀取不含新欄位的舊設定檔
- **WHEN** 應用程式讀取不含 `opencodeRoot` 或 `enabledProviders` 的既有 `settings.json`
- **THEN** 系統使用預設值（`opencodeRoot` = `~/.local/share/opencode/`、`enabledProviders` = `["copilot", "opencode"]`），不產生錯誤

## ADDED Requirements

### Requirement: OpenCode 路徑設定
系統 SHALL 在設定頁面提供 OpenCode 根目錄路徑的設定欄位。

#### Scenario: 修改 OpenCode 路徑
- **WHEN** 使用者在設定頁面修改 OpenCode 路徑並儲存
- **THEN** 系統更新 `opencodeRoot` 設定，下次掃描 session 時使用新路徑

#### Scenario: 驗證 OpenCode 路徑
- **WHEN** 使用者輸入 OpenCode 路徑
- **THEN** 系統檢查該路徑下是否存在 `opencode.db`，並顯示驗證結果

### Requirement: 啟用平台設定
系統 SHALL 在設定頁面提供各平台的啟用 checkbox。

#### Scenario: 停用 OpenCode provider
- **WHEN** 使用者在設定中取消勾選 OpenCode
- **THEN** `enabledProviders` 移除 `"opencode"`，session 列表不再包含 OpenCode session
