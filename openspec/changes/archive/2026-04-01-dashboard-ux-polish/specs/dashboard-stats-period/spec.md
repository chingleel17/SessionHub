## ADDED Requirements

### Requirement: Dashboard 統計時間範圍切換
系統 SHALL 在 Dashboard 統計區提供「本周 / 本月」切換器，token 用量與互動次數只統計所選時間範圍內有更新的 session。

#### Scenario: 預設顯示本周統計
- **WHEN** 使用者切換至 Dashboard 頁面
- **THEN** 系統預設選取「本周」，token 用量與互動次數只加總本周（週一 00:00 至今）更新的 session 統計

#### Scenario: 切換至本月
- **WHEN** 使用者點擊「本月」切換按鈕
- **THEN** 系統重新計算 token 用量與互動次數，只包含當月（1 日 00:00 至今）更新的 session
- **AND** 切換器高亮顯示「本月」選項

#### Scenario: 無符合 session 時
- **WHEN** 所選時間範圍內沒有任何 session 更新
- **THEN** token 用量與互動次數顯示 0，不顯示錯誤
