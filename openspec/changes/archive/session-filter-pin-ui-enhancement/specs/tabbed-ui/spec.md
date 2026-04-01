## MODIFIED Requirements

### Requirement: 固定 Dashboard 標籤
系統 SHALL 在標籤列最左側固定顯示 Dashboard 標籤，且釘選專案的標籤緊接在 Dashboard 之後、其他專案標籤之前顯示。

#### Scenario: 應用程式啟動
- **WHEN** 應用程式啟動
- **THEN** Dashboard 標籤自動開啟且為第一個標籤，釘選專案標籤（若有）緊接其後

#### Scenario: 釘選專案 tab 置頂
- **WHEN** 使用者釘選一個或多個專案
- **THEN** 這些專案的 tab 排列在 Dashboard tab 之後，非釘選專案 tab 之前，並以特殊視覺樣式（pin icon）標示

#### Scenario: 釘選 tab 點擊
- **WHEN** 使用者點擊釘選專案 tab
- **THEN** activeView 切換至該專案，效果與點擊一般專案 tab 相同
