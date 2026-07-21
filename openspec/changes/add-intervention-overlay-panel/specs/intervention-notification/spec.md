## MODIFIED Requirements

### Requirement: 介入通知發送

系統 SHALL 在偵測到 session 活動狀態從非 `waiting` 轉換為 `waiting` 時，若設定中 `enable_intervention_notification` 為 `true` 且 `enable_waiting_toast` 為 `true`，透過 `tauri-plugin-notification` 發送 Windows Toast 通知。`enable_waiting_toast` 使使用者可在僅保留 overlay 常駐提醒時停用 Toast 打擾，且預設為 `true` 以維持既有行為。

#### Scenario: 狀態轉換觸發通知

- **WHEN** 前端輪詢到某 session 的 `SessionActivityStatus.status` 由其他狀態變為 `waiting`
- **AND** `AppSettings.enable_intervention_notification` 為 `true`
- **AND** `AppSettings.enable_waiting_toast` 為 `true`
- **THEN** 系統呼叫 `send_intervention_notification` Tauri command
- **AND** 通知標題為 `SessionHub — 需要您介入`
- **AND** 通知內容包含 session 的專案名稱（取自 `cwd` 最後一段路徑）與 summary 前 60 字元
- **AND** 通知攜帶 `session_id` 作為識別資料

#### Scenario: 通知設定關閉時不發送

- **WHEN** 前端偵測到 session 進入 `waiting` 狀態
- **AND** `AppSettings.enable_intervention_notification` 為 `false`
- **THEN** 系統不發送任何通知

#### Scenario: waiting Toast 開關關閉時不發 Toast

- **WHEN** 前端偵測到 session 進入 `waiting` 狀態
- **AND** `AppSettings.enable_intervention_notification` 為 `true`
- **AND** `AppSettings.enable_waiting_toast` 為 `false`
- **THEN** 系統不發送 Windows Toast
- **AND** overlay 常駐提醒區仍照常顯示該 waiting 項目（不受此開關影響）

#### Scenario: 同一 session 不重複通知

- **WHEN** session 持續處於 `waiting` 狀態（未轉換為其他狀態再回來）
- **THEN** 系統對該 session 僅發送一次通知，不重複觸發
- **AND** 去重狀態儲存於前端 React state（`Map<sessionId, lastNotifiedStatus>`），應用重啟後清空
