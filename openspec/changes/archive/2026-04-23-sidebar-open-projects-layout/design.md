## Context

SessionHub 目前採用橫排 tab 列（位於主內容區頂部）來呈現已開啟的專案，側邊欄僅顯示釘選專案的快捷連結。兩者功能重疊，且 tab 列佔據了寶貴的垂直空間。本設計將兩者合一：所有已開啟的專案全部管理於 sidebar，消除 tab bar。

## Goals / Non-Goals

**Goals:**
- 移除頂部橫排 tab 列
- Sidebar 新增「已開啟」區塊（釘選在上、已開啟在下）
- 已開啟項目顯示 × 關閉按鈕，點擊關閉並切換至 dashboard
- 折疊狀態下已開啟項目以 icon button 呈現，hover 顯示 × 按鈕

**Non-Goals:**
- 不修改 Rust 後端
- 不改變 ProjectView 內子分頁（Sessions / Plans & Specs）的行為
- 不持久化「已開啟專案」狀態至磁碟（App reload 後重置為空）

## Decisions

### 決策 1：已開啟列表由 App.tsx 管理，傳入 Sidebar

`openProjectKeys: string[]` 狀態已存在於 `App.tsx`（`useState`）。繼續沿用此 state，新增 `onCloseProject` callback 傳至 Sidebar，由 Sidebar 呼叫。

**替代方案**：將 open list 移至 Context — 不採用，違反「IPC 與狀態集中在 App.tsx」架構慣例，且非必要。

### 決策 2：關閉專案後自動切換至 dashboard（若正在檢視被關閉的專案）

關閉 active project 時，`onCloseProject` 呼叫後由 App.tsx 判斷若 `activeView === projectKey` 則 `setActiveView("dashboard")`，與現有行為一致。

### 決策 3：移除 tab bar 後 workspace-header 仍保留

`workspace-header` 含有 Dashboard title / 專案名稱列，保留不變。移除的是 `.tab-bar` 及 `.tab-item` 相關 CSS 與 JSX，不影響 header。

### 決策 4：折疊 sidebar 中的 × 按鈕呈現方式

折疊時空間受限，已開啟項目的 icon button 採 hover 時於右上角顯示微型 × 的方式，避免佔位。釘選項目不顯示 × （釘選由設定管理，非此流程關閉）。

## Risks / Trade-offs

- [樣式回歸] 移除 `.tab-bar` 後若有其他地方隱性依賴其 height 撐開佈局，可能發生 layout 偏移 → 在移除後執行視覺回歸確認
- [UX 轉換成本] 現有使用者習慣橫排 tab，需適應側邊欄方式 → 變化直覺，預計影響低
