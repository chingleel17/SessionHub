## 1. Rust 後端：聚合查詢引擎

- [x] 1.1 在 `src-tauri/src/commands/` 新增 `analytics.rs`，定義 `AnalyticsDataPoint` struct（label, outputTokens, inputTokens, interactionCount, costPoints, sessionCount, missingCount）並加上 `#[serde(rename_all = "camelCase")]`
- [x] 1.2 實作 `get_analytics_data_internal(db, cwd, start_date, end_date, group_by)` 函式，使用 SQLite `strftime` 依 day/week/month 分組聚合 `session_stats` JOIN `sessions_cache`
- [x] 1.3 在 `get_analytics_data_internal` 加入參數驗證（日期格式、startDate ≤ endDate、groupBy 白名單）
- [x] 1.4 將 `cost_points` 從 `session_stats` 的 `model_metrics` JSON 欄位解析並加總（若欄位不存在則為 0）
- [x] 1.5 實作 `#[tauri::command] fn get_analytics_data(...)` 包裝 `_internal`，在 `src-tauri/src/commands/mod.rs` 匯出
- [x] 1.6 在 `src-tauri/src/lib.rs` 的 `invoke_handler![]` 登記 `get_analytics_data`
- [x] 1.7 撰寫 Rust 單元測試：驗證 day/week/month 分組格式、空結果、參數錯誤三種情境

## 2. 前端型別與 IPC

- [x] 2.1 在 `src/types/index.ts` 新增 `AnalyticsDataPoint` TypeScript 介面
- [x] 2.2 在 `src/App.tsx` 新增 `fetchAnalyticsData(cwd: string | null, startDate: string, endDate: string, groupBy: "day" | "week" | "month")` 函式，以 `invoke<AnalyticsDataPoint[]>` 呼叫後端
- [x] 2.3 在 `AppSettings` 介面及 Rust struct 新增 `analyticsRefreshInterval: 10 | 30`（預設 30）與 `analyticsPanelCollapsed: boolean`（預設 false），同步更新序列化與設定讀寫邏輯

## 3. TrendChart 折線圖元件

- [x] 3.1 新增 `src/components/TrendChart.tsx`，以純 SVG 繪製多條折線，props 接受 `data: AnalyticsDataPoint[]`、`lines` 設定（key + label + color）
- [x] 3.2 實作 X 軸標籤（label）、Y 軸數值刻度、折線路徑計算（min/max 正規化）
- [x] 3.3 實作折線顯示切換按鈕（至少保留一條）
- [x] 3.4 實作空資料與單點資料的特殊渲染（空狀態文字 / 單點圓點）
- [x] 3.5 新增 CSS class `trend-chart`，以 CSS 變數定義折線顏色，適配深色 / 淺色主題
- [x] 3.6 在 SVG 根元素加上 `role="img"` 與 `aria-label`

## 4. PieChart 圓餅圖元件

- [x] 4.1 新增 `src/components/PieChart.tsx`，props 接受 `slices: { label: string; value: number; color: string }[]`
- [x] 4.2 以純 SVG path（arc）計算並繪製各切片，附上圖例列表（label + 百分比）
- [x] 4.3 處理空資料（全零）與單一切片（完整圓形）邊界情況
- [x] 4.4 新增 CSS class `pie-chart`，顏色以 CSS 變數定義，適配主題
- [x] 4.5 在 SVG 根元素加上 `role="img"` 與 `aria-label`

## 5. ProjectView Analytics 子頁籤

- [x] 5.1 在 ProjectView 子分頁清單中插入「Analytics」頁籤（Sessions、**Analytics**、Plans & Specs 順序），更新翻譯 key `projectView.tab.analytics`
- [x] 5.2 新增 `src/components/ProjectAnalyticsTab.tsx`，顯示時間範圍選擇器（快速選項 + 自訂）、groupBy 切換、「產生圖表」按鈕
- [x] 5.3 點擊「產生圖表」時呼叫 `App.tsx` 傳入的 `onFetchAnalytics` callback，結果以 `useState` 快取在元件內
- [x] 5.4 查詢完成後渲染 `TrendChart`（折線圖）與 `PieChart`（各模型 token 佔比），標題顯示所選時間範圍
- [x] 5.5 查詢失敗時呼叫 `showToast` 顯示錯誤，保留上一次成功圖表

## 6. Dashboard Analytics Panel

- [x] 6.1 新增 `src/components/DashboardAnalyticsPanel.tsx`，進入 Dashboard 時自動觸發查詢（`cwd: null`，對應當前統計週期），顯示全域趨勢折線圖與各專案 token 佔比圓餅圖
- [x] 6.2 實作折疊 / 展開功能，折疊狀態讀寫 `analyticsPanelCollapsed` 設定
- [x] 6.3 實作 `setInterval` 定時重整，間隔依 `analyticsRefreshInterval` 設定值（10 或 30 分鐘）；重整時背景載入不清空現有圖表
- [x] 6.4 切換「本周 / 本月」統計週期時同步重新查詢圖表
- [x] 6.5 展開時若距上次查詢超過重整間隔則自動重新查詢
- [x] 6.6 載入失敗時顯示錯誤提示與「重試」按鈕
- [x] 6.7 在 Dashboard 頁面的統計卡片下方插入 `DashboardAnalyticsPanel`

## 7. 設定頁面

- [x] 7.1 在 `SettingsView` 新增「Analytics 自動重整間隔」選項（10 分鐘 / 30 分鐘），儲存至 `settings.json`
- [x] 7.2 更新相關翻譯 key（`settings.analyticsRefreshInterval`、`settings.analyticsRefreshInterval.10`、`settings.analyticsRefreshInterval.30`）

## 8. 整合驗證

- [x] 8.1 執行 `bun run build` 確認前端無型別錯誤
- [x] 8.2 執行 `cd src-tauri && cargo test` 確認 Rust 單元測試全數通過
- [x] 8.3 手動驗證：在 ProjectView Analytics 頁籤以真實資料產生圖表，確認折線與圓餅正確渲染
- [x] 8.4 手動驗證：Dashboard Analytics Panel 進入時自動載入，切換統計週期同步更新
