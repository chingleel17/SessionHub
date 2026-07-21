## ADDED Requirements

### Requirement: 資料位置現況檢視

系統 SHALL 在設定頁「資料位置」區塊列出各已啟用 provider（Claude Code、Codex、Copilot CLI、opencode）與 SessionHub 自身的資料目錄現況，包含：解析後的實際路徑、路徑來源（預設 / 環境變數 / 手動設定）、目錄是否存在、目錄佔用大小。

#### Scenario: 顯示各工具資料位置

- **WHEN** 使用者開啟設定頁的「資料位置」區塊
- **THEN** 系統列出每個已啟用 provider 的實際資料根目錄路徑
- **AND** 標示該路徑來源為「預設位置」、「環境變數」（含變數名稱）或「手動設定」
- **AND** 同時列出 SessionHub 自身資料目錄（`%APPDATA%\SessionHub` 或 override 後的位置）

#### Scenario: 非同步計算目錄大小

- **WHEN** 「資料位置」區塊載入
- **THEN** 各目錄大小以非同步方式計算，計算期間顯示 loading 狀態
- **AND** 目錄不存在時顯示「不存在」而非錯誤

### Requirement: 資料根目錄解析納入官方環境變數

系統解析各 provider 預設資料根目錄時，SHALL 依序採用：AppSettings 手動設定值 → 官方環境變數 → `USERPROFILE` 預設路徑。官方環境變數對應為：Claude Code `CLAUDE_CONFIG_DIR`、Codex `CODEX_HOME`、Copilot CLI `COPILOT_HOME`、opencode `XDG_DATA_HOME`（資料位於 `$XDG_DATA_HOME\opencode`）。

#### Scenario: 環境變數已設定且無手動設定

- **WHEN** 使用者未在 AppSettings 手動填寫 provider root，且對應環境變數已設定（如 `CODEX_HOME=D:\AgentData\codex`）
- **THEN** session 掃描、quota 讀取與現況檢視皆以環境變數指向的目錄為準

#### Scenario: 環境變數未設定

- **WHEN** AppSettings 未手動設定且環境變數不存在
- **THEN** 系統沿用既有 `USERPROFILE` 預設路徑，行為與現行版本一致

### Requirement: 引導式資料搬遷

系統 SHALL 提供搬遷流程：使用者為某工具選擇新目錄後，依序執行（1）複製既有資料至新目錄並驗證、（2）寫入使用者層級（`HKCU\Environment`）環境變數並廣播 `WM_SETTINGCHANGE`、（3）同步更新 SessionHub 對應的 root 設定。來源目錄 MUST 保留不刪除。

#### Scenario: 搬遷成功

- **WHEN** 使用者選擇新目錄並確認搬遷，且複製與驗證（檔案數與總大小比對）成功
- **THEN** 系統寫入對應的使用者層級環境變數指向新目錄
- **AND** 更新 AppSettings 中對應 root 欄位，session 掃描立即改讀新位置
- **AND** 顯示完成提示：需重開終端機、確認正常後可自行刪除舊目錄、若憑證失效需重新登入

#### Scenario: 搬遷前置檢查

- **WHEN** 使用者確認開始搬遷
- **THEN** 系統先檢查目的地磁碟可用空間不小於來源目錄大小，不足則拒絕開始並說明原因
- **AND** 提示使用者先關閉對應的 CLI 工具

#### Scenario: 複製失敗或使用者取消

- **WHEN** 複製過程中發生錯誤或使用者取消
- **THEN** 系統停止複製並移除目的地已複製的部分內容
- **AND** 不寫入環境變數、不更新 AppSettings，維持搬遷前狀態

#### Scenario: 搬遷進度回報

- **WHEN** 複製進行中
- **THEN** 後端以 Tauri event 回報進度（已複製檔案數/總數、bytes）
- **AND** 前端顯示進度並提供取消按鈕

#### Scenario: XDG_DATA_HOME 已被設為其他值

- **WHEN** 使用者對 opencode 啟動搬遷，但 `XDG_DATA_HOME` 已存在且指向其他位置
- **THEN** 系統不覆寫該變數，改為顯示說明並指示使用者手動處理
- **AND** UI 註明 `XDG_DATA_HOME` 為共用變數，變更會影響其他遵循 XDG 的程式

#### Scenario: SessionHub 自身資料目錄搬遷

- **WHEN** 使用者搬遷 SessionHub 自身資料目錄
- **THEN** 系統複製資料後寫入 `COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE` 使用者層級環境變數
- **AND** 提示需重啟 SessionHub 才生效
