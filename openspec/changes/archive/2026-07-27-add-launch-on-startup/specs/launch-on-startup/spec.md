## ADDED Requirements

### Requirement: 開機自動啟動註冊

系統 SHALL 提供「開機時自動啟動」能力，於使用者登入 Windows 時自動啟動 SessionHub。註冊範圍 MUST 限於當前使用者（per-user），不得寫入系統層級（HKLM）設定。

#### Scenario: 啟用開機自動啟動

- **WHEN** 使用者在設定頁勾選「開機時自動啟動」並儲存設定
- **THEN** 系統將 `launchOnStartup: true` 寫入 `settings.json`
- **AND** 系統向作業系統註冊當前使用者的登入自動啟動項目
- **AND** 註冊項目的啟動命令包含 `--autostart` 參數

#### Scenario: 停用開機自動啟動

- **WHEN** 使用者取消勾選「開機時自動啟動」並儲存設定
- **THEN** 系統將 `launchOnStartup: false` 寫入 `settings.json`
- **AND** 系統解除該使用者的登入自動啟動註冊

#### Scenario: 註冊失敗時回報錯誤

- **WHEN** 系統嘗試註冊或解除註冊自動啟動項目但作業系統操作失敗（例如權限不足或防毒軟體攔截）
- **THEN** `save_settings` 回傳 `Err(String)` 並附上失敗原因
- **AND** 前端顯示錯誤 toast，不得靜默失敗讓使用者誤以為已生效

### Requirement: 設定為單一真實來源與啟動對帳

`settings.json` 中的 `launchOnStartup` SHALL 為自動啟動狀態的單一真實來源。系統 MUST 於應用程式啟動時執行一次對帳，將作業系統的實際註冊狀態校正為與設定一致。設定為啟用時，系統 MUST 重新寫入註冊，以目前執行檔位置及 `--autostart` 參數取代可能過期的註冊命令。

#### Scenario: 外部停用後的對帳

- **WHEN** 使用者透過工作管理員「啟動」分頁停用 SessionHub，而 `settings.json` 中 `launchOnStartup` 仍為 `true`
- **THEN** 應用程式下次啟動時偵測到作業系統未註冊
- **AND** 系統重新註冊自動啟動項目，使實際狀態與設定一致

#### Scenario: 外部殘留註冊的對帳

- **WHEN** 作業系統存在自動啟動註冊，而 `settings.json` 中 `launchOnStartup` 為 `false`
- **THEN** 應用程式啟動時解除該註冊

#### Scenario: 更新後重新寫入註冊命令

- **WHEN** `settings.json` 中 `launchOnStartup` 為 `true` 且應用程式成功啟動
- **THEN** 系統重新註冊自動啟動項目，使用目前執行檔位置及 `--autostart` 參數
- **AND** 不因作業系統原本已顯示為啟用而略過此操作

#### Scenario: 對帳失敗不阻擋啟動

- **WHEN** 啟動對帳過程中作業系統操作失敗
- **THEN** 系統記錄錯誤但繼續完成應用程式啟動流程，不得中止 `setup`

### Requirement: 啟動來源識別與隱藏啟動

系統 SHALL 依啟動命令是否含 `--autostart` 參數判斷本次啟動來源。僅當啟動命令含 `--autostart`、`launchOnStartup` 與 `startMinimizedOnStartup` 皆為 `true` 時，主視窗 MUST 不顯示，應用程式僅常駐系統匣。

#### Scenario: 開機自動啟動且啟用隱藏啟動

- **WHEN** 應用程式以 `--autostart` 參數啟動，且 `launchOnStartup` 與 `startMinimizedOnStartup` 皆為 `true`
- **THEN** 主視窗保持隱藏且不出現在工作列
- **AND** 系統匣圖示正常建立
- **AND** hook 事件接收、watcher 與 quota 監控等背景功能照常運作

#### Scenario: 開機自動啟動但停用隱藏啟動

- **WHEN** 應用程式以 `--autostart` 參數啟動，且 `launchOnStartup` 為 `true`、`startMinimizedOnStartup` 為 `false`
- **THEN** 主視窗照常顯示

#### Scenario: 停用自動啟動後收到過期啟動參數

- **WHEN** 應用程式以 `--autostart` 參數啟動，但 `launchOnStartup` 為 `false`
- **THEN** 主視窗照常顯示，不受 `startMinimizedOnStartup` 影響

#### Scenario: 使用者手動啟動

- **WHEN** 使用者手動點擊捷徑啟動應用程式（命令不含 `--autostart`）
- **THEN** 主視窗一律顯示，不受 `startMinimizedOnStartup` 影響

#### Scenario: 隱藏啟動後由系統匣喚出視窗

- **WHEN** 應用程式處於隱藏啟動狀態，使用者點擊系統匣圖示或選單的「顯示視窗」
- **THEN** 主視窗顯示並取得焦點

#### Scenario: 隱藏啟動後再次啟動應用程式

- **WHEN** 應用程式處於隱藏啟動狀態，使用者再次執行 SessionHub
- **THEN** single-instance 機制不建立第二實例
- **AND** 既有實例的主視窗顯示並取得焦點

### Requirement: 隱藏啟動時關閉視窗不結束應用程式

當 `launch_on_startup` 與 `start_minimized_on_startup` 皆為 `true` 時，關閉主視窗 SHALL 隱藏視窗而非結束應用程式，即使 `minimize_to_tray` 為 `false`。此條件與 `minimize_to_tray` 為聯集關係，任一成立即觸發隱藏。

#### Scenario: 啟用隱藏啟動但未啟用最小化至系統匣

- **WHEN** `launch_on_startup` 與 `start_minimized_on_startup` 為 `true`、`minimize_to_tray` 為 `false`，使用者點擊主視窗關閉鈕
- **THEN** 系統阻止視窗關閉並改為隱藏視窗
- **AND** 應用程式繼續於系統匣常駐，hook 事件接收與 quota 監控不中斷

#### Scenario: minimize_to_tray 既有行為不變

- **WHEN** `minimize_to_tray` 為 `true` 而 `launch_on_startup` 為 `false`，使用者點擊關閉鈕
- **THEN** 系統維持既有行為，隱藏視窗而非結束應用程式

#### Scenario: 兩者皆停用時正常結束

- **WHEN** `minimize_to_tray`、`launch_on_startup` 皆為 `false`，使用者點擊關閉鈕
- **THEN** 應用程式正常結束

### Requirement: 隱藏啟動設定的相依性

`startMinimizedOnStartup` SHALL 僅在 `launchOnStartup` 為 `true` 時生效。設定頁 MUST 在 `launchOnStartup` 未啟用時將隱藏啟動選項標示為停用狀態。

#### Scenario: 主開關關閉時的 UI 狀態

- **WHEN** 使用者在設定頁未勾選「開機時自動啟動」
- **THEN** 「開機時隱藏至系統匣」選項為 disabled 狀態且不可修改

#### Scenario: 主開關開啟後解除停用

- **WHEN** 使用者勾選「開機時自動啟動」
- **THEN** 「開機時隱藏至系統匣」選項變為可用，並維持其既有值（預設 `true`）
