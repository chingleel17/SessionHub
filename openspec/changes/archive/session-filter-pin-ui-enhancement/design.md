## Context

SessionHub 目前已有 session 列表、篩選（搜尋、標籤、封存切換）、session 動作（開啟終端、複製指令、封存、刪除）以及專案分組（tabbed-ui）。

現況痛點：
1. 無法快速識別並清除「從未開始對話」的空 session（summaryCount = 0），需逐一手動比對
2. 動作按鈕為純文字，資訊密度低，在卡片上佔用大量空間
3. 沒有「我的常用專案」概念，每次啟動都要重新找到目標專案

技術現況：
- 前端狀態集中於 `App.tsx`，子元件透過 props 驅動（禁止子元件直接 invoke）
- `AppSettings` 存於 `%APPDATA%\SessionHub\settings.json`，透過 `save_settings` / `get_settings` 讀寫
- session 篩選邏輯目前在 `App.tsx` 前端過濾（React state）
- icon 資源目前全部為文字 / emoji

## Goals / Non-Goals

**Goals:**
- 新增前端 filter：可切換「隱藏空 session（summaryCount = 0）」
- 新增後端 command `delete_empty_sessions`：批次刪除所有 summaryCount = 0 的 session 資料夾
- 所有 action 按鈕（開啟終端、複製、封存、取消封存、刪除）改為 SVG icon button，hover 顯示 tooltip
- 新增「釘選專案」功能：專案 header 可釘選，釘選後出現在 sidebar 快速入口區；釘選狀態存於 settings.json `pinnedProjects` 陣列
- 支援釘選專案也在頂部 tab 以特殊樣式標示（置頂排序）

**Non-Goals:**
- 不實作 drag-and-drop 排序釘選清單（手動控制順序）
- 不實作 icon theme 切換（統一使用 inline SVG）
- 不修改 session 的實際存放格式或 workspace.yaml
- 不做 session 分頁虛擬化（效能改善留待後續）

## Decisions

### D1：空 session 篩選放前端 vs 後端

**決定：前端 filter state**

`summaryCount` 已由後端回傳至前端，前端直接過濾即可（與現有 showArchived 模式一致）。不新增後端 command，減少 IPC 往返。

批次「刪除」空 session 才需新增後端 command（需存取 FS），使用 `delete_empty_sessions` 回傳刪除數量。

### D2：Icon 方案

**決定：inline SVG component**

原因：
- 無需引入 icon library 套件（YAGNI，避免增加 bundle）
- Tauri app 離線執行，CDN icon font 不可靠
- SVG 可直接以 CSS currentColor 繼承主題顏色

實作：建立 `src/components/Icons.tsx`，匯出具名 SVG function component（`TerminalIcon`、`CopyIcon`、`ArchiveIcon`、`UnarchiveIcon`、`DeleteIcon`、`PinIcon`、`UnpinIcon`）。

### D3：釘選狀態存放位置

**決定：AppSettings.pinnedProjects: string[]**

`pinnedProjects` 為 project key（即 session.cwd 路徑字串）的陣列，存入既有的 `settings.json`。
- 不需新建 DB table，向下相容（settings.json 舊版不含此欄位時預設空陣列）
- 與 showArchived 一樣屬於 UI 偏好，放設定檔最合理

### D4：釘選顯示位置

**決定：Sidebar 快速入口 + Tab 置頂**

- Sidebar：在現有 Sessions / Settings 導覽項目之間加入「釘選專案」區段，直接顯示釘選專案名稱，點擊切換至該專案 tab
- Tab 列：釘選的專案 tab 以特殊 CSS class 標示（加上 pin icon），並排序置頂

## Risks / Trade-offs

- **[Risk] pinnedProjects 路徑字串與實際目錄不同步** → 顯示時若 cwd 不存在對應 session group，靜默忽略（不顯示）即可；刪除所有 session 後釘選自動消失
- **[Risk] delete_empty_sessions 誤刪** → 刪除前以確認對話框顯示預計刪除數量，與現有刪除 pattern 一致
- **[Trade-off] SVG inline 增加元件程式碼量** → 集中於 `Icons.tsx` 便於維護，不分散到各元件

## Migration Plan

1. 前端型別擴充：`AppSettings` 加入 `pinnedProjects?: string[]`（選擇性，向下相容）
2. 後端 Rust struct `AppSettings` 加入 `#[serde(default)] pinned_projects: Vec<String>`
3. 新增 Tauri command `delete_empty_sessions` → 回傳刪除數量 `usize`
4. 新增 `Icons.tsx`，逐一替換各元件的文字 action 按鈕
5. 新增 filter state `hideEmptySessions` 及對應 UI 切換開關
6. 新增 pinned projects 邏輯（toggle pin、sidebar 區塊、tab 排序）
7. 無資料遷移需求，無 rollback 風險（settings.json 欄位選擇性）

## Open Questions

- 無
