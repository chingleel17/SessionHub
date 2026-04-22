## ADDED Requirements

### Requirement: 介入通知發送

系統 SHALL 在偵測到 session 活動狀態從非 `waiting` 轉換為 `waiting` 時，若設定中 `enable_intervention_notification` 為 `true`，透過 `tauri-plugin-notification` 發送 Windows Toast 通知。

#### Scenario: 狀態轉換觸發通知

- **WHEN** 前端輪詢到某 session 的 `SessionActivityStatus.status` 由其他狀態變為 `waiting`
- **AND** `AppSettings.enable_intervention_notification` 為 `true`
- **THEN** 系統呼叫 `send_intervention_notification` Tauri command
- **AND** 通知標題為 `SessionHub — 需要您介入`
- **AND** 通知內容包含 session 的專案名稱（取自 `cwd` 最後一段路徑）與 summary 前 60 字元
- **AND** 通知攜帶 `session_id` 作為識別資料

#### Scenario: 通知設定關閉時不發送

- **WHEN** 前端偵測到 session 進入 `waiting` 狀態
- **AND** `AppSettings.enable_intervention_notification` 為 `false`
- **THEN** 系統不發送任何通知

#### Scenario: 同一 session 不重複通知

- **WHEN** session 持續處於 `waiting` 狀態（未轉換為其他狀態再回來）
- **THEN** 系統對該 session 僅發送一次通知，不重複觸發
- **AND** 去重狀態儲存於前端 React state（`Map<sessionId, lastNotifiedStatus>`），應用重啟後清空

### Requirement: Hook 腳本通知（獨立路徑）

系統 SHALL 在 provider bridge hook 腳本中提供 PowerShell WinRT Toast 片段，當 AI agent 發出需介入信號時，即使 SessionHub 未開啟也能發送系統通知。

#### Scenario: Hook 腳本發送 Toast

- **WHEN** hook 腳本偵測到 `waiting` 狀態事件並執行通知邏輯
- **THEN** 腳本透過 PowerShell `[Windows.UI.Notifications.ToastNotificationManager]` 發送 Toast
- **AND** Toast 的 `tag` 設為 `session_id`，避免同一 session 多次通知疊加

#### Scenario: Hook 腳本通知不依賴 SessionHub 運行狀態

- **WHEN** hook 腳本執行時 SessionHub 行程未開啟
- **THEN** 通知仍正常發送，不因 SessionHub 未運行而失敗

### Requirement: 通知點擊聚焦 Session

系統 SHALL 在使用者點擊 Windows Toast 通知後，將 SessionHub 視窗帶到前景並聚焦至對應 session 的專案 tab。

#### Scenario: 點擊通知聚焦視窗

- **WHEN** 使用者點擊 SessionHub 發出的 Toast 通知
- **THEN** SessionHub 主視窗取得焦點並帶到前景
- **AND** 前端路由切換至該 session 所屬的 project tab（以 `session_id` 定位 `projectKey`）

#### Scenario: 點擊通知時 SessionHub 未開啟

- **WHEN** 使用者點擊 Toast 通知時 SessionHub 未在執行
- **THEN** 系統不崩潰（通知點擊無 callback 可執行）
- **AND** 此情況僅於 Hook 腳本路徑發生，SessionHub 應用端通知點擊一定有視窗存在

### Requirement: 通知去重使用 Toast Tag

系統 SHALL 在發送 Toast 通知時設定 `tag` 為 `sessionhub-{session_id}`、`group` 為 `intervention`，確保同一 session 的後續通知取代而非疊加顯示。

#### Scenario: 相同 session 通知取代舊通知

- **WHEN** 系統對同一 session 發送第二次通知
- **THEN** Windows 通知中心顯示最新通知，移除之前同 tag 的通知
- **AND** 通知數量不因多次輪詢而累積
