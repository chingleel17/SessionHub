## MODIFIED Requirements

### Requirement: Dashboard 統計時間範圍切換
系統 SHALL 在 Dashboard 提供「本周 / 本月」切換器，並將所選時間範圍同時套用到統計卡、專案預覽、最近活動與 Kanban 資料集；token 用量與互動次數只統計所選時間範圍內有更新的 session。

#### Scenario: 預設顯示本周統計
- **WHEN** 使用者切換至 Dashboard 頁面
- **THEN** 系統預設選取「本周」，token 用量與互動次數只加總本周（週一 00:00 至今）更新的 session 統計
- **AND** Dashboard 其餘清單與看板內容也只顯示本周更新的 session

#### Scenario: 切換至本月
- **WHEN** 使用者點擊「本月」切換按鈕
- **THEN** 系統重新計算 token 用量與互動次數，只包含當月（1 日 00:00 至今）更新的 session
- **AND** 專案預覽、最近活動與 Kanban 也同步改為只顯示當月更新的 session
- **AND** 切換器高亮顯示「本月」選項

#### Scenario: 無符合 session 時
- **WHEN** 所選時間範圍內沒有任何 session 更新
- **THEN** token 用量與互動次數顯示 0，不顯示錯誤
- **AND** Dashboard 其餘區塊顯示對應的空狀態，而非舊時間範圍資料
