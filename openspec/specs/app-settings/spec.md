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
