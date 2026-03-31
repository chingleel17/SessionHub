## ADDED Requirements

### Requirement: 排除空 session 篩選
系統 SHALL 在 session 篩選列提供「隱藏無對話 session」切換選項，啟用時排除所有 summaryCount = 0 的 session（不含封存 session 的顯示邏輯，兩者獨立）。

#### Scenario: 啟用「隱藏空 session」
- **WHEN** 使用者啟用「隱藏空 session」切換
- **THEN** 系統隱藏所有 summaryCount = 0 的 session，其餘 session 正常顯示

#### Scenario: 關閉「隱藏空 session」
- **WHEN** 使用者關閉「隱藏空 session」切換
- **THEN** 系統顯示全部 session（不受 summaryCount 過濾）

#### Scenario: 空 session 計數提示
- **WHEN** 目前列表中存在 summaryCount = 0 的 session
- **THEN** 篩選列顯示目前隱藏的空 session 數量（例如：「已隱藏 3 個空 session」）

### Requirement: 批次刪除空 session
系統 SHALL 提供「刪除所有空 session」操作，刪除前顯示確認對話框告知預計刪除數量，確認後呼叫後端 `delete_empty_sessions` command 執行刪除。

#### Scenario: 批次刪除確認流程
- **WHEN** 使用者點擊「刪除空 session」按鈕
- **THEN** 系統顯示確認對話框，內容包含「將刪除 N 個無對話記錄的 session，此操作不可復原」

#### Scenario: 確認刪除執行
- **WHEN** 使用者在確認對話框點擊確認
- **THEN** 系統呼叫 `delete_empty_sessions` command，刪除所有 summaryCount = 0 的 session 資料夾，顯示 Toast「已刪除 N 個空 session」，並重新整理 session 列表

#### Scenario: 無空 session 時的狀態
- **WHEN** 目前列表中不存在 summaryCount = 0 的 session
- **THEN** 「刪除空 session」按鈕為禁用狀態
