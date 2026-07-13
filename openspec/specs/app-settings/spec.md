## Purpose

定義 SessionHub 應用程式設定的持久化格式、預設值與設定頁應提供的管理能力。

## Requirements

### Requirement: 設定持久化

系統 SHALL 將應用程式設定儲存於 `%APPDATA%\SessionHub\settings.json`，應用程式重啟後設定保持不變。

#### Scenario: 儲存設定

- **WHEN** 使用者修改設定並點擊儲存
- **THEN** 系統將設定寫入 `settings.json`

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

#### Scenario: Agents 設定欄位

- **WHEN** 系統讀寫 settings.json
- **THEN** AppSettings SHALL 額外包含 `allow_create_project_config_dir: bool` 與 `agents_source_root: String`
- **AND** `allow_create_project_config_dir` 預設為 `false`
- **AND** `agents_source_root` 預設為空字串，代表使用預設全域來源根目錄

### Requirement: 設定對話框權限判斷

系統 SHALL 在設定對話框開啟圖標點擊時随即執行權限判斷。

#### Scenario: 設定對話框開啟

- **WHEN** 使用者點擊開啟設定對話框
- **THEN** 系統桌點檢查目前設定值是否完整
- **AND** 若 copilotRoot、opencodeRoot 或 codexRoot 未設定，自動嘗試從環境變數推導預設路徑

### Requirement: opencode_root 自動填入

系統 SHALL 自動偵測 OpenCode 的預設 storage 目錄並填入設定。

#### Scenario: 預設路徑自動偵測

