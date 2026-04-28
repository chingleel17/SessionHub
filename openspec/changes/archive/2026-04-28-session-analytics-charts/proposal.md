## Why

目前 SessionHub 的統計資訊僅提供即時數字加總（本周／本月的 token、互動次數、點數），無法觀察趨勢變化、跨時段比較或各專案佔比分布。使用者需要能以圖表形式查看指定時間區間的統計走勢與圓餅圖，協助評估 AI coding 工具的使用效率與成本分配。

## What Changes

- 新增後端 Rust command，支援依日／周／月分組的統計聚合查詢（指定專案或全部專案、指定起迄時間）
- 新增前端圖表元件（折線趨勢圖、圓餅分布圖），使用輕量級 SVG 方案繪製，無需引入大型圖表框架
- 在 **ProjectView** 新增「Analytics」子頁籤，可手動觸發指定區間的圖表查詢
- 在 **Dashboard** 新增 Analytics Panel，顯示全域趨勢走勢圖，定時自動重新整理（可設定 10 分鐘或 30 分鐘）
- 圖表類型：token 趨勢折線圖、互動次數折線圖、點數消耗折線圖、各專案佔比圓餅圖

## Capabilities

### New Capabilities

- `analytics-query-engine`：後端聚合查詢引擎，依時間分組從 session_stats 快取與 events.jsonl 計算統計走勢資料，回傳結構化時序陣列
- `analytics-charts-ui`：前端圖表元件（折線圖 `TrendChart`、圓餅圖 `PieChart`），純 SVG + CSS，接收時序資料陣列並渲染
- `project-analytics-tab`：ProjectView 內新增 Analytics 子頁籤，提供時間範圍選擇器與手動觸發查詢，展示該專案的圖表
- `dashboard-analytics-panel`：Dashboard 新增 Analytics Panel，顯示全域（所有專案）圖表，支援定時自動重整（預設 30 分鐘）

### Modified Capabilities

- `dashboard`：新增 Analytics Panel 區塊，修改 dashboard 版面配置
- `project-subtabs`：新增 Analytics 頁籤至 ProjectView 子頁籤清單
- `dashboard-stats-period`：Analytics Panel 復用時間範圍切換器（本周／本月／自訂），統計週期選擇邏輯需對圖表查詢同步適用

## Impact

- **Rust 後端**：新增 `get_analytics_data` command（`src-tauri/src/commands/`），查詢 `session_stats` 資料表並以 groupBy 參數（day/week/month）聚合
- **SQLite**：不新增資料表，複用現有 `session_stats` 快取；若快取不足則觸發按需解析
- **前端**：新增 `src/components/TrendChart.tsx`、`PieChart.tsx`、`AnalyticsPanel.tsx`；`App.tsx` 新增 analytics query invoke 與定時 refresh 邏輯
- **相依性**：純 SVG 方案，不引入 recharts / d3 等外部圖表套件（保持套件精簡）
- **無破壞性變更**：既有 stats 計算路徑不受影響
