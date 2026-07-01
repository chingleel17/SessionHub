## Context

Explorer 面板由 `PlansSpecsView.tsx` 管理，Tree / List / Cols 三種模式共用同一份 `rootNodes`（由 `buildOpenSpecTree` 產生）。change 清單來自 `openspecData.activeChanges / archivedChanges`，目前按後端掃描順序排列（即目錄名稱字母序）。

CSS 位於 `src/App.css`，Tree 模式的群組標籤由 `.explorer-list-group-label` 套用 `font-weight: 700`，而 List 的 `.explorer-list-row-name` 與 Cols 的 `.explorer-cols-entry-name` 目前沒有設定 font-weight（繼承 inherit，通常為 400）。

## Goals / Non-Goals

**Goals:**
- List 與 Cols 的 change 名稱加粗（`font-weight: 600`）
- Explorer 標頭新增排序切換器，支援三欄位：進度（taskProgress.done/total）、名稱（字母序）、建立時間（.openspec.yaml `created` 欄位）
- 點擊已選中欄位切換升降冪，點擊新欄位預設升冪
- 排序設定 per-project 持久化於 localStorage
- 排序同時作用於 Tree / List / Cols 三種模式（在傳入 buildOpenSpecTree 前先排序）

**Non-Goals:**
- sisyphus 區段不排序
- 後端不做排序（排序在前端進行）

## Decisions

### 決策 1：時間排序的資料來源

**選擇：後端 Rust 讀取 `.openspec.yaml` 的 `created` 欄位，加入 `OpenSpecChange.createdAt: String`**

這是最可靠的建立時間來源（不依賴 OS mtime，在 git clone 後也正確）。需要：
- `src-tauri/src/types.rs`：`OpenSpecChange` 新增 `created_at: Option<String>`
- `src-tauri/src/openspec_scan.rs`：`scan_openspec_change` 讀取 `.openspec.yaml` 解析 `created` 欄位
- `src/types/index.ts`：`OpenSpecChange` 新增 `createdAt?: string | null`

**替代方案：用目錄 mtime**

拒絕：git clone 後 mtime 全部一樣，排序無意義。

### 決策 2：排序 state 的存放位置

**選擇：`PlansSpecsView` 內部 state**，以 `localStorage` 持久化（key: `explorer-sort:<projectCwd>`，值: `{ field: "progress"|"name"|"createdAt", dir: "asc"|"desc" }`）

排序是純前端的 UI 偏好，不影響後端資料。

### 決策 3：排序切換器的互動設計

**UI：** 三個按鈕並排（`進度 ↑↓` / `名稱 ↑↓` / `時間 ↑↓`），圖示由 CSS 字符表示。已選中欄位顯示目前方向箭頭（↑升冪 / ↓降冪），未選中欄位顯示中性圖示（⇅）。

**點擊行為：**
- 點擊已選中欄位 → 切換方向
- 點擊未選中欄位 → 設為該欄位升冪

### 決策 4：進度排序的計算方式

以 `done / total`（完成率）排序，`total === 0`（無 tasks）視為 `-1`（升冪時排在最後）。

## Risks / Trade-offs

- [風險] `created_at` 欄位為 `Option<String>`，舊有 change 若無此欄位時間排序退化為維持原序 → 可接受，新 change 都有此欄位
- [取捨] 排序切換器佔用標頭空間 → 設計為緊湊小按鈕，與 view 切換器同列
