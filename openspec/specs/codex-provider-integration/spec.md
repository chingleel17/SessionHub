## ADDED Requirements

### Requirement: Hook 腳本安裝路徑與整合檔指令一致

系統在安裝或更新 Codex / Copilot integration 時，寫出 hook 腳本檔案（`.sh` 與 `.ps1`）的根目錄 SHALL 與寫入 `hooks.json`（或對應 integration 檔）內 hook 指令所引用的腳本路徑為同一目錄，且兩者 SHALL 由同一個解析函式提供，避免路徑分歧。

該腳本根目錄 SHALL 為 provider 設定根目錄下的 `hooks/`（Codex 為 `~/.codex/hooks/`，Copilot 為 `~/.copilot/hooks/`），而非 `%APPDATA%\SessionHub` 下的 bundled 目錄。

#### Scenario: 安裝後腳本存在於整合檔所引用的路徑
- **WHEN** 使用者安裝 Codex integration
- **THEN** `hooks.json` 中每個 hook 指令引用的 `.sh` / `.ps1` 路徑實際存在對應檔案
- **AND** Codex 觸發 hook 時不會因找不到腳本而以非零碼結束

#### Scenario: 安裝與寫入 config 使用同一解析來源
- **WHEN** `ensure_codex_hook_scripts_installed` 寫出腳本，且 `render_codex_integration` 產生指令
- **THEN** 兩者解析出的 hook 腳本根目錄為相同絕對路徑

### Requirement: 清除舊版內嵌 hook group

系統在安裝或更新 Codex integration 時 SHALL 移除既有的 SessionHub hook group，包含新版以 `# sessionhub-provider-event-bridge` 標記的 group，以及舊版 v4 PowerShell 內嵌指令 group（特徵為 `commandWindows` 內含 `provider = 'codex'` 之類的內嵌記錄指令），避免同一事件下新舊 group 並存。

#### Scenario: 升級時清除舊版內嵌 group
- **WHEN** `hooks.json` 中既有舊版 v4 內嵌 PowerShell hook group
- **AND** 使用者重新安裝或更新 Codex integration
- **THEN** 每個事件下只保留一個 SessionHub hook group
- **AND** 舊版內嵌 group 被移除

#### Scenario: 重複安裝維持冪等
- **WHEN** 對同一 `hooks.json` 連續執行兩次安裝
- **THEN** 每個事件下仍只有一個 SessionHub hook group

### Requirement: Codex integration 安裝與狀態管理

系統 SHALL 能檢測 Codex 的 hook integration 狀態，並允許使用者由 SessionHub 自動安裝、更新或重新安裝 Codex hook 設定；系統 SHALL 同時追蹤已安裝 integration 的版本號，並在版本落差時提示使用者更新。

#### Scenario: 安裝 Codex integration
- **WHEN** 使用者在設定頁對 Codex 點擊「安裝整合」
- **THEN** 系統建立或更新 Codex hook 設定到受支援的 Codex 設定位置
- **AND** 狀態更新為 `installed` 或顯示具體錯誤

#### Scenario: 偵測到 Codex integration 缺失
- **WHEN** 使用者開啟設定頁且 Codex integration 檔案不存在或缺少 SessionHub 受管理條目
- **THEN** 系統將 Codex 狀態標示為 `missing`
- **AND** 提供安裝入口

#### Scenario: 偵測到已安裝版本過舊
- **WHEN** 使用者進入設定頁或應用程式啟動
- **AND** 已安裝 Codex integration 的 `integrationVersion` 低於程式內建版本
- **THEN** 系統將狀態標示為 `outdated`
- **AND** 顯示已安裝版本與可更新動作

### Requirement: Codex hook 發送標準 bridge 事件

系統 SHALL 透過 Codex hook 將 session lifecycle 事件寫入本地 bridge 檔案，並沿用既有標準化 bridge record 欄位格式。

#### Scenario: Codex hook 發送 refresh 事件
- **WHEN** Codex hook 在受支援的 lifecycle event 被觸發
- **THEN** 系統寫入 provider=`codex` 的 bridge record
- **AND** bridge record 至少包含 `provider`、`eventType`、`timestamp`

#### Scenario: 事件包含 session 識別資訊
- **WHEN** Codex hook 可取得 `session_id`、`cwd` 或等效欄位
- **THEN** 系統將其寫入標準 bridge 欄位
- **AND** 後端不需直接解析 Codex hook 原始輸入格式才能完成 refresh

### Requirement: Codex bridge 優先於 filesystem watcher

當 Codex integration 已安裝且可用時，系統 SHALL 以 provider bridge 事件作為主要的即時刷新來源。

#### Scenario: Codex bridge 可用
- **WHEN** Codex integration 狀態為 `installed`
- **THEN** 系統監看 Codex bridge 檔案事件並發出 `codex-sessions-updated`
- **AND** 不以 Codex filesystem watcher 作為主要刷新來源

#### Scenario: Codex bridge 不可用
- **WHEN** Codex integration 狀態不是 `installed`
- **THEN** 系統退回使用 Codex session 目錄的 filesystem watcher
- **AND** 維持基本即時更新能力
