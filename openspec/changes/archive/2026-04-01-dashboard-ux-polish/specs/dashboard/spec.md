## MODIFIED Requirements

### Requirement: Session 統計摘要
系統 SHALL 在 Dashboard 頁面以緊湊的水平 stat bar 顯示整體 session 統計資訊，每個指標搭配 icon，並在 token 用量與互動次數旁顯示當前時間範圍（本周 / 本月）標籤。

#### Scenario: 顯示統計數字
- **WHEN** 使用者切換至 Dashboard 頁面
- **THEN** 系統以單列 stat bar 顯示：總 session 數量、已封存數量、活躍專案數量、parse 錯誤數量、token 用量（含時間範圍）、互動次數（含時間範圍）
- **AND** 每個指標前顯示對應 icon
- **AND** stat bar 垂直高度 SHALL 不超過 72px
