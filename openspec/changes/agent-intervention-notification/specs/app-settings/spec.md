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
  - `pinned_projects: Vec<String>` — 釘選專案的 cwd 列表
  - `enabled_providers: Vec<String>` — 啟用的 provider（`"copilot"` / `"opencode"`）
  - `provider_integrations: HashMap<String, ProviderIntegration>` — provider bridge 設定
  - `default_launcher: Option<String>` — 預設啟動工具（`"terminal"` / `"opencode"` / `"gh-copilot"` / `"gemini"` / `"explorer"`）
  - `enable_intervention_notification: bool` — 是否啟用 Windows 介入通知（預設 `true`）

## ADDED Requirements

### Requirement: 介入通知設定開關

系統 SHALL 在設定頁提供「介入通知」開關，讓使用者控制是否接收 Windows Toast 通知。

#### Scenario: 開啟通知設定

- **WHEN** 使用者在設定頁將「介入通知」toggle 切換為啟用並儲存
- **THEN** 系統將 `enable_intervention_notification: true` 寫入 settings.json
- **AND** 後續 session 進入 `waiting` 狀態時發送 Toast 通知

#### Scenario: 關閉通知設定

- **WHEN** 使用者在設定頁將「介入通知」toggle 切換為停用並儲存
- **THEN** 系統將 `enable_intervention_notification: false` 寫入 settings.json
- **AND** 後續即使 session 進入 `waiting` 狀態也不發送通知

#### Scenario: 設定預設值

- **WHEN** settings.json 不存在 `enable_intervention_notification` 欄位（舊版升級）
- **THEN** 系統將其視為 `true`（預設啟用通知）
