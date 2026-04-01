## ADDED Requirements

### Requirement: ProjectView 統計摘要 banner

系統 SHALL 在 ProjectView 頂部（toolbar-card 區域內）顯示專案統計摘要 banner。

#### Scenario: Banner 顯示

- **WHEN** ProjectView 渲染且所有 session stats 已載入
- **THEN** banner 顯示：「N sessions · X turns · Y tokens」（含 K/M 格式化）
- **AND** 統計包含可見與已封存的所有 session

#### Scenario: 部分 stats 載入中

- **WHEN** 部分 session 的 stats 仍在載入
- **THEN** banner 以現有資料顯示目前加總值，並附上載入指示

#### Scenario: 無 session 的專案

- **WHEN** 專案沒有任何 session
- **THEN** 統計 banner 隱藏（不顯示「0 sessions · 0 turns · 0 tokens」）
