## Context

ProjectView 的 sessions sub-tab 目前結構：
- `.project-page`（CSS Grid，gap: 6px）
  - `.sub-tab-bar`
  - `.toolbar-card`（2 欄 grid，padding: 20px 24px）包含 ProjectStatsBanner + filter-bar
  - `.tag-filter-bar`（有 tag 時顯示）
  - `.session-list`（session 卡片列表）

所有元素都在 `.workspace-content`（overflow-y: auto）內捲動。toolbar-card 目前會隨 session 卡片一起消失在視野外，使用者需捲回頂部才能調整篩選條件。此外 toolbar-card 的高度約 120–140px，佔用大量版面。

## Goals / Non-Goals

**Goals:**
- toolbar-card 與 tag-filter-bar 在 session 列表捲動時固定於頁面上方（sticky）
- 將 toolbar-card 高度縮減約 50%（目標 60–70px）
- 將 ProjectStatsBanner + filter-bar 整合為緊湊的單一橫列佈局
- 保持現有所有篩選邏輯不變（邏輯層不動，只改 CSS 與 HTML 結構）

**Non-Goals:**
- 不改動篩選邏輯、排序邏輯、tag 篩選邏輯
- 不改動 Rust 後端或 Tauri IPC
- 不增加新的篩選功能
- 不修改 Plans & Specs 或 Plan Editor sub-tab 的佈局

## Decisions

### 決策 1：使用 `position: sticky` 而非 `position: fixed`

**選擇**：`position: sticky; top: 0` 相對於 `.workspace-content` 捲動容器。

**理由**：sticky 定位不需要指定寬度、不會脫離文件流，且自然繼承父容器的寬度。fixed 需計算 sidebar 寬度，在視窗縮放或 sidebar 折疊時容易出現偏移問題。

**做法**：將 `.toolbar-card` + `.tag-filter-bar` 包裝在一個 `.sticky-filter-header` div 內，該 div 套用 `position: sticky; top: 0; z-index: 10; background: var(--color-surface-app)`。sticky 相對於最近的捲動祖先 `.workspace-content` 生效。

**替代方案考慮**：直接在 `.toolbar-card` 加 sticky → 不可行，因為 tag-filter-bar 是 toolbar-card 的同級元素，兩者需一起固定。

### 決策 2：扁平化 toolbar-card 為單行 flex 佈局

**選擇**：將 `.toolbar-card` 的 `display: grid; grid-template-columns: repeat(2, 1fr)` 改為 `display: flex; align-items: center; flex-wrap: wrap; gap: 8px`，並降低 padding 至 `8px 16px`。

**理由**：目前 2 欄 grid 在 ProjectStatsBanner 與 filter-bar 之間產生垂直空間，是高度過大的主因。flex 單行可以讓 stats + 所有篩選控件在同一行，高度自然縮減到約 40–48px（加上 padding 約 56–64px）。

**ProjectStatsBanner 調整**：移除 `margin-bottom: 12px`，使其成為 flex 行中的一段文字。

**替代方案考慮**：保留 2 欄 grid 但縮小 padding → 仍會有兩行，高度縮減有限（約 80–90px）。

### 決策 3：tag-filter-bar 也加入 sticky-filter-header

**選擇**：tag-filter-bar 與 toolbar-card 一起放進 `.sticky-filter-header`。

**理由**：tag 篩選也是使用者頻繁操作的控件，若 tag-filter-bar 捲走但 toolbar-card 固定，使用者體驗不一致。

## Risks / Trade-offs

- **z-index 衝突**：如果頁面上有 dropdown 或 popover（如 launcher menu），其 z-index 需高於 sticky header 的 z-index（10）。目前 launcher menu overlay 的 z-index 需確認 > 10。
  → 緩解：sticky header 使用 z-index: 10，launcher menu 保持更高 z-index（如 100+）。

- **sticky 在某些瀏覽器的 grid 子元素行為**：如果 `.project-page` 的 grid row 高度剛好等於 sticky 元素高度，sticky 可能不生效。
  → 緩解：用 wrapper div `.sticky-filter-header` 作為 sticky 元素（整體是一個 grid item），而非讓 toolbar-card 本身 sticky。

- **flex-wrap 時高度增加**：小螢幕或長 filter-bar 時 flex 換行，高度超過目標。
  → 緩解：設定 `min-width` 讓搜尋框不過度縮小，讓排版在合理視窗寬度（≥ 800px）下維持單行。

## Migration Plan

1. 修改 `src/components/ProjectView.tsx`：在 sessions sub-tab 內，用 `<div className="sticky-filter-header">` 包裝 `toolbar-card` + `tag-filter-bar`
2. 修改 `src/App.css`：
   - 新增 `.sticky-filter-header` sticky 樣式
   - 修改 `.toolbar-card` 為 flex 單行、縮小 padding
   - 移除 `.project-stats-banner` 的 `margin-bottom`
   - 縮小 `.compact-field`、`.compact-checkbox` 的 `min-height`
3. 確認 launcher menu overlay z-index > 10

無需 rollback strategy（純 CSS/HTML 結構，無資料遷移）。

## Open Questions

- 是否需要讓 tag-filter-bar 在 sticky header 中可收合（collapse）？→ 本次 **不實作**，保持現有展開行為。
- sticky header 背景是否需要 backdrop-filter blur？→ 本次保持純色背景，視覺效果足夠。
