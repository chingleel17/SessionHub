## 1. 後端 Sisyphus 路徑修正

- [x] 1.1 `src-tauri/src/sisyphus.rs`：`list_files_with_ext()` 改回傳完整絕對路徑（`path.to_string_lossy()`），維持既有排序
- [x] 1.2 `src-tauri/src/types/sisyphus_openspec.rs`：`SisyphusNotepad` 新增 `issues_path: Option<String>`、`learnings_path: Option<String>`
- [x] 1.3 `src-tauri/src/sisyphus.rs`：`scan_sisyphus_internal()` 於組裝 notepad 時填入 `issues.md` / `learnings.md` 的絕對路徑（檔案不存在則為 `None`）
- [x] 1.4 `src-tauri/src/lib.rs`：更新 `scan_sisyphus_*` 測試，`evidence_files` / `draft_files` 斷言改以 `project_dir` 組合路徑或 `ends_with` 比對；新增 notepad 路徑欄位斷言
- [x] 1.5 執行 `cargo test` 確認後端測試通過

## 2. 前端型別與 buildTree

- [x] 2.1 `src/types/index.ts`：`SisyphusNotepad` 補 `issuesPath: string | null`、`learningsPath: string | null`
- [x] 2.2 `src/types/index.ts`：`TreeNode` 新增 `sourceKind?: "sisyphus" | "openspec"`
- [x] 2.3 `src/utils/buildTree.ts`：`buildSisyphusTree()` 為其產出的所有節點（群組、entry、葉節點）標記 `sourceKind: "sisyphus"`；`buildOpenSpecTree()` 同樣全面標記 `"openspec"`
- [x] 2.4 `src/utils/buildTree.ts`：`buildSisyphusTree()` 移除 evidence 區塊
- [x] 2.5 `src/utils/buildTree.ts`：notepad 由葉節點改為帶 children 的節點，依 `issuesPath` / `learningsPath` 產生可點擊子節點（路徑為 null 者不產生節點）
- [x] 2.6 `src/components/PlansSpecsView.tsx`：`rootNodes` 的兩個來源根節點加上對應 `sourceKind`

## 3. Explorer 來源分層（List / Cols）

- [x] 3.1 `PlansSpecsView.tsx`：新增 `sourceSections` memo（`Array<{ sourceNode, groups }>`），取代 `allColsStatusGroups` 與 `renderListView()` 內重複的攤平迴圈
- [x] 3.2 `PlansSpecsView.tsx`：新增來源層展開狀態 state，預設所有來源展開
- [x] 3.3 `PlansSpecsView.tsx`：`renderListView()` 以 `sourceSections` 渲染，最外層為可折疊來源標頭
- [x] 3.4 `PlansSpecsView.tsx`：`renderColumnsPanel()` 第一欄以 `sourceSections` 渲染，來源標頭可折疊、其下為狀態群組手風琴（維持全域單選展開），第二欄結構不變
- [x] 3.5 `src/styles/`：新增 `explorer-list-source-*` 與 `explorer-cols-source-*` 標頭樣式，沿用既有設計 token 與 `tree-group-arrow` 指示符

## 4. OpenSpec 專屬邏輯限縮

- [x] 4.1 `PlansSpecsView.tsx`：`resolveChangeAction()` 的呼叫條件由 `entryNode.icon === "spec"` 改為僅在項目屬於 OpenSpec 來源且非 spec 時套用
- [x] 4.2 `PlansSpecsView.tsx`：`renderListChangeRow()` 依 `item.sourceKind` 判斷，對 Sisyphus 來源項目不渲染 artifact chip 與 `n specs` 計數，僅顯示名稱並保留點擊開啟行為
- [x] 4.3 `PlansSpecsView.tsx`：Cols 第二欄點擊 Sisyphus 項目時直接以 `getSelectableNode()` 開啟，不走 `:tasks` 子節點推導

## 5. 狀態同步去除寫死深度

- [x] 5.1 `PlansSpecsView.tsx`：`columnsOpenGroups` 初始值改為空集合，新增 effect 依 `sourceSections` 推導預設群組（優先 `openspec:active-changes`，否則第一個可見群組）
- [x] 5.2 `PlansSpecsView.tsx`：selectedNode 同步 effect 改以反查方式從 `findNodePath()` 結果中找出所屬狀態群組與 change，不再使用 `path[1]` / `path[2]` 固定 offset
- [x] 5.3 `PlansSpecsView.tsx`：選取節點時一併展開其所屬來源層級
- [x] 5.4 `PlansSpecsView.tsx`：`hasSisyphus` / `hasOpenSpec` 改由 builder 產出推導（`sisyphusNodes.length > 0` / `openspecNodes.length > 0`），取代現行各自維護的欄位條件清單
- [x] 5.5 `PlansSpecsView.tsx`：`columnsChangeId` 自動選取 effect 改用 `sourceSections`，並排除已折疊來源層底下的群組，避免收折 Sisyphus 後仍自動選中其項目

## 6. 測試與驗證

- [x] 6.1 新增 `tests/buildTree.test.ts`：涵蓋 sourceKind 標記、notepad 展開為子節點、evidence 已排除、drafts 節點帶絕對路徑
- [x] 6.2 執行前端測試與 `bun run build`（或專案既有 lint / typecheck 指令）確認通過
- [x] 6.3 手動驗證：同時具備 `.sisyphus` 與 `openspec` 的專案，於 Tree / List / Cols 三模式確認兩來源各自可展開收折，且 Sisyphus 項目無 OpenSpec 動作徽章
- [x] 6.4 手動驗證：點擊 Sisyphus plan、notepad 的 issues/learnings、draft 皆能正確載入內容
