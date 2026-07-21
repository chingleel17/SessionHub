## ADDED Requirements

### Requirement: ProjectView Analytics 子頁籤

ProjectView SHALL 在子分頁列新增「Analytics」頁籤，提供該專案的統計圖表查詢介面。

#### Scenario: Analytics 頁籤顯示

- **WHEN** 使用者進入任一專案 tab
- **THEN** ProjectView 子分頁列 SHALL 包含「Analytics」頁籤，並固定顯示在最後一個位置（位於「Plans & Specs」右側）

#### Scenario: 初始狀態

- **WHEN** 使用者切換至 Analytics 頁籤但尚未查詢
- **THEN** 顯示時間範圍選擇器（快速選項：最近 7 天 / 最近 30 天 / 本月 / 自訂）與「產生圖表」按鈕
- **AND** 圖表區域顯示空白提示文字「選擇時間範圍後點擊產生圖表」

### Requirement: 手動觸發圖表查詢

Analytics 頁籤 SHALL 在使用者點擊「產生圖表」後才觸發後端查詢，不自動載入。

#### Scenario: 點擊產生圖表

- **WHEN** 使用者選擇時間範圍後點擊「產生圖表」
- **THEN** 前端呼叫 `get_analytics_data`，傳入當前專案的 `cwd`、所選起訖日期、以及預設 `groupBy: "day"`
- **AND** 查詢期間顯示載入狀態

#### Scenario: 查詢完成顯示圖表

- **WHEN** `get_analytics_data` 回傳資料
- **THEN** 頁面顯示 `TrendChart`（token / 互動趨勢折線圖）與 `PieChart`（各模型 token 佔比）
- **AND** 圖表標題顯示所選時間範圍

#### Scenario: 查詢失敗

- **WHEN** `get_analytics_data` 回傳錯誤
- **THEN** 呼叫 `showToast` 顯示錯誤訊息，圖表區域維持上一次成功結果（或空狀態）

### Requirement: 分組粒度切換

Analytics 頁籤 SHALL 提供 groupBy 切換，讓使用者選擇「依日 / 依周 / 依月」的聚合粒度。

#### Scenario: 切換 groupBy

- **WHEN** 使用者切換分組粒度後點擊「產生圖表」
- **THEN** 重新查詢並以新粒度重新渲染折線圖
- **AND** X 軸標籤格式對應更新（日：MM-DD，周：WNN，月：YYYY-MM）

### Requirement: 查詢結果在頁籤存活期間快取

Analytics 頁籤的查詢結果 SHALL 在該頁籤未被銷毀前保留在元件 state，避免切換子頁籤後重新查詢。

#### Scenario: 切換子頁籤後切回

- **WHEN** 使用者切換至 Sessions 頁籤再切回 Analytics 頁籤
- **THEN** 上次查詢的圖表仍顯示，不重新查詢
