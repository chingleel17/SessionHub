## MODIFIED Requirements

### Requirement: Kanban 看板欄位定義

系統 SHALL 將所選 Dashboard 時間範圍內的 non-archived sessions 依自動偵測的活動狀態分配至對應欄位。

#### Scenario: Active 欄顯示

- **WHEN** session 的活動狀態為 `active`
- **THEN** 系統將該 session 的 Kanban 卡片顯示於 Active 欄

#### Scenario: Waiting 欄顯示

- **WHEN** session 的活動狀態為 `waiting`
- **THEN** 系統將該 session 的 Kanban 卡片顯示於 Waiting 欄

#### Scenario: Idle 欄顯示

- **WHEN** session 的活動狀態為 `idle`
- **THEN** 系統將該 session 的 Kanban 卡片顯示於 Idle 欄

#### Scenario: Done 欄顯示

- **WHEN** session 的活動狀態為 `done`（已封存或超過 24h 無活動）
- **THEN** 系統將該 session 的 Kanban 卡片顯示於 Done 欄

#### Scenario: 超出區間的 session 不顯示

- **WHEN** session 的 `updatedAt` 不在目前選取的本周或本月區間內
- **THEN** 該 session 不會出現在任何 Kanban 欄位

### Requirement: Done 欄位數量限制

Done 欄位 SHALL 限制顯示數量，避免已完成 session 大量佔用畫面。

#### Scenario: Done 欄預設顯示 10 個

- **WHEN** Done 欄的 ProjectCard 總數（或 session 數）超過 10 個
- **THEN** 系統只顯示最新的 10 個
- **AND** 底部顯示「載入更多」按鈕

#### Scenario: 載入更多

- **WHEN** 使用者點擊「載入更多」按鈕，或捲動至 Done 欄底部
- **THEN** 系統追加顯示下一批（10 個）Done 狀態的項目

#### Scenario: 篩選區間改變時重置載入數量

- **WHEN** 使用者在 Dashboard 切換本周或本月
- **THEN** Done 欄位的載入更多數量重設為初始值
- **AND** 不沿用先前區間留下的展開數量
