## MODIFIED Requirements

### Requirement: 設定欄位完整定義

AppSettings SHALL 包含以下欄位：

#### Scenario: 設定結構

- **WHEN** 系統讀寫 settings.json
- **THEN** 完整格式如下：
  - `copilot_root: String` — Copilot session-state 父目錄
  - `opencode_root: String` — OpenCode storage 根目錄
  - `codex_root: String` — Codex 根目錄（其下包含 `sessions/` 與 integration 設定）
  - `terminal_path: String` — 終端機執行檔路徑
  - `external_editor_path: String` — 外部編輯器路徑
  - `show_archived: bool` — 是否顯示封存 session
  - `show_status_bar: bool` — 是否顯示全域底部狀態列（預設 `true`）
  - `pinned_projects: Vec<String>` — 釘選專案的 cwd 列表
  - `enabled_providers: Vec<String>` — 啟用的 provider（`"copilot"` / `"opencode"` / `"codex"`）
  - `provider_integrations: HashMap<String, ProviderIntegration>` — provider bridge 設定
  - `default_launcher: Option<String>` — 預設啟動工具（`"terminal"` / `"opencode"` / `"gh-copilot"` / `"gemini"` / `"explorer"`）
  - `enable_intervention_notification: bool` — 是否啟用 Windows 介入通知（預設 `true`）

### Requirement: 設定頁顯示 provider integration 狀態

系統 SHALL 在設定頁顯示每個已支援 provider 的 integration 狀態、設定檔位置，以及最後檢查結果。

#### Scenario: 顯示 provider 狀態

- **WHEN** 使用者開啟設定頁
- **THEN** 系統顯示 Copilot、OpenCode 與 Codex 各自的 integration 狀態
- **AND** 顯示其設定檔、plugin 或 hook 路徑（若可解析）

#### Scenario: 重新檢查 integration 狀態

- **WHEN** 使用者點擊 provider 的「重新檢查」
- **THEN** 系統重新偵測 integration 安裝狀態
- **AND** 更新畫面中的狀態與錯誤訊息

## ADDED Requirements

### Requirement: 設定頁文案涵蓋三個 provider integration

系統 SHALL 以能同時描述 Copilot hook、OpenCode plugin 與 Codex hook 的文案呈現 provider integration 區塊。

#### Scenario: 顯示 integration 區塊說明
- **WHEN** 使用者開啟設定頁的 provider integration 管理區塊
- **THEN** 系統顯示涵蓋三個 provider 的說明文字
- **AND** 不得只描述 Copilot 與 OpenCode 兩種整合方式
