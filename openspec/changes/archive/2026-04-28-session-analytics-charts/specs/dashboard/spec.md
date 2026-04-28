## MODIFIED Requirements

### Requirement: Dashboard 視圖模式切換

系統 SHALL 提供清單視圖與 Kanban 視圖的切換按鈕，兩種視圖並列可選。Analytics Panel 位於視圖區域（清單 / Kanban）下方，獨立於視圖模式切換之外。

#### Scenario: 顯示視圖切換按鈕

- **WHEN** 使用者在 Dashboard 頁面
- **THEN** 系統在 Dashboard 標題區域顯示「清單」與「Kanban」切換按鈕
- **AND** 當前視圖的按鈕以 active 樣式標示

#### Scenario: Analytics Panel 不受視圖切換影響

- **WHEN** 使用者切換清單 / Kanban 視圖
- **THEN** Analytics Panel 保持顯示，不因視圖切換而重置或重新查詢
