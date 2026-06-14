# jq-dependency-check Specification

## Purpose
TBD - created by archiving change hook-driven-status-sync. Update Purpose after archive.
## Requirements
### Requirement: 後端 check_jq_available command
後端 SHALL 提供 `check_jq_available` Tauri command，執行 `jq --version`（或 `which jq`）判斷 jq 是否可用，回傳 `bool`。

#### Scenario: jq 已安裝時回傳 true
- **WHEN** 系統 PATH 中可找到 `jq`
- **THEN** `check_jq_available` 回傳 `true`

#### Scenario: jq 未安裝時回傳 false
- **WHEN** 系統中無 jq
- **THEN** `check_jq_available` 回傳 `false`，不回傳 error

### Requirement: SettingsView 在 Claude integration 區域顯示 jq 狀態提示
當用戶進入「設定」頁面且 Claude integration 卡片可見時，SettingsView SHALL 呼叫 `check_jq_available` 一次，若回傳 false 則顯示 info banner 說明 jq 的用途與安裝方式。

#### Scenario: jq 不可用時顯示安裝提示
- **WHEN** 用戶開啟設定頁面，`check_jq_available` 回傳 `false`
- **THEN** Claude integration 卡片下方顯示 info banner，內容包含 winget 安裝指令或 Git for Windows 安裝說明

#### Scenario: jq 可用時不顯示任何提示
- **WHEN** `check_jq_available` 回傳 `true`
- **THEN** 設定頁面不顯示任何 jq 相關提示

#### Scenario: jq 提示不阻擋安裝按鈕
- **WHEN** jq 不可用，banner 顯示中
- **THEN** Claude integration 的「安裝」按鈕仍可點擊，安裝流程正常進行

#### Scenario: 提示文字透過 i18n key 顯示
- **WHEN** 顯示 jq 相關 banner
- **THEN** 所有文字使用 `t("settings.jqNotFound.*")` i18n key，不硬編碼中文

