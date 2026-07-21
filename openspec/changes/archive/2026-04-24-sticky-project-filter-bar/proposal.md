## Why

ProjectView 的篩選工具列（toolbar-card + tag-filter-bar）目前會隨 session 卡片一起捲動，導致使用者在長列表中無法快速修改搜尋或排序條件。同時 toolbar-card 的高度偏高，佔用過多版面空間。

## What Changes

- **toolbar-card 固定在頁面頂部**：在 session sub-tab 中，toolbar-card 與 tag-filter-bar 改為黏著（sticky）定位，使其在 session 列表捲動時保持可見。
- **toolbar-card 高度縮減約一半**：減少 padding、壓縮 ProjectStatsBanner 與 filter-bar 的垂直空間，使整體高度從約 120px 降至 60px 左右。
- **排版優化**：統一 filter-bar 內各元素的對齊方式，將 ProjectStatsBanner 與 filter-bar 整合成單行或更緊湊的雙行佈局。

## Capabilities

### New Capabilities

- `sticky-filter-toolbar`: 篩選工具列（toolbar-card + tag-filter-bar）在 session 列表捲動時固定於頂部，且保持視覺層級（z-index + 背景遮蓋）。

### Modified Capabilities

- `session-filter`: 篩選列高度縮減、排版優化，UI 行為不變（現有篩選邏輯不改動）。

## Impact

- `src/components/ProjectView.tsx`：調整 toolbar-card 與 tag-filter-bar 的 CSS 類別或 style，改為 sticky 定位。
- `src/App.css`：修改 `.toolbar-card`、`.tag-filter-bar`、`.filter-bar`、`.compact-field`、`.compact-checkbox` 的 CSS，縮減 padding 與高度。
- `src/components/ProjectStatsBanner.tsx`：可能需調整 stats banner 的 padding 或字型大小以配合緊湊佈局。
- 無 API / Rust 後端變更。
