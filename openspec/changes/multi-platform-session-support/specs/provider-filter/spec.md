## ADDED Requirements

### Requirement: 平台篩選 UI
系統 SHALL 在 session 列表上方提供平台篩選控制項，讓使用者選擇要顯示哪些平台的 session。

#### Scenario: 預設顯示所有平台
- **WHEN** 使用者首次開啟應用程式
- **THEN** 所有平台（Copilot、OpenCode）的篩選 checkbox 皆為勾選狀態

#### Scenario: 取消勾選某平台
- **WHEN** 使用者取消勾選 OpenCode 的 checkbox
- **THEN** session 列表僅顯示 Copilot 來源的 session，OpenCode session 被隱藏

#### Scenario: 篩選狀態持久化
- **WHEN** 使用者修改平台篩選後關閉應用程式並重新開啟
- **THEN** 篩選狀態與上次關閉時相同

### Requirement: 篩選邏輯後端整合
系統 SHALL 將 `enabledProviders` 設定傳遞至後端，由 Rust 端在掃描時篩選，避免傳送使用者未啟用平台的資料。

#### Scenario: 僅啟用 Copilot
- **WHEN** `enabledProviders` 為 `["copilot"]`
- **THEN** `get_sessions` 命令僅掃描 Copilot session-state 目錄，不查詢 OpenCode 資料庫

#### Scenario: 全部啟用
- **WHEN** `enabledProviders` 為 `["copilot", "opencode"]`
- **THEN** `get_sessions` 命令同時掃描 Copilot 與查詢 OpenCode，合併結果回傳
