## MODIFIED Requirements

### Requirement: 設定欄位完整定義

AppSettings SHALL 包含以下欄位：

#### Scenario: 設定結構

- **WHEN** 系統讀寫 settings.json
- **THEN** 完整格式如下：
  - `copilot_root: String` — Copilot session-state 父目錄
  - `opencode_root: String` — OpenCode storage 根目錄
  - `terminal_path: String` — 終端機執行檔路徑
  - `external_editor_path: String` — 外部編輯器路徑
  - `show_archived: bool` — 是否顯示封存 session
  - `show_status_bar: bool` — 是否顯示全域底部狀態列（預設 `true`）
  - `pinned_projects: Vec<String>` — 釘選專案的 cwd 列表
  - `enabled_providers: Vec<String>` — 啟用的 provider（`"copilot"` / `"opencode"`）
  - `provider_integrations: HashMap<String, ProviderIntegration>` — provider bridge 設定
  - `default_launcher: Option<String>` — 預設啟動工具（`"terminal"` / `"opencode"` / `"gh-copilot"` / `"gemini"` / `"explorer"`）
  - `enable_intervention_notification: bool` — 是否啟用 Windows 介入通知（預設 `true`）
  - `enable_quota_monitoring: bool` — 是否啟用 quota monitoring（預設 `true`）
  - `quota_refresh_interval: u32` — quota 自動刷新間隔（分鐘）

## ADDED Requirements

### Requirement: 設定頁提供 quota monitoring 開關與刷新間隔

系統 SHALL 在設定頁提供 quota monitoring 的開關與刷新間隔設定，讓使用者控制是否顯示與更新 provider quota。

#### Scenario: 關閉 quota monitoring
- **WHEN** 使用者在設定頁關閉 quota monitoring 並儲存
- **THEN** 系統將 `enable_quota_monitoring` 寫入設定
- **AND** 停止背景 quota refresh

#### Scenario: 調整 quota refresh interval
- **WHEN** 使用者在設定頁選擇新的 quota refresh interval 並儲存
- **THEN** 系統將新設定寫入 `settings.json`
- **AND** 後續背景 quota refresh 依新間隔執行
