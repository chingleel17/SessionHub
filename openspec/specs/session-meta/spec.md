## ADDED Requirements

### Requirement: 自訂備註
系統 SHALL 允許使用者對指定 session 新增或編輯純文字備註，備註儲存於本地 SQLite DB，不修改 `workspace.yaml`。

#### Scenario: 新增備註
- **WHEN** 使用者在 session 卡片上點擊「備註」並輸入文字後儲存
- **THEN** 系統將備註寫入 SQLite `session_meta` 表

#### Scenario: 清空備註
- **WHEN** 使用者在編輯備註對話框將內容清空後儲存
- **THEN** 系統將 `notes` 儲存為 `NULL`

### Requirement: 自訂標籤
系統 SHALL 允許使用者對指定 session 新增、改名與刪除標籤，標籤儲存於本地 SQLite DB 的 `session_meta.tags`。

#### Scenario: 批次編輯標籤
- **WHEN** 使用者在編輯標籤對話框輸入以逗號分隔的標籤字串並儲存
- **THEN** 系統將字串解析為標籤陣列並覆蓋原有標籤

#### Scenario: 單一標籤改名
- **WHEN** 使用者點擊 session 卡片右側某一個標籤 chip，並輸入新名稱後儲存
- **THEN** 系統只更新被點擊的那一筆標籤，其餘標籤維持不變

#### Scenario: 單一標籤刪除
- **WHEN** 使用者在單一標籤編輯對話框按下「刪除此標籤」
- **THEN** 系統只刪除被點擊的那一筆標籤

#### Scenario: 標籤改名合併重複值
- **WHEN** 使用者將某標籤改名為已存在的標籤（大小寫視為相同）
- **THEN** 系統 SHALL 自動去重並合併，避免重複標籤
