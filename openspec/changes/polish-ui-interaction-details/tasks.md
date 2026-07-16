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

## 5. 品質驗證

- [x] 5.1 執行 `bun run lint` 與前端 build，確認無新增警告
- [x] 5.2 手動走查：Dashboard、設定頁、專案頁於 dark/light 主題下的按鈕/選單/checkbox hover、側欄收折動畫與收折按鈕位置
