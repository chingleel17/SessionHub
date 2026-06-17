## ADDED Requirements

### Requirement: Hook 離線系統通知

系統 SHALL 在 provider hook 腳本中提供獨立於 SessionHub 行程的系統通知能力，使用隨附的 `snoretoast.exe` 發送 Windows Toast；當 SessionHub 未開啟時通知仍正常發送。

#### Scenario: SessionHub 未運行時 hook 仍發通知

- **WHEN** provider hook 腳本在對應事件點執行
- **AND** SessionHub 行程未開啟
- **THEN** hook 透過隨附的 `snoretoast.exe` 發送 Windows Toast 通知
- **AND** 通知發送不因 SessionHub 未運行而失敗

#### Scenario: notify 模組失敗不阻斷 hook

- **WHEN** `notify.cjs` 呼叫 `snoretoast.exe` 發生錯誤
- **THEN** hook 不拋出例外、不阻斷其餘 hook 流程
- **AND** 錯誤寫入 `%APPDATA%\SessionHub\logs\hook-errors.log`

### Requirement: 依事件語意觸發通知

系統 SHALL 由各 provider hook 在「完成」「等待回應/需決策」「需授權」事件點直接觸發通知，不依賴 SessionHub 應用內的 activity 狀態判定。

#### Scenario: Copilot 觸發點

- **WHEN** Copilot `on-session-end` 執行
- **THEN** 發送「完成」類通知
- **WHEN** Copilot `on-pre-tool-use` 執行（即將執行工具、可能需授權）
- **THEN** 發送「需授權」類通知

#### Scenario: Codex 觸發點

- **WHEN** Codex `on-stop` 執行
- **THEN** 發送「完成」類通知
- **WHEN** Codex `PermissionRequest` 事件觸發
- **THEN** 發送「需授權」類通知

#### Scenario: Claude 觸發點

- **WHEN** Claude `Stop` hook 觸發
- **THEN** 發送「完成」類通知
- **WHEN** Claude `Notification` hook 以 matcher `permission_prompt` 觸發
- **THEN** 發送「需授權」類通知
- **WHEN** Claude `Notification` hook 以 matcher `idle_prompt` 觸發
- **THEN** 發送「等待回應」類通知

### Requirement: 通知開關以設定快照控制

系統 SHALL 讓 hook 啟動時讀取已落地的 `settings.json` 快照，依 `enable_intervention_notification` 與 `enable_session_end_notification` 決定是否發送，無需 SessionHub 提供 IPC。

#### Scenario: 完成類通知受 session end 開關控制

- **WHEN** hook 準備發送「完成」類通知
- **AND** 快照中 `enable_session_end_notification` 為 `false`
- **THEN** hook 不發送通知

#### Scenario: 等待/授權類通知受介入開關控制

- **WHEN** hook 準備發送「等待回應/需決策」或「需授權」類通知
- **AND** 快照中 `enable_intervention_notification` 為 `false`
- **THEN** hook 不發送通知

#### Scenario: 設定讀取失敗採安全預設

- **WHEN** hook 無法讀取 `settings.json` 或欄位缺失
- **THEN** `enable_intervention_notification` 視為 `true`、`enable_session_end_notification` 視為 `false`

### Requirement: 通知去重使用 Tag

系統 SHALL 在 hook 發送通知時，將 snoretoast 的 `tag` 設為 `sessionhub-{session_id}`、`group` 設為 `intervention`，確保同一 session 後續通知取代而非疊加。

#### Scenario: 同一 session 通知取代舊通知

- **WHEN** hook 對同一 session 發送第二次通知
- **THEN** Windows 通知中心以相同 `tag` 取代先前通知，數量不累積

### Requirement: snoretoast 與 notify 模組隨 hook 安裝

系統 SHALL 在 provider hook 安裝流程中，一併將 `snoretoast.exe` 與 `notify.cjs` 複製至 hook 落地目錄，使 hook 可獨立呼叫。

#### Scenario: 安裝 hook 時複製通知資源

- **WHEN** SessionHub 執行 provider hook 安裝
- **THEN** 落地目錄包含可執行的 `snoretoast.exe` 與 `notify.cjs`

#### Scenario: 平台限制

- **WHEN** 在非 Windows 平台執行 `.sh` hook
- **THEN** 不發送系統通知（維持現狀，不視為錯誤）
