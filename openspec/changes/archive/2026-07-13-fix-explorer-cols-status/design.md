## Context

`PlansSpecsView.tsx` 的 Cols（`renderColumnsPanel`）與 List（`renderListChangeRow`）兩種檢視模式，在渲染 `openspec:*` 群組（Active Changes / Archived Changes / Specs）內的每一個節點時，統一呼叫 `resolveChangeAction(entryNode: TreeNode)` 來得出狀態徽章文字與可複製的 slash command。

該函式只依賴 `entryNode.children` 是否含有 `icon === "proposal"` / `"tasks"` 的子節點、以及 `entryNode.progress`，完全不知道這個節點是從哪個群組來的：

- Active Changes 與 Archived Changes 節點的 `icon` 都是 `"change"`，唯一差異在於 `id` 前綴（`openspec:change:` vs `openspec:archived:`，見 `buildTree.ts` 143-177 行），`resolveChangeAction` 沒有讀取這個前綴。
- Specs 節點（`buildTree.ts` 179-193 行）是純葉節點：`icon: "spec"`、無 `children`、無 `progress`，本質上不是「change」，卻仍被丟進同一個判斷函式。

`TreeNode` 型別（`types/index.ts` 389-400）目前沒有欄位可以直接表示「這個 change 節點屬於哪個群組」或「這是 spec 而非 change」，唯一可用的訊號是既有的 `icon` 欄位與 `id` 字串前綴。

## Goals / Non-Goals

**Goals:**
- Archived Changes 群組內、進度 100% 的 change 節點，狀態徽章顯示「已封存」，不再顯示「可封存」。
- Specs 群組內的節點（僅 `spec.md`）不再套用 change 專屬的 proposal/design/tasks 完整性檢查，不顯示「待 propose」等 change 動作徽章與可複製指令。
- 不改變 Active Changes 群組既有的「待 propose / 可 apply / 進行中 / 可封存」判斷邏輯與行為。

**Non-Goals:**
- 不重新設計 Explorer 的整體資料結構（`TreeNode` 仍沿用現有欄位為主，只做最小必要擴充）。
- 不處理 Tree 與 List 模式：`resolveChangeAction` 目前僅在 Cols 模式（`renderColumnsPanel`，第 663 行）被呼叫；List 模式的 `renderListChangeRow`（第 506-569 行）走的是完全不同的 artifact-chips 渲染路徑，未呼叫此函式，不受本次兩個 bug 影響，故不在此次變更範圍。
- 不新增「已封存」之後的其他狀態（如封存失敗、封存中）。

## Decisions

**1. 用 `icon` 分派節點類型，而非新增 `nodeKind` 欄位**

`resolveChangeAction` 呼叫前，先用既有 `entryNode.icon` 判斷：
- `icon === "spec"` → 直接視為完整規格，回傳一個新的、不含 propose/apply/archive 動作的結果（例如 `null` 或一個「無動作」的 action 物件），渲染端據此不顯示動作徽章列。
- `icon === "change"` → 才進入 `resolveChangeAction` 既有邏輯。

理由：`icon` 欄位已經精準區分 spec vs change 節點，不需要為此新增型別欄位；改動面最小。

備選方案（否決）：在 `TreeNode` 新增 `kind: "change" | "spec"` 欄位。否決原因：與既有 `icon` 語意重複，徒增資料維護成本。

**2. 用 `id` 前綴判斷 archived，並新增 `isArchived` 布林參數傳入 `resolveChangeAction`**

在呼叫端（`renderColumnsPanel` / `renderListChangeRow` 所在的 group 迭代處，已持有 `groupNode.id`）判斷 `groupNode.id === "openspec:archived-changes"`，將 `isArchived: boolean` 一併傳給 `resolveChangeAction(entryNode, isArchived)`。

`resolveChangeAction` 內部邏輯調整為：
```
if (!hasProposal) → "待 propose"
if (!hasTasks || !progress) → "可 apply"
if (progress.done >= progress.total && progress.total > 0) {
  return isArchived ? "已封存" : "可封存"
}
→ "進行中 x/y"
```

理由：群組來源（`groupNode.id`）在呼叫端是現成資訊，不需要反過來讓 `buildTree.ts` 或 `TreeNode` 攜帶額外狀態；改動集中在 `PlansSpecsView.tsx` 一處函式簽章。

備選方案（否決）：在 `changeToNodes`/`buildOpenSpecTree` 產生節點時就把 `isArchived` 寫進 `TreeNode`（新增欄位）。否決原因：`TreeNode` 是通用樹節點型別，被 Sisyphus、Agents 等其他資料來源共用，為單一 OpenSpec 用途加欄位會擴大型別的影響面；用呼叫端已知的 `groupNode.id` 判斷更局部。

**3. 新增「已封存」的 tone**

沿用既有 `tone: "done"`（與「可封存」相同視覺表現，畢竟都代表 100% 完成的終態），只有 label 文字從「可封存」改為「已封存」，且「已封存」狀態不提供可複製的 `/opsx:archive` 指令（因為已經封存過，該指令重跑沒有意義）。

## Risks / Trade-offs

- [風險] `groupNode.id` 是字串魔法值（`"openspec:archived-changes"`），未來若 group id 命名調整，判斷會悄悄失效 → [緩解] 從 `buildTree.ts` 匯出一個常數（如 `ARCHIVED_CHANGES_GROUP_ID`）供兩處共用，避免字串硬編碼分散。
- [風險] Specs 節點不再顯示任何動作徽章，若使用者原本（即使是誤判）依賴「待 propose」點擊去複製指令，會需要改用其他入口 → 可接受，因為這是修正誤判，不是移除正確功能。
