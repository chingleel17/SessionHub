## ADDED Requirements

### Requirement: OpenSpec change SHALL expose tasks progress summary
系統 SHALL 在掃描 OpenSpec change 時，對包含 `tasks.md` 的 change 計算任務完成摘要，提供前端可直接使用的 `done`、`total` 與 `status` 資訊。

#### Scenario: tasks.md 包含 checklist 項目
- **WHEN** change 目錄下存在 `tasks.md`，且其中包含 markdown checklist 項目
- **THEN** 系統回傳該 change 的 task progress
- **AND** `done` 為已勾選項目數
- **AND** `total` 為 checklist 項目總數
- **AND** `status` 依完成狀態標記為 `not_started`、`in_progress` 或 `done`

#### Scenario: tasks.md 不含 checklist 項目
- **WHEN** change 目錄下存在 `tasks.md`，但沒有任何可辨識的 checklist 項目
- **THEN** 系統不得偽造進度數值
- **AND** 前端可將該 change 視為沒有 progress summary

#### Scenario: change 不存在 tasks.md
- **WHEN** change 目錄下不存在 `tasks.md`
- **THEN** 系統仍回傳該 change 的其他 metadata
- **AND** task progress 欄位為空值