- **WHEN** 使用者開啟設定對話框時 opencode_root 為空
- **THEN** 系統自動嘗試 `%USERPROFILE%\.local\share\opencode\` 路徑
- **AND** 若該目錄存在，將其填入輸入框

### Requirement: codex_root 自動填入

系統 SHALL 自動偵測 Codex 的預設 session 根目錄並填入設定。

#### Scenario: 預設路徑自動偵測

- **WHEN** 使用者開啟設定對話框時 `codex_root` 為空
- **THEN** 系統自動嘗試 `%USERPROFILE%\.codex` 路徑
- **AND** 若該目錄存在，將其填入輸入框

### Requirement: 設定頁顯示 provider integration 狀態

系統 SHALL 在設定頁顯示每個已支援 provider 的 integration 狀態、設定檔位置，以及最後檢查結果。設定檔位置欄位的標籤 SHALL 依該 provider 實際的整合機制顯示對應語意的名稱，不得對所有 provider 一律使用同一個籠統標籤。

#### Scenario: 顯示 provider 狀態

- **WHEN** 使用者開啟設定頁
- **THEN** 系統顯示 Copilot、OpenCode、Codex 與 Claude Code 各自的 integration 狀態
- **AND** 顯示其設定檔、plugin 或 hook 路徑（若可解析）

#### Scenario: 重新檢查 integration 狀態

- **WHEN** 使用者點擊 provider 的「重新檢查」
- **THEN** 系統重新偵測 integration 安裝狀態
- **AND** 更新畫面中的狀態與錯誤訊息

#### Scenario: Claude Code 顯示 Hook 路徑標籤

- **WHEN** 設定頁顯示 Claude Code 的 provider integration 卡片
- **THEN** 設定檔位置欄位標籤 SHALL 顯示「Hook 路徑」（而非「設定 / plugin 路徑」）
- **AND** 欄位值為 Claude Code hooks 設定目錄（例如 `%USERPROFILE%\.claude\hooks`）

#### Scenario: 其他 provider 沿用既有標籤

- **WHEN** 設定頁顯示 Copilot、OpenCode 或 Codex 的 provider integration 卡片
- **THEN** 設定檔位置欄位標籤沿用「設定 / plugin 路徑」

### Requirement: 一般設定不顯示 Claude Hook 腳本路徑欄位

設定頁的一般設定區塊 SHALL NOT 顯示「Claude Hook 腳本路徑」輸入欄位；hook 腳本路徑的檢視與調整 SHALL 統一由平台整合管理區（provider integration 卡片）提供。`AppSettings.hook_scripts_path` 欄位與其後端預設值邏輯保留不變。

#### Scenario: 一般設定無 hook 路徑欄位

- **WHEN** 使用者開啟設定頁的一般設定區塊
- **THEN** 不顯示「Claude Hook 腳本路徑」的 label、輸入框與「選擇資料夾」按鈕

#### Scenario: 既有自訂路徑不遺失

- **WHEN** 使用者先前已在 settings.json 設定自訂 `hook_scripts_path`，且於 UI 移除欄位後儲存其他設定
- **THEN** `hook_scripts_path` 原值原樣寫回 settings.json，不被清空或重設為預設值

#### Scenario: 整合管理區為唯一調整入口

- **WHEN** 使用者需要檢視或變更 Claude hook 腳本路徑
- **THEN** 於平台整合管理區的 Claude Code 卡片檢視「Hook 路徑」並透過編輯操作調整

### Requirement: 專案設定資料夾建立開關

系統 SHALL 於全域設定提供 `allowCreateProjectConfigDir` 開關（預設關閉）。此開關僅控制是否允許新建或寫入專案根目錄下的 `.sessionhub/` 資料夾，不影響讀取行為；即使開關關閉，若專案內已存在 `.sessionhub/agents.json`，系統仍會讀取並繼續寫回該既有檔案。

#### Scenario: 設定頁顯示開關

- **WHEN** 使用者開啟設定頁
- **THEN** 系統於「Agents」區塊顯示「允許在專案內建立 .sessionhub 設定資料夾」開關與說明文字
- **AND** 說明文字明確標註僅影響新建或寫入，不影響既有檔案的讀取
- **AND** 預設為關閉

#### Scenario: 開啟時新偏好存於專案內

- **WHEN** `allowCreateProjectConfigDir` 為開啟，且系統需要儲存專案 agents 偏好，且專案內尚無該檔案
- **THEN** 系統建立 `<project>/.sessionhub/agents.json` 並寫入
- **AND** 首次建立時 UI 顯示提示，建議使用者自行決定是否將 `.sessionhub/` 加入 `.gitignore`

#### Scenario: 關閉且專案內無既有檔案時走 APPDATA fallback

- **WHEN** `allowCreateProjectConfigDir` 為關閉，且系統需要儲存專案 agents 偏好，且專案內尚無 `.sessionhub/agents.json`
- **THEN** 系統不在專案內建立任何檔案，改寫入 `%APPDATA%\SessionHub\project-agents\<專案路徑雜湊>.json`

#### Scenario: 關閉但專案內已有既有檔案時仍寫回專案內

- **WHEN** `allowCreateProjectConfigDir` 為關閉，但專案內已存在 `.sessionhub/agents.json`
- **THEN** 系統仍讀取並寫回該既有檔案，不因開關關閉而改寫到 APPDATA
- **AND** 不會刪除或搬移該既有檔案

#### Scenario: 開關切換不遷移既有偏好

- **WHEN** 使用者切換 `allowCreateProjectConfigDir` 的開關狀態
- **THEN** 系統不自動搬移或合併 APPDATA 與專案內兩處既有的偏好內容，兩者各自獨立保留
- **AND** 後續讀取依「專案內優先、否則 APPDATA」的優先序取用其一，不合併欄位

### Requirement: 專案級 agents 偏好持久化

系統 SHALL 持久化每個專案的 agents 偏好：記住的衝突選擇（conflictChoice）、掃描忽略路徑（ignoredPaths）、啟用的同步目標（enabledTargets）。讀取時 SHALL 優先採用專案內 `.sessionhub/agents.json`，不存在時回退至 APPDATA 位置，兩者皆不存在時使用預設值（衝突每次詢問、無忽略路徑、四個目標全啟用）。

#### Scenario: 讀取偏好的優先序

- **WHEN** 開啟專案的 Agents 分頁
- **THEN** 系統依序嘗試讀取 `<project>/.sessionhub/agents.json` → APPDATA fallback → 預設值
- **AND** 此優先序不受 `allowCreateProjectConfigDir` 開關狀態影響

#### Scenario: 偏好向後相容

- **WHEN** 偏好檔缺少部分欄位（舊版本寫入）
- **THEN** 系統以預設值補齊缺少欄位，不報錯

### Requirement: 全域 agents 來源根目錄可自訂

系統 SHALL 於全域設定提供 `agentsSourceRoot` 欄位（預設為空字串），讓使用者自訂全域範圍 Skills、Commands、AGENTS.md 的正本來源目錄，覆寫預設的 `~/.agents`。此設定僅影響全域範圍，專案範圍固定使用 `<project>/.agents`，不受影響。

#### Scenario: 設定頁顯示來源路徑欄位

- **WHEN** 使用者開啟設定頁的「Agents」區塊
- **THEN** 系統顯示「Agents 正本根目錄（全域範圍）」路徑輸入欄與「瀏覽」按鈕
- **AND** 欄位留空時顯示預設值（`%USERPROFILE%\.agents`）作為提示文字

#### Scenario: 設定自訂路徑後全域掃描改用該路徑

- **WHEN** 使用者將 `agentsSourceRoot` 設為自訂路徑並儲存
- **THEN** 全域範圍的 Skills、Commands 矩陣改以該路徑下的 `skills` 與 `skills/command` 為來源端進行掃描與同步
- **AND** 全域 AGENTS.md 掃描改以該目錄為根

#### Scenario: 欄位留空時維持原有預設行為

- **WHEN** `agentsSourceRoot` 為空字串或僅含空白
- **THEN** 系統 fallback 使用 `%USERPROFILE%\.agents` 作為全域來源根目錄，行為與升級前一致

#### Scenario: 專案範圍不受此設定影響

- **WHEN** 使用者已設定 `agentsSourceRoot`，並開啟某專案的 Agents 分頁
- **THEN** 該專案範圍仍固定使用 `<project>/.agents` 作為來源根目錄，不套用全域自訂路徑

### Requirement: 設定頁在寬畫面展開 provider integration 管理區塊

系統 SHALL 在設定頁的桌面寬畫面中，將 provider integration 管理區塊配置於可有效利用主內容寬度的獨立版面區域，而非與一般設定欄位共同擠在單一窄卡片中。

#### Scenario: 桌面寬畫面顯示 provider integration

- **WHEN** 使用者在桌面寬畫面開啟設定頁
- **THEN** 系統將 provider integration 顯示在獨立卡片或等效寬版區塊中
- **AND** 該區塊寬度 SHALL 明顯大於一般設定欄位區塊

#### Scenario: provider integration 包含多筆資訊與操作

- **WHEN** provider integration 卡片顯示狀態、操作按鈕、設定檔路徑、bridge 路徑與最後事件時間
- **THEN** 系統 SHALL 以分區排版呈現這些資訊
- **AND** 長路徑與操作按鈕不應因單一卡片欄寬過窄而長期處於擁擠換行狀態

### Requirement: 設定頁 provider integration 版面需具備響應式回退

系統 SHALL 在較窄視窗或空間不足時，讓 provider integration 管理區塊回退為可閱讀的堆疊式布局，但仍需保留完整的狀態、路徑與操作能力。

#### Scenario: 視窗寬度不足

- **WHEN** 設定頁可用寬度不足以容納寬版排版
- **THEN** 系統將 provider integration 內容回退為堆疊式布局
- **AND** 使用者仍可看到並操作所有 provider integration 功能

### Requirement: 預設啟動工具設定

系統 SHALL 在 AppSettings 新增 `default_launcher` 欄位，讓使用者選擇點擊 SessionCard 主啟動按鈕時的預設工具。

#### Scenario: 讀取預設啟動工具

- **WHEN** 系統載入設定
- **THEN** `default_launcher` 欄位為 None 時，系統預設使用 `terminal`

#### Scenario: 儲存預設啟動工具

- **WHEN** 使用者在設定頁選擇預設啟動工具並儲存
- **THEN** 系統將選擇的工具類型字串寫入 settings.json 的 `default_launcher` 欄位

### Requirement: 狀態列開關設定

系統 SHALL 在設定頁一般設定區塊提供開關，讓使用者啟用或停用全域狀態列。

#### Scenario: 關閉狀態列

- **WHEN** 使用者在設定頁將「顯示狀態列」切換為關閉並儲存
- **THEN** `settings.showStatusBar` 設為 `false`
- **AND** 全域狀態列立即從畫面消失

#### Scenario: 預設啟用

- **WHEN** 使用者首次安裝或 settings.json 尚未包含 `show_status_bar` 欄位
- **THEN** 系統預設 `show_status_bar` 為 `true`，狀態列顯示

### Requirement: 設定頁可啟用或停用 Codex provider

系統 SHALL 在設定頁的 provider 啟用區塊提供 Codex 選項。

#### Scenario: 啟用 Codex

- **WHEN** 使用者勾選 Codex provider 並儲存設定
- **THEN** `enabled_providers` 會包含 `"codex"`

#### Scenario: 停用 Codex

- **WHEN** 使用者取消勾選 Codex provider 並儲存設定
- **THEN** `enabled_providers` 不包含 `"codex"`

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

### Requirement: 主題色系改由設定頁管理

系統 SHALL 將主題與色系切換集中於設定頁管理，避免 Sidebar 在展開與收折狀態出現不同位置或重複控制。

#### Scenario: Sidebar 不重複顯示主題切換

- **WHEN** 使用者切換 Sidebar 展開或收折狀態
- **THEN** 系統不應因不同狀態重複渲染多組主題切換控制

#### Scenario: 設定頁顯示主題切換

- **WHEN** 使用者開啟設定頁
- **THEN** 系統於設定頁顯示單一主題或色系切換控制
- **AND** 其行為與原先 Sidebar 主題切換一致

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
