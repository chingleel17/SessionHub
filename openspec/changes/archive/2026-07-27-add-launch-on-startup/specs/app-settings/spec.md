## ADDED Requirements

### Requirement: 開機自動啟動設定欄位

AppSettings SHALL 額外包含 `launch_on_startup: bool` 與 `start_minimized_on_startup: bool` 兩個欄位，序列化至前端為 `launchOnStartup` 與 `startMinimizedOnStartup`。

#### Scenario: 欄位定義與預設值

- **WHEN** 系統讀寫 `settings.json`
- **THEN** AppSettings SHALL 包含下列欄位：
  - `launch_on_startup: bool` — 是否於使用者登入時自動啟動 SessionHub（預設 `false`）
  - `start_minimized_on_startup: bool` — 開機自動啟動時是否隱藏主視窗、僅常駐系統匣（預設 `true`，僅在 `launch_on_startup` 為 `true` 時生效）

#### Scenario: 舊版設定檔向後相容

- **WHEN** 既有 `settings.json` 不含 `launchOnStartup` 或 `startMinimizedOnStartup`
- **THEN** 系統以預設值 `false` 與 `true` 讀入，不報錯
- **AND** 下次儲存設定時新欄位寫入 `settings.json`

### Requirement: 儲存設定時同步作業系統自動啟動註冊

`save_settings` SHALL 在寫入 `settings.json` 後，依 `launch_on_startup` 的值同步作業系統的登入自動啟動註冊狀態，行為與現有 tray／overlay 設定即時生效的副作用一致。

#### Scenario: 儲存時啟用註冊

- **WHEN** 使用者儲存設定且 `launch_on_startup` 為 `true`
- **THEN** 系統在設定寫入成功後向作業系統註冊自動啟動項目
- **AND** 無需重啟應用程式即生效

#### Scenario: 儲存時解除註冊

- **WHEN** 使用者儲存設定且 `launch_on_startup` 為 `false`
- **THEN** 系統在設定寫入成功後解除作業系統的自動啟動註冊

#### Scenario: 同步失敗回報

- **WHEN** 設定已成功寫入 `settings.json`，但作業系統註冊同步失敗
- **THEN** `settings.json` 保留使用者要求的 `launchOnStartup` 值，作為下次儲存及啟動對帳的重試來源
- **AND** `save_settings` 回傳 `Err(String)` 說明同步失敗原因，供前端顯示錯誤 toast，且不得顯示儲存成功

### Requirement: 設定頁提供開機自動啟動開關

設定頁「一般」區塊 SHALL 提供「開機時自動啟動」與「開機時隱藏至系統匣」兩個開關，所有文案 MUST 透過 `t("key")` 取得，並同時提供 zh-TW 與 en-US 翻譯。

#### Scenario: 切換開機自動啟動

- **WHEN** 使用者在設定頁勾選或取消「開機時自動啟動」
- **THEN** 表單狀態 `launchOnStartup` 隨之更新
- **AND** 儲存後設定生效

#### Scenario: 隱藏啟動選項的相依停用

- **WHEN** `launchOnStartup` 為 `false`
- **THEN** 「開機時隱藏至系統匣」checkbox 呈現 disabled 狀態
