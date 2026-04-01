## MODIFIED Requirements

### Requirement: 過濾並顯示 session
系統 SHALL 支援多維度過濾 session 列表，包含關鍵字搜尋、標籤篩選、封存狀態切換，以及「隱藏空 session（summaryCount = 0）」切換。

#### Scenario: 正常讀取 session 列表
- **WHEN** 使用者開啟應用程式且 `session-state/` 目錄存在
- **THEN** 系統顯示所有可解析的 session，每筆包含：id、summary（若存在）、cwd、created_at、updated_at、summaryCount

#### Scenario: 隱藏空 session 過濾
- **WHEN** 使用者啟用「隱藏空 session」選項
- **THEN** 系統從列表移除所有 summaryCount = 0 的 session，其他過濾條件不受影響

#### Scenario: 空 session 篩選與封存篩選獨立運作
- **WHEN** 使用者同時啟用「隱藏空 session」與「顯示封存」
- **THEN** 系統顯示：已封存且 summaryCount > 0 的 session，以及未封存且 summaryCount > 0 的 session
