## ADDED Requirements

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
