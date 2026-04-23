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

## ADDED Requirements

### Requirement: 狀態列開關設定

系統 SHALL 在設定頁一般設定區塊提供開關，讓使用者啟用或停用全域狀態列。

#### Scenario: 關閉狀態列

- **WHEN** 使用者在設定頁將「顯示狀態列」切換為關閉並儲存
- **THEN** `settings.showStatusBar` 設為 `false`
- **AND** 全域狀態列立即從畫面消失

#### Scenario: 預設啟用

- **WHEN** 使用者首次安裝或 settings.json 尚未包含 `show_status_bar` 欄位
- **THEN** 系統預設 `show_status_bar` 為 `true`，狀態列顯示
