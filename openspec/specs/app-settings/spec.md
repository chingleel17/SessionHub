## ADDED Requirements

### Requirement: 設定持久化

系統 SHALL 將應用程式設定儲存於 `%APPDATA%\SessionHub\settings.json`，應用程式重啟後設定保持不變。

#### Scenario: 儲存設定

- **WHEN** 使用者修改設定並點擊儲存
- **THEN** 系統將設定寫入 `settings.json`

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

### Requirement: 設定對話框權限判斷

系統 SHALL 在設定對話框開啟圖標點擊時随即執行權限判斷。

#### Scenario: 設定對話框開啟

- **WHEN** 使用者點擊開啟設定對話框
- **THEN** 系統桌點檢查目前設定值是否完整
- **AND** 若 copilotRoot 或 opencodeRoot 未設定，自動嘗試從環境變數推導預設路徑

### Requirement: opencode_root 自動填入

系統 SHALL 自動偵測 OpenCode 的預設 storage 目錄並填入設定。

#### Scenario: 預設路徑自動偵測

- **WHEN** 使用者開啟設定對話框時 opencode_root 為空
- **THEN** 系統自動嘗試 `%USERPROFILE%\.local\share\opencode\` 路徑
- **AND** 若該目錄存在，將其填入輸入框

### Requirement: 設定頁顯示 provider integration 狀態

系統 SHALL 在設定頁顯示每個已支援 provider 的 integration 狀態、設定檔位置，以及最後檢查結果。

#### Scenario: 顯示 provider 狀態

- **WHEN** 使用者開啟設定頁
- **THEN** 系統顯示 Copilot 與 OpenCode 各自的 integration 狀態
- **AND** 顯示其設定檔或 plugin 路徑（若可解析）

#### Scenario: 重新檢查 integration 狀態

- **WHEN** 使用者點擊 provider 的「重新檢查」
- **THEN** 系統重新偵測 integration 安裝狀態
- **AND** 更新畫面中的狀態與錯誤訊息

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
