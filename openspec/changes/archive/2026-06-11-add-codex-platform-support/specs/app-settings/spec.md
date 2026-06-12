## MODIFIED Requirements

### Requirement: 設定欄位完整定義

AppSettings SHALL 包含以下欄位：

#### Scenario: 設定結構

- **WHEN** 系統讀寫 settings.json
- **THEN** 完整格式如下：
  - `copilot_root: String` — Copilot session-state 父目錄
  - `opencode_root: String` — OpenCode storage 根目錄
  - `codex_root: String` — Codex 根目錄（其下包含 `sessions/`）
  - `terminal_path: String` — 終端機執行檔路徑
  - `external_editor_path: String` — 外部編輯器路徑
  - `show_archived: bool` — 是否顯示封存 session
  - `show_status_bar: bool` — 是否顯示全域底部狀態列（預設 `true`）
  - `pinned_projects: Vec<String>` — 釘選專案的 cwd 列表
  - `enabled_providers: Vec<String>` — 啟用的 provider（`"copilot"` / `"opencode"` / `"codex"`）
  - `provider_integrations: HashMap<String, ProviderIntegration>` — provider bridge 設定
  - `default_launcher: Option<String>` — 預設啟動工具（`"terminal"` / `"opencode"` / `"gh-copilot"` / `"gemini"` / `"explorer"`）
  - `enable_intervention_notification: bool` — 是否啟用 Windows 介入通知（預設 `true`）

### Requirement: 設定對話框權限判斷

系統 SHALL 在設定對話框開啟圖標點擊時随即執行權限判斷。

#### Scenario: 設定對話框開啟

- **WHEN** 使用者點擊開啟設定對話框
- **THEN** 系統桌點檢查目前設定值是否完整
- **AND** 若 `copilotRoot`、`opencodeRoot` 或 `codexRoot` 未設定，自動嘗試從環境變數推導預設路徑

## ADDED Requirements

### Requirement: codex_root 自動填入

系統 SHALL 自動偵測 Codex 的預設 session 根目錄並填入設定。

#### Scenario: 預設路徑自動偵測

- **WHEN** 使用者開啟設定對話框時 `codex_root` 為空
- **THEN** 系統自動嘗試 `%USERPROFILE%\.codex` 路徑
- **AND** 若該目錄存在，將其填入輸入框

### Requirement: 設定頁可啟用或停用 Codex provider

系統 SHALL 在設定頁的 provider 啟用區塊提供 Codex 選項。

#### Scenario: 啟用 Codex

- **WHEN** 使用者勾選 Codex provider 並儲存設定
- **THEN** `enabled_providers` 會包含 `"codex"`

#### Scenario: 停用 Codex

- **WHEN** 使用者取消勾選 Codex provider 並儲存設定
- **THEN** `enabled_providers` 不包含 `"codex"`
