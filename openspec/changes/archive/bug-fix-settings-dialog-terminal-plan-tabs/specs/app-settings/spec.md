## ADDED Requirements

### Requirement: dialog 權限宣告
系統 SHALL 在 `capabilities/default.json` 中包含 `"dialog:default"` 以啟用檔案/資料夾選擇 dialog。

#### Scenario: 選擇資料夾按鈕
- **WHEN** 使用者在設定頁面點擊「選擇資料夾」按鈕
- **THEN** 系統開啟原生資料夾選擇 dialog

#### Scenario: 選擇檔案按鈕
- **WHEN** 使用者在設定頁面點擊「選擇檔案」按鈕
- **THEN** 系統開啟原生檔案選擇 dialog

### Requirement: opencode_root 顯示補填
系統 SHALL 在 `get_settings` 回傳設定前，若 `opencode_root` 欄位為空字串，自動補填 `default_opencode_root()` 計算出的預設路徑。

#### Scenario: 舊版 settings.json 無 opencode_root 欄位
- **WHEN** 應用程式讀取不含 `opencode_root` 的舊版 `settings.json`
- **THEN** 前端收到的 `opencode_root` 為計算後的預設路徑（非空字串）

#### Scenario: opencode_root 已有值
- **WHEN** `settings.json` 中 `opencode_root` 已有非空字串
- **THEN** 系統直接回傳原有值，不做補填

## MODIFIED Requirements

### Requirement: 設定持久化
系統 SHALL 將應用程式設定儲存於 `%APPDATA%\SessionHub\settings.json`，應用程式重啟後設定保持不變。

#### Scenario: 儲存設定
- **WHEN** 使用者修改設定並點擊儲存
- **THEN** 系統將設定寫入 `settings.json`

#### Scenario: 讀取設定時補填空白欄位
- **WHEN** 應用程式啟動並讀取 `settings.json`
- **THEN** 任何空白的路徑欄位（如 `opencode_root`）SHALL 以對應的預設值填充後再回傳給前端
