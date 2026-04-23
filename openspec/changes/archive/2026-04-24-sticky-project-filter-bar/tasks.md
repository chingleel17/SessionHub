## 1. CSS 修改

- [x] 1.1 在 `src/App.css` 新增 `.sticky-filter-header` 樣式：`position: sticky; top: 0; z-index: 10; background: var(--color-surface-app); padding-bottom: 6px`
- [x] 1.2 修改 `.toolbar-card`：改為 `display: flex; align-items: center; flex-wrap: wrap; gap: 8px; padding: 8px 16px`（移除 `grid-template-columns`，縮小 padding）
- [x] 1.3 修改 `.project-stats-banner`：移除 `margin-bottom: 12px`，讓它成為 flex row 中的一個項目
- [x] 1.4 修改 `.compact-checkbox`：縮小 `min-height` 至 `32px`（從 42px）
- [x] 1.5 確認 launcher menu overlay 的 z-index > 10（在 `[data-launcher-menu]` 相關 CSS 中確認或調整）

## 2. ProjectView.tsx 結構調整

- [x] 2.1 在 sessions sub-tab 的 `<>` Fragment 內，用 `<div className="sticky-filter-header">` 包裹 `.toolbar-card` section 與 `.tag-filter-bar` section
- [x] 2.2 確認 `session-list` div 不在 `sticky-filter-header` 內（只有 toolbar-card + tag-filter-bar 在裡面）

## 3. 驗證

- [x] 3.1 執行 `bun run build` 確認前端 TypeScript 無型別錯誤
- [x] 3.2 手動測試：向下捲動 session 列表，確認篩選工具列固定於頁面頂部
- [x] 3.3 手動測試：開啟 launcher menu，確認 overlay 正常顯示於工具列上方
- [x] 3.4 手動測試：切換到 Plans & Specs sub-tab，確認無多餘 sticky 佔位
- [x] 3.5 手動測試：toolbar-card 高度目測約 56–72px，較原先縮減約 50%
