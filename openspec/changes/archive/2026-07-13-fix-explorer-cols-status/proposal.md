## Why

Plans & Specs Explorer 的 `Cols` 檢視模式（同樣邏輯也影響 `List` 模式）在渲染每一個節點時，統一呼叫 `resolveChangeAction()` 來決定狀態徽章與可執行的 slash command。這個函式原本只為「進行中的 OpenSpec change」設計，卻被無差別套用到 Archived Changes 與 Specs 兩個群組的節點上，導致兩個明顯的錯誤狀態顯示：

1. 已經位於 `changes/archive/` 目錄下的 change，進度 100% 時仍被判定為「可封存」，而不是「已封存」——因為判斷邏輯完全沒有讀取節點所屬的群組（是否為 archived）。
2. Specs 群組下的正規規格（僅含一個 `spec.md`，本身即完整）被誤判為缺少 `proposal.md` 而顯示「待 propose」——因為判斷邏輯把 spec 節點當成 change 節點在檢查 `hasProposal`。

## What Changes

- 新增節點類型判斷：Cols／List 模式渲染時，依節點所屬群組（Active / Archived / Specs）分派到對應的狀態判斷邏輯，而非一律呼叫同一個 `resolveChangeAction()`。
- `resolveChangeAction` 針對來自 Archived Changes 群組的節點，新增「已封存」狀態分支（不可再顯示「可封存」）。
- 新增/調整 Specs 群組節點的狀態判斷：spec 節點（僅有 `spec.md`）視為完整規格，不套用 change 專屬的 `hasProposal`/`hasDesign`/`hasTasks` 檢查，不顯示「待 propose」等 change 動作徽章。
- 「已封存」對應的樣式 tone 與既有「可封存」等徽章保持視覺一致的設計語彙（沿用 `tone` 分類機制）。

## Capabilities

### Modified Capabilities

- `plans-specs-explorer-layout`: Cols（與 List）模式下的 change/spec 狀態徽章判斷邏輯需正確區分 Active / Archived / Specs 三種節點來源，Archived 節點需顯示「已封存」而非「可封存」，Specs 節點不再套用 change 專屬的完整性檢查。

## Impact

- `src/components/PlansSpecsView.tsx`：`resolveChangeAction()`（約 31-52 行）、`renderColumnsPanel()`（約 590-706 行）、`renderListChangeRow()`（約 506-569 行）。
- `src/utils/buildTree.ts`：`buildOpenSpecTree()` 中 archived / specs 節點的建構方式（約 160-193 行），視需要可能新增欄位以標記節點類型供渲染端判斷。
- `src/types/index.ts`：如需在 `TreeNode` 上新增群組類型標記欄位。
