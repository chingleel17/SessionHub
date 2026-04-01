## ADDED Requirements

### Requirement: 批次刪除空 session（後端）
後端 SHALL 提供 `delete_empty_sessions` command，接收 Copilot root 路徑，刪除所有 `workspace.yaml` 中 summaryCount = 0（或解析後 messages 清單為空）的 session 資料夾，回傳實際刪除數量。

#### Scenario: 成功批次刪除
- **WHEN** 前端呼叫 `delete_empty_sessions` 且存在 summaryCount = 0 的 session
- **THEN** 後端刪除這些 session 的資料夾，回傳刪除數量（usize）

#### Scenario: 無空 session 時的回傳
- **WHEN** 前端呼叫 `delete_empty_sessions` 但所有 session 皆有對話記錄
- **THEN** 後端回傳 0，不執行任何刪除

#### Scenario: 刪除失敗處理
- **WHEN** 批次刪除過程中某個 session 資料夾無法刪除（如權限問題）
- **THEN** 後端記錄錯誤並繼續處理其餘 session，最終回傳成功刪除數量
