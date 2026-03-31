## ADDED Requirements

### Requirement: 自訂備註
系統 SHALL 允許使用者對指定 session 新增或編輯純文字備註，備註儲存於本地 SQLite DB，不修改 `workspace.yaml`。

#### Scenario: 新增備註
- **WHEN** 使用者在 session 卡片上點擊「備註」並輸入文字後儲存
- **THEN** 系統將備註寫入 SQLite `session_meta` 表
