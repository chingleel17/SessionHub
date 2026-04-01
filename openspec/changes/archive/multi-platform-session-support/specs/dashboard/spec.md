## MODIFIED Requirements

### Requirement: Session 統計摘要
系統 SHALL 在 Dashboard 頁面顯示整體 session 的統計資訊，涵蓋所有已啟用平台的 session 資料。

#### Scenario: 顯示多平台統計數字
- **WHEN** 使用者切換至 Dashboard 頁面
- **THEN** 系統顯示總 session 數量（含所有平台）、已封存數量、活躍專案數量

## ADDED Requirements

### Requirement: 依平台統計細分
系統 SHALL 在 Dashboard 顯示各平台的 session 數量分佈。

#### Scenario: 顯示平台分佈
- **WHEN** Dashboard 載入且有多個平台的 session 資料
- **THEN** 顯示各平台的 session 計數（如 "Copilot: 50, OpenCode: 120"）
