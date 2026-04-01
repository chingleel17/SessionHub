## ADDED Requirements

### Requirement: 設定頁顯示 provider integration 狀態
系統 SHALL 在設定頁顯示每個已支援 provider 的 integration 狀態、設定檔位置，以及最後檢查結果，讓使用者了解目前使用 bridge 或 fallback 模式。

#### Scenario: 顯示 provider 狀態
- **WHEN** 使用者開啟設定頁
- **THEN** 系統顯示 Copilot 與 OpenCode 各自的 integration 狀態
- **AND** 顯示其設定檔或 plugin 路徑（若可解析）

### Requirement: 設定頁提供 provider 設定快速操作
系統 SHALL 在設定頁提供 provider 設定檔的快速開啟、直接編輯，以及安裝或更新 integration 的操作入口。

#### Scenario: 快速開啟 provider 設定
- **WHEN** 使用者點擊 provider 的「開啟設定」或等效按鈕
- **THEN** 系統開啟對應設定目錄或檔案

#### Scenario: 重新檢查 integration 狀態
- **WHEN** 使用者點擊 provider 的「重新檢查」
- **THEN** 系統重新偵測 integration 安裝狀態
- **AND** 更新畫面中的狀態與錯誤訊息
