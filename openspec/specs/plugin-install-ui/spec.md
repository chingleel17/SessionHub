## ADDED Requirements

### Requirement: 設定頁 opencode 整合狀態區塊

設定頁 SHALL 在「opencode」相關設定區域新增一個整合狀態區塊，顯示 bridge 插件的目前安裝狀態，並提供操作按鈕。

#### Scenario: 顯示「未安裝」狀態

- **WHEN** 進入設定頁，`get_plugin_status` 回傳 `status: "not_installed"`
- **THEN** 顯示紅色/警告標示與文字「未安裝」
- **AND** 顯示「安裝插件」按鈕

#### Scenario: 顯示「版本過舊」狀態

- **WHEN** 進入設定頁，`get_plugin_status` 回傳 `status: "outdated"`
- **THEN** 顯示橘色/警告標示與文字「版本過舊（已安裝 v{N}，最新 v{M}）」
- **AND** 顯示「更新插件」按鈕

#### Scenario: 顯示「已是最新版」狀態

- **WHEN** 進入設定頁，`get_plugin_status` 回傳 `status: "up_to_date"`
- **THEN** 顯示綠色/成功標示與文字「已安裝（v{N}）」
- **AND** 顯示「重新安裝」按鈕（次要樣式）

### Requirement: 點擊安裝/更新按鈕執行插件寫入

系統 SHALL 在使用者點擊安裝/更新/重新安裝按鈕時，呼叫 `install_opencode_plugin` command，完成後重新查詢狀態並顯示結果通知。

#### Scenario: 安裝成功

- **WHEN** 使用者點擊「安裝插件」或「更新插件」按鈕
- **AND** `install_opencode_plugin` 回傳 `Ok`
- **THEN** 按鈕顯示 loading 狀態直到完成
- **AND** 顯示 success toast：「插件安裝成功」
- **AND** 狀態區塊更新為「已是最新版」

#### Scenario: 安裝失敗

- **WHEN** 使用者點擊安裝按鈕
- **AND** `install_opencode_plugin` 回傳 `Err`
- **THEN** 顯示 error toast 包含錯誤訊息
- **AND** 狀態區塊維持原來的錯誤狀態

#### Scenario: 安裝過程中按鈕防重複點擊

- **WHEN** 使用者點擊安裝按鈕後安裝尚未完成
- **THEN** 按鈕呈現 disabled + loading 狀態，防止重複觸發
