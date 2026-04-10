## ADDED Requirements

### Requirement: 啟動時自動偵測插件版本並提示
系統 SHALL 在應用程式啟動後查詢插件狀態，若插件未安裝或版本過舊，顯示一次性 warning toast 引導使用者前往設定頁更新。

#### Scenario: 啟動時插件版本過舊
- **WHEN** 應用程式啟動完成，`get_plugin_status` 回傳 `status: "outdated"`
- **THEN** 顯示 warning toast：「opencode 插件版本過舊，請前往設定頁更新」
- **AND** toast 僅顯示一次（不在每次路由切換時重複）

#### Scenario: 啟動時插件未安裝
- **WHEN** 應用程式啟動完成，`get_plugin_status` 回傳 `status: "not_installed"`
- **THEN** 顯示 warning toast：「opencode 插件尚未安裝，請前往設定頁安裝」
- **AND** toast 僅顯示一次

#### Scenario: 啟動時插件已是最新版
- **WHEN** 應用程式啟動完成，`get_plugin_status` 回傳 `status: "up_to_date"`
- **THEN** 不顯示任何提示，靜默通過

#### Scenario: 版本查詢失敗
- **WHEN** `get_plugin_status` 呼叫回傳錯誤
- **THEN** 不顯示 toast，不阻塞應用程式主流程
