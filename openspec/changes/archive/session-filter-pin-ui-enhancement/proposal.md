## Why

SessionHub 目前缺少快速篩選空 session（無對話記錄）的能力，且工具列操作（開啟終端、複製指令、封存、刪除）沒有圖示，視覺回饋不佳；同時缺少「釘選專案」功能，使用者無法快速回到常用專案。這三項改善可顯著提升日常使用效率。

## What Changes

- **新增 session 篩選選項**：在現有過濾列加入「排除無對話 session」切換，讓使用者快速隱藏 summary_count = 0 的空 session
- **新增批次刪除空 session**：提供一鍵刪除所有無對話 session 的操作，支援確認對話框
- **Action 按鈕加入 Icon**：開啟終端、複製指令、封存、刪除等操作全部改為 icon button，保留 tooltip 文字說明
- **釘選專案功能**：可將專案（project group）釘選到頂部 tab 列或側邊欄快速入口，釘選狀態持久化至 settings.json

## Capabilities

### New Capabilities
- `session-filter-empty`: 篩選並排除 summary_count = 0 的 session，以及批次刪除這類 session
- `action-icons`: 所有 session / project 動作按鈕改為帶有 icon 的視覺元素
- `pinned-projects`: 允許使用者釘選專案至頂部 tab 或 sidebar 快速入口，並持久化釘選清單

### Modified Capabilities
- `session-list`: 新增「排除空 session」過濾條件（修改篩選邏輯）
- `session-actions`: 新增「刪除空 session」批次動作
- `tabbed-ui`: 釘選專案後在 tab 區頂端顯示釘選 tab
- `app-settings`: settings.json 新增 `pinnedProjects: string[]` 欄位

## Impact

- **前端**：`src/App.tsx`（filter state、pinned projects state、invoke 呼叫）、`src/components/`（SessionCard、ProjectHeader、TabBar、Sidebar）、`src/types/index.ts`（Settings 型別新增 pinnedProjects）
- **後端**：`src-tauri/src/lib.rs`（`delete_empty_sessions` 新 command、`save_settings` / `get_settings` 支援 pinnedProjects）
- **設定檔**：`settings.json` schema 擴充
- **無 breaking changes**：pinnedProjects 預設空陣列，向下相容
