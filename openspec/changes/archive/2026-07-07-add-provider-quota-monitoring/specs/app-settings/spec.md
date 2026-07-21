## MODIFIED Requirements

### Requirement: 設定欄位完整定義

AppSettings SHALL 包含以下欄位（含本次新增的 quota monitoring 欄位）：

#### Scenario: 設定結構

- **WHEN** 系統讀寫 settings.json
- **THEN** 完整格式如下：
  - `copilot_root: String` — Copilot session-state 父目錄
  - `opencode_root: String` — OpenCode storage 根目錄
  - `codex_root: String` — Codex 資料根目錄
  - `claude_root: String` — Claude Code 資料根目錄（預設 `~/.claude`）
  - `terminal_path: Option<String>` — 終端機執行檔路徑
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
  - `claude_quota_reset_day: u8` — Claude 本地 quota 每月重置日（既有欄位，保留）
  - `minimize_to_tray: bool` — 是否最小化到系統匣
  - **`enable_quota_monitoring: bool`** — 是否啟用 quota monitoring（**新增**，預設 `true`）
  - **`quota_refresh_interval: u32`** — quota 自動刷新間隔（**新增**，單位分鐘，預設 `30`，允許值：`5` / `15` / `30` / `60`）

#### Scenario: 新欄位的序列化與預設值

- **WHEN** 舊版 settings.json 不含 `enable_quota_monitoring` 或 `quota_refresh_interval`
- **THEN** 系統以預設值 `true` 與 `30` 讀入，不報錯
- **AND** 下次儲存設定時新欄位寫入 settings.json

## ADDED Requirements

### Requirement: 設定頁提供 quota monitoring 開關與刷新間隔

系統 SHALL 在設定頁提供 quota monitoring 的開關與刷新間隔設定，讓使用者控制是否顯示與更新 provider quota。

#### Scenario: 關閉 quota monitoring
- **WHEN** 使用者在設定頁關閉 quota monitoring 並儲存
- **THEN** 系統將 `enable_quota_monitoring: false` 寫入 settings.json
- **AND** 停止背景 quota refresh 輪詢
- **AND** Dashboard quota overview 區塊與 status bar quota 摘要隱藏

#### Scenario: 調整 quota refresh interval
- **WHEN** 使用者在設定頁選擇新的 quota refresh interval 並儲存
- **THEN** 系統將新設定寫入 settings.json
- **AND** 後續背景 quota refresh 依新間隔執行
- **AND** 允許值為 5 / 15 / 30 / 60 分鐘（UI 以下拉選單呈現）

#### Scenario: 預設行為
- **WHEN** 使用者首次啟動 SessionHub（無既有設定）
- **THEN** `enable_quota_monitoring` 預設為 `true`，`quota_refresh_interval` 預設為 `30`
- **AND** app 啟動後自動執行第一次 quota refresh
