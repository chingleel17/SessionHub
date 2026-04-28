## Context

SessionHub 目前的統計資訊（token、互動次數、計費點數）以即時數字呈現，每次渲染時從 `session_stats` SQLite 快取加總。時間範圍固定為「本周 / 本月」切換，無法查看細粒度的走勢變化或各專案間的比例分布。

本設計目標是在不破壞現有即時統計的前提下，新增以「圖表」形式展示時序聚合資料的能力。

## Goals / Non-Goals

**Goals:**
- 新增後端聚合查詢 command，將 `session_stats` 依日/周/月分組並回傳時序陣列
- 新增純 SVG 折線趨勢圖與圓餅分布圖元件（無外部圖表套件相依）
- 在 ProjectView 新增 Analytics 子頁籤，支援手動觸發、時間範圍選擇
- 在 Dashboard 新增 Analytics Panel，支援定時自動重整（預設 30 分鐘）
- 支援「指定單一專案」或「所有專案」兩種查詢模式

**Non-Goals:**
- 即時（< 1 秒）資料串流圖表
- 跨裝置或雲端同步統計資料
- 匯出圖表為圖片或 PDF
- 自訂聚合維度（如按 model 分組的走勢圖，第一版不做）

## Decisions

### 決策 1：純 SVG 圖表，不引入外部圖表套件

**選擇：** 自行以 SVG + TypeScript 實作 `TrendChart` 與 `PieChart` 元件。

**理由：** recharts / d3 等套件體積大（100 KB+ gzip），且本專案需求單純（只需折線圖與圓餅圖）。SVG 方案輕量、可完全控制樣式（配合現有純 CSS BEM 慣例），且無授權或相容性風險。

**替代方案：** recharts（React 原生但肥大）、d3（功能強大但 API 複雜、學習成本高）。

---

### 決策 2：後端新增 `get_analytics_data` command，從 session_stats 聚合

**選擇：** 在 `src-tauri/src/commands/` 新增 `analytics.rs`，SQL 查詢 `session_stats` 資料表，依 `updated_at`（或 `created_at`）的日期分組。

**資料流：**
```
frontend invoke("get_analytics_data", { cwd?, startDate, endDate, groupBy })
  → analytics_internal(db, params)
    → SELECT session_id, output_tokens, input_tokens, interaction_count, ... FROM session_stats
      JOIN sessions_cache ON session_id WHERE cwd LIKE ? AND date BETWEEN ?
      GROUP BY strftime('%Y-%m-%d', updated_at)  -- or %Y-%W / %Y-%m
    → Vec<AnalyticsDataPoint>
```

**AnalyticsDataPoint struct：**
```rust
pub struct AnalyticsDataPoint {
    pub label: String,          // "2025-04-21" / "2025-W16" / "2025-04"
    pub output_tokens: i64,
    pub input_tokens: i64,
    pub interaction_count: i64,
    pub cost_points: f64,       // sum of modelMetrics.requestsCost
    pub session_count: i64,
}
```

**理由：** 不新增資料表，複用既有快取；若 session_stats 快取不完整，前端可先觸發掃描再查詢。

---

### 決策 3：Dashboard 定時重整使用 `setInterval` + React state

**選擇：** Dashboard Analytics Panel 以 React `useEffect` + `setInterval` 驅動，間隔由設定值（10 或 30 分鐘）決定，預設 30 分鐘。不使用 FS watcher 或 Tauri event。

**理由：** 圖表資料不需要毫秒級更新，輪詢方式實作最簡單且不消耗系統資源。首次進入 Dashboard 時立即觸發一次查詢。

---

### 決策 4：ProjectView Analytics 頁籤為手動觸發（非自動重整）

**選擇：** Analytics 子頁籤預設顯示空白提示，使用者選擇時間範圍後點擊「產生圖表」才觸發查詢，結果在頁面存活期間快取在元件 state。

**理由：** 每次進入專案頁籤就觸發聚合查詢成本高（大型專案可能有數百個 session），手動觸發更符合使用情境，也避免影響啟動效能。

---

### 決策 5：圓餅圖顯示「各專案 token 佔比」，只在全域（Dashboard）模式有意義

**選擇：** 圓餅圖固定顯示「各專案在查詢區間的 token / 互動佔比」。若在 ProjectView 單一專案模式下，圓餅圖可切換為「各模型 token 佔比」。

**理由：** 單一專案的跨時段圓餅圖意義有限，模型分布更有參考價值。

## Risks / Trade-offs

- **[Risk] session_stats 快取不完整** → 若使用者尚未瀏覽過所有 session，部分 session 的統計不在快取中，圖表資料可能不完整。→ **Mitigation**：圖表查詢前先呼叫現有的 `refresh_all_stats` 掃描，或在 UI 顯示「資料可能不完整，請先完成掃描」提示。
- **[Risk] SQLite GROUP BY 日期效能** → session 數量龐大時，跨月聚合查詢可能緩慢。→ **Mitigation**：加上 `updated_at` 索引（若尚未存在）；查詢結果同樣以 React Query 快取。
- **[Risk] SVG 圖表可及性（a11y）不足** → 純 SVG 無障礙支援有限。→ **Mitigation**：加上 `role="img"` 與 `aria-label`，圖表下方提供數字摘要表格。
- **[Risk] cost_points 只對 Copilot 有意義** → OpenCode 目前無 requestsCost 欄位。→ **Mitigation**：UI 對 cost_points 為 0 的點不繪製，並說明僅 Copilot 支援。

## Migration Plan

1. 後端：新增 `analytics.rs` command 及 Rust struct，在 `run()` 的 `invoke_handler![]` 登記
2. 前端：新增 `TrendChart.tsx`、`PieChart.tsx`、`AnalyticsPanel.tsx` 元件
3. `App.tsx` 新增 `invoke("get_analytics_data", ...)` 函式與 React Query key
4. ProjectView：新增 Analytics 子頁籤（sub-tab），在 project-subtabs 機制下登記
5. Dashboard：在現有 stats 區域下方插入 Analytics Panel，加入 setInterval 定時重整

無資料庫 migration（不新增資料表，只新增 SQL 查詢）。可隨時回滾（移除 command 與元件即可）。

## Open Questions

- `cost_points` 聚合時，`modelMetrics` 目前存為 JSON blob 在 session_stats。需確認是否已有獨立的 `cost_points` 欄位或需解析 JSON 加總。
- Dashboard Analytics Panel 的位置：在「統計卡片」下方還是「Kanban/清單」下方？
- 圖表顏色主題是否需跟隨 app-theme 的深色 / 淺色模式切換？
