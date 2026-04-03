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

## ADDED Requirements

### Requirement: 預設啟動工具設定

系統 SHALL 在 AppSettings 新增 `default_launcher` 欄位，讓使用者選擇點擊 SessionCard 主啟動按鈕時的預設工具。

#### Scenario: 讀取預設啟動工具

- **WHEN** 系統載入設定
- **THEN** `default_launcher` 欄位為 None 時，系統預設使用 `terminal`

#### Scenario: 儲存預設啟動工具

- **WHEN** 使用者在設定頁選擇預設啟動工具並儲存
- **THEN** 系統將選擇的工具類型字串寫入 settings.json 的 `default_launcher` 欄位
