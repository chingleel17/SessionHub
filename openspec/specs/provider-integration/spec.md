## Requirements

### Requirement: Provider integration 安裝與狀態管理

系統 SHALL 能檢測 Copilot 與 OpenCode 的 bridge integration 狀態，並允許使用者由 SessionHub 自動安裝、更新或重新安裝整合檔案；系統 SHALL 同時追蹤已安裝插件的版本號，並在版本落差時提示使用者更新。

#### Scenario: 安裝 OpenCode integration

- **WHEN** 使用者在設定頁對 OpenCode 點擊「安裝整合」
- **THEN** 系統建立或更新 OpenCode plugin 檔案到偵測到的 plugin 設定位置
- **AND** 狀態更新為 `installed` 或顯示具體錯誤

#### Scenario: provider 路徑不可寫入

- **WHEN** SessionHub 無法寫入 provider 設定目錄
- **THEN** 系統將該 provider 狀態標示為 `manual_required`
- **AND** 提供快速開啟或編輯設定檔案的入口

#### Scenario: 偵測到已安裝版本過舊

- **WHEN** 使用者進入設定頁或應用程式啟動
- **AND** 已安裝插件的 `integrationVersion` 低於程式內建的 `CURRENT_PLUGIN_VERSION`
- **THEN** 系統將整合狀態標示為 `outdated` 並顯示版本資訊（已安裝 v{N} / 最新 v{M}）
- **AND** 提供「更新插件」按鈕

#### Scenario: 重新安裝已是最新版的插件

- **WHEN** 使用者在設定頁點擊「重新安裝」且插件已是最新版
- **THEN** 系統仍執行寫入（覆蓋），並顯示 success toast

### Requirement: Bridge 事件格式標準化

系統 SHALL 以統一的本地 bridge 事件格式接收 provider 更新訊號，至少包含 provider、eventType、timestamp，並在可取得時包含 sessionId 與 cwd。

#### Scenario: OpenCode plugin 發送 session 更新

- **WHEN** OpenCode plugin 發出 `session.updated` 或等效事件
- **THEN** SessionHub 接收的 bridge record 以標準欄位格式保存
- **AND** 後續 refresh 流程不需直接解析 OpenCode 原始事件格式

#### Scenario: Copilot hook 發送 session 結束

- **WHEN** Copilot hook 發出 session 結束事件
- **THEN** SessionHub 接收的 bridge record 包含 provider=`copilot`
- **AND** 系統可依該事件重新驗證並更新對應 session 清單
