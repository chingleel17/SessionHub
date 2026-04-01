## MODIFIED Requirements

### Requirement: Plan sub-tab 可關閉標籤樣式
Plan sub-tab 使用可關閉標籤，系統 SHALL 以 inline-flex 佈局顯示標籤文字與 × 關閉按鈕，外觀與普通 sub-tab 視覺一致，× 按鈕位於標籤文字右側且不搶眼（預設 opacity 較低，hover 時升高）。

#### Scenario: Plan sub-tab 正常顯示
- **WHEN** plan sub-tab 被開啟
- **THEN** 標籤顯示「Plan · <session 摘要前綴>」，右側有 × 按鈕，整體視覺與 Sessions/Plans & Specs 標籤一致

#### Scenario: 關閉 Plan sub-tab
- **WHEN** 使用者點擊 × 按鈕
- **THEN** 該 plan tab 從列表移除，activeSubTab 回退至 "sessions"

### Requirement: Plan sub-tab 狀態跨專案保留
系統 SHALL 在應用程式運行期間保留每個專案已開啟的 plan sub-tab 狀態，切換至其他專案再切回時，先前開啟的 plan tab 應仍存在。

#### Scenario: 切換專案再切回
- **WHEN** 使用者在專案 A 開啟 plan tab，切換到專案 B，再切回專案 A
- **THEN** 專案 A 的 plan tab 仍然存在且 active sub-tab 保持原狀態

#### Scenario: 關閉應用程式不保留
- **WHEN** 使用者關閉應用程式後重新啟動
- **THEN** plan tab 狀態不保留（重置為 "sessions" sub-tab）
