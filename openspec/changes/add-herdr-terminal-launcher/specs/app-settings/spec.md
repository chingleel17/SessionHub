## ADDED Requirements

### Requirement: 設定頁提供終端啟動器選擇

設定頁 SHALL 提供終端啟動器選擇控制項，讓使用者在 shell 與 herdr 之間切換。

#### Scenario: 顯示啟動器選項

- **WHEN** 使用者開啟設定頁的終端相關設定區塊
- **THEN** 系統顯示終端啟動器選擇控制項，包含 `shell` 與 `herdr` 兩個選項
- **AND** 當前選取值反映 `terminal_launcher` 設定

#### Scenario: 切換至 herdr 啟動器

- **WHEN** 使用者將啟動器切換為 herdr 並儲存
- **THEN** 系統保存 `terminal_launcher` 為 `"herdr"`
- **AND** 後續終端啟動改走 herdr 流程

#### Scenario: 啟動器文案在地化

- **WHEN** 系統顯示終端啟動器相關文案
- **THEN** 文案透過翻譯鍵取得，且 zh-TW 與 en-US 兩份 locale 皆提供對應字串

## MODIFIED Requirements

### Requirement: 設定欄位完整定義

AppSettings SHALL 包含以下欄位（含 quota monitoring 相關欄位）：

#### Scenario: 設定結構

- **WHEN** 系統讀寫 settings.json
- **THEN** 完整格式如下：
  - `copilot_root: String` — Copilot session-state 父目錄
  - `opencode_root: String` — OpenCode storage 根目錄
  - `codex_root: String` — Codex 資料根目錄
  - `claude_root: String` — Claude Code 資料根目錄（預設 `~/.claude`）
  - `terminal_path: Option<String>` — 終端機執行檔路徑
  - `terminal_launcher: Option<String>` — 終端啟動器種類（`"shell"` / `"herdr"`，預設 `"shell"`）
  - `external_editor_path: Option<String>` — 外部編輯器路徑
  - `show_archived: bool` — 是否顯示封存 session
  - `show_status_bar: bool` — 是否顯示全域底部狀態列（預設 `true`）
  - `pinned_projects: Vec<String>` — 釘選專案的 cwd 列表
  - `enabled_providers: Vec<String>` — 啟用的 provider（`"copilot"` / `"opencode"` / `"codex"` / `"claude"`）
  - `provider_integrations: Vec<ProviderIntegrationStatus>` — provider bridge 設定
  - `default_launcher: Option<String>` — 預設啟動工具（`"terminal"` / `"opencode"` / `"gh-copilot"` / `"gemini"` / `"explorer"`）
  - `enable_intervention_notification: bool` — 是否啟用 Windows 介入通知（預設 `true`）
  - `enable_session_end_notification: bool` — 是否啟用 session 結束通知（預設 `false`）
  - `analytics_refresh_interval: u32` — analytics 自動刷新間隔（預設 `30` 分鐘）
  - `analytics_panel_collapsed: bool` — analytics panel 是否收合
  - `hook_scripts_path: String` — Claude hook 腳本安裝路徑
  - `claude_quota_reset_day: u8` — Claude 本地 quota 每月重置日
  - `minimize_to_tray: bool` — 是否最小化到系統匣
  - `enable_quota_monitoring: bool` — 是否啟用 quota monitoring（預設 `true`）
  - `quota_enabled_providers: Vec<String>` — 啟用 quota 監控的 provider（`"copilot"` / `"opencode"` / `"codex"` / `"claude"`）
  - `quota_refresh_interval: u32` — quota 自動刷新間隔（單位分鐘，預設 `30`，允許值：`5` / `15` / `30` / `60`）

#### Scenario: 新欄位的序列化與預設值

- **WHEN** 舊版 settings.json 不含 `enable_quota_monitoring`、`quota_enabled_providers` 或 `quota_refresh_interval`
- **THEN** 系統以預設值 `true`、目前支援且已啟用的 provider 清單，以及 `30` 讀入，不報錯
- **AND** 下次儲存設定時新欄位寫入 settings.json

#### Scenario: terminal_launcher 的向後相容

- **WHEN** 舊版 settings.json 不含 `terminal_launcher`
- **THEN** 系統以 `"shell"` 讀入，維持既有終端啟動行為，不報錯
- **AND** 下次儲存設定時該欄位寫入 settings.json

#### Scenario: Agents 設定欄位

- **WHEN** 系統讀寫 settings.json
- **THEN** AppSettings SHALL 額外包含 `allow_create_project_config_dir: bool` 與 `agents_source_root: String`
- **AND** `allow_create_project_config_dir` 預設為 `false`
- **AND** `agents_source_root` 預設為空字串，代表使用預設全域來源根目錄
