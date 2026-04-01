# Tasks: dashboard-ux-polish

## A. SessionStatsBadge — 時長換算小時

- [x] 在 `src/components/SessionStatsBadge.tsx` 新增 `formatDuration(minutes: number): string`
  - `< 60` → `${minutes}m`
  - `>= 60` 且整除 → `${hours}h`
  - `>= 60` 且不整除 → `${(minutes / 60).toFixed(1)}h`
- [x] 將顯示時長的 JSX 由 `{formatCompactNumber(stats.durationMinutes)} {t("stats.duration")}` 改為 `{formatDuration(stats.durationMinutes)}`

## B. Sidebar — 收合時顯示縮短版本號

- [x] 在 `src/components/Sidebar.tsx` 中，將 `sidebar-version` div 移出 `!isSidebarCollapsed` 條件（或另建收合分支）
- [x] 收合狀態：顯示 `v{major}.{minor}`，`title` 屬性帶完整版本號
- [x] 展開狀態：維持現有 `v{packageJson.version}`

## C. App.tsx — workspace-subtitle + period state + filtered totals

- [x] 在 `activeView === "dashboard"` 的 `workspace-subtitle` 分支加入 `t("dashboard.subtitle")`
- [x] 新增 `dashboardPeriod: "week" | "month"` state（預設 `"week"`）
- [x] 計算 `filteredTotalOutputTokens`（useMemo，依 `dashboardPeriod` 篩選 session 的 `updatedAt`）
- [x] 計算 `filteredTotalInteractions`（同上）
- [x] 將 `dashboardPeriod`、`onPeriodChange`、`filteredTotalOutputTokens`、`filteredTotalInteractions` 傳入 `DashboardView`

## D. DashboardView — stat bar 重構 + period 切換

- [x] 接收新 props：`dashboardPeriod`、`onPeriodChange`、`filteredTotalOutputTokens`、`filteredTotalInteractions`
- [x] 移除 `stats-grid` + `stat-card` 結構，改為 `stat-bar` + `stat-bar-item`（icon + value + label）
- [x] 加入本周 / 本月切換按鈕（`period-toggle` 樣式）
- [x] token 用量與互動次數使用 filtered totals

## E. CSS + 翻譯

- [x] 在 `src/styles/` 新增 `.stat-bar`、`.stat-bar-item`、`.period-toggle` 樣式
- [x] 修改 `.workspace-subtitle` 專案路徑字體：`font-size: 0.72rem`、`text-overflow: ellipsis`、`overflow: hidden`、`white-space: nowrap`
- [x] 確保 `.workspace-header` 高度在 dashboard 與專案頁之間一致（`min-height`）
- [x] 新增翻譯 key（zh-TW + en-US）：
  - `dashboard.subtitle`
  - `dashboard.stats.period.week`
  - `dashboard.stats.period.month`
