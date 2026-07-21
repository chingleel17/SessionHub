## Why

目前應用程式使用頂部橫排 tab 列來管理已開啟的專案，造成空間浪費且與側邊欄的釘選專案導覽重複。將已開啟的專案整合至側邊欄可統一導覽體驗、節省垂直空間，並讓使用者在單一位置管理所有專案（釘選與已開啟）。

## What Changes

- **移除**頂部橫排專案 tab 列（`tab-bar`）
- Sidebar 新增「已開啟的專案」區塊，顯示所有目前開啟中的專案
- 每個已開啟的專案項目旁新增 **×** 按鈕，可直接關閉
- Sidebar 版面調整：釘選專案區塊在上，已開啟專案區塊在下
- 已折疊的 sidebar 狀態下，已開啟的專案以 icon button 形式顯示（含 ×）
- Dashboard 保持為固定入口，不在已開啟清單中（與原本一致）

## Capabilities

### New Capabilities

- `sidebar-open-projects`: Sidebar 顯示已開啟專案清單，每個項目有關閉（×）按鈕，支援折疊模式

### Modified Capabilities

- `tabbed-ui`: 移除橫排 tab 列需求；專案切換與關閉改由 sidebar 負責

## Impact

- `src/components/Sidebar.tsx`：新增 `openProjectKeys` props、`onCloseProject` callback 與對應 UI
- `src/App.tsx`：移除 tab bar 的 JSX 渲染，傳遞 `openProjectKeys` 與 `onCloseProject` 至 Sidebar
- `src/styles/`：移除或停用 tab bar 相關 CSS（`.tab-bar`, `.tab-item`），新增 sidebar open-projects 樣式
- `openspec/specs/tabbed-ui/spec.md`：更新需求，反映 tab 列移除後的行為改變
