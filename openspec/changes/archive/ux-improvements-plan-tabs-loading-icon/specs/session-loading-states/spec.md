## ADDED Requirements

### Requirement: 切換專案時顯示骨架載入
當 sessions 資料尚未載入完成時，系統 SHALL 在 ProjectView 的 sessions 列表區域顯示骨架卡片（skeleton cards），以動畫方式告知使用者正在載入中。

#### Scenario: 首次開啟專案分頁
- **WHEN** 使用者點擊尚未載入過的專案分頁
- **THEN** sessions 列表區域顯示 3 個骨架卡片，帶有 shimmer 動畫

#### Scenario: 資料載入完成
- **WHEN** sessions 資料回傳完成
- **THEN** 骨架卡片消失，正常 session 卡片顯示

#### Scenario: 重新切換回已載入的專案
- **WHEN** 使用者切換到先前已載入的專案分頁
- **THEN** 立即顯示快取的 session 卡片，不顯示骨架（React Query staleTime 內）
