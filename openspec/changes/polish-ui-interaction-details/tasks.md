## 1. ProviderIcon 可辨識性

- [x] 1.1 調整 `src/components/ProviderIcon.tsx` 與 `App.css` 的 `.provider-icon`：底色改用 provider 專屬 accent token、縮寫字級提高至可讀（≥9px）、確保 dark/light 主題對比 ≥ 3:1
- [x] 1.2 走查 session 卡片、設定頁整合卡片、Dashboard 等所有使用 `ProviderIcon` 的位置，確認五個 provider 皆無空白圓點觀感

## 2. 設定頁立即刷新改為 IconButton

- [x] 2.1 確認/新增 `Icons.tsx` 的線性 refresh 圖示（若 `TrayQuotaPanel`、`QuotaOverview` 已有可沿用則共用）
- [x] 2.2 將 `SettingsView.tsx` quota 監控卡片的「立即刷新」`ghost-button` 改為 `IconButton`（refresh 圖示 + `quota.monitoring.manualRefresh` tooltip），行為不變

## 3. 互動元件 hover / active / 過渡

- [x] 3.1 在 `App.css` 為 `ui-button`、`ui-icon-button`、`ui-select`、checkbox 加入 transition token（背景/邊框/文字色，120–200ms），並於 `prefers-reduced-motion` 停用
- [x] 3.2 重寫 variant hover：primary/danger 改為同色系加深或提亮（新增 hover token 如 `--color-action-primary-hover`），移除套用於 primary/danger 的淺色 subtle 背景 hover；secondary/ghost 維持 subtle 背景 + 邊框強調
- [x] 3.3 加入 `:active` 回饋（背景加深或輕微下沉），reduce motion 下停用 transform
- [x] 3.4 統一 Select 與 checkbox 的 hover、focus-visible 樣式（checkbox 用 accent-color 或自訂樣式）
- [x] 3.5 盤點並遷移殘留的自訂按鈕 class（`ghost-button` 等）至 `ui-button` 對應 variant，移除或別名舊規則
- [x] 3.6 驗證 primary（儲存設定）與 danger 按鈕 hover 時文字 contrast ratio ≥ 4.5:1（dark/light 雙主題）

## 4. 側欄收折體驗

- [x] 4.1 為 `.app-shell` 的 `grid-template-columns` 加入約 200ms 過渡，側欄文字改以 opacity/visibility 淡出淡入取代瞬間 `display: none`，避免擠壓變形；reduce motion 下立即切換
- [x] 4.2 將收折按鈕從 `.sidebar-brand` 內移出為固定位置元素，展開與收折狀態下座標一致（不再位於品牌 icon 旁）
- [x] 4.3 統一收折狀態下品牌 icon 與 `.sidebar-link` icon 的水平置中基準，消除切換時 icon 跳位
- [x] 4.4 驗證 <900px 行動版側欄行為不退化
- [x] 4.5 收折/展開改為共用單一 DOM：移除 `Sidebar.tsx` 收折分支（`sidebar-icon-button`、quick-actions），導覽/釘選/已開啟/footer 固定用 `sidebar-link`，收折僅由 CSS 淡出文字
- [x] 4.6 收折寬度改 80px 對齊既有 icon 軸（中心 x=40）：移除置中規則、收折按鈕 `left: 24px`，消除收折過程「先往右再往左」的水平漂移
- [x] 4.7 修正底部即時狀態綠點：`.realtime-dot` 加 `flex-shrink: 0`、pill `padding-left: 21px` 對齊 icon 軸、label 以 `max-width` 過渡收攏、刷新鈕收折時淡出，收折後綠點不再消失或位移
- [x] 4.8 釘選項目兩態統一為首字母 icon + pin 徽章；已開啟項目關閉鈕收折時改右上角浮動小圓鈕；「全部關閉」收折時淡出僅留分隔線
- [x] 4.9 Dashboard 與釘選區分隔線改為兩態皆顯示；`sidebar-link` 內距改 `margin-left: 2px + padding: 0 12px` 修正收折項目右緣被 1px 邊框裁切

## 5. Analytics 頁互動細節

- [x] 5.1 `ProjectView.tsx`：將 Analytics sub-tab 移至 Agents 之後（順序 Sessions → Plans/Specs → Agents → Analytics）
- [x] 5.2 `ProjectAnalyticsTab.tsx` + `App.css`：控制列由 grid 自動換欄改為 flex 水平排列，產生按鈕靠右對齊
- [x] 5.3 快速區間擴充為「近一週 / 本週 / 近一個月 / 本月」四項（`buildQuickRange` 新增 `thisWeek`、以週一為週起始），並補 `analytics.quickRange.last7d / thisWeek / last30d` 至 zh-TW / en-US
- [x] 5.4 `App.css`：趨勢圖 `.trend-chart-series--primary` 由硬編碼 `#6366f1` 改用 `var(--color-action-primary)`
- [x] 5.5 `App.css`：`.trend-chart-toggle` 補 hover / active（subtle bg + subtle border）、`justify-content: center` 文字置中與 `--motion-fast` 過場

## 6. 品質驗證

- [x] 6.1 執行 `bun run lint` 與前端 build，確認無新增警告
- [x] 6.2 手動走查：Dashboard、設定頁、專案頁於 dark/light 主題下的按鈕/選單/checkbox hover、側欄收折動畫與收折按鈕位置
- [x] 6.3 手動走查：Analytics 頁 sub-tab 順序、控制列水平排列、四個快速區間、趨勢圖數據線配色與圖例切換鈕 hover（dark/light 雙主題）
