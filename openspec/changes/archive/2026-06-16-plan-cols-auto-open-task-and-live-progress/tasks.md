## 1. Cols 自動選取 tasks artifact

- [x] 1.1 在 `PlansSpecsView.tsx` 的 `renderColumnsPanel` 中，找到 `getSelectableNode` 函式（或 import）確認可用
- [x] 1.2 修改 Cols 模式 change 項目的 `onClick` handler：在 `setColumnsChangeId(entryNode.id)` 之後，從 `entryNode.children` 找到 id 包含 `:tasks` 的子節點
- [x] 1.3 若 tasks 子節點存在，呼叫 `getSelectableNode(tasksNode)` 取得最深層可選取節點，再呼叫 `handleSelect(selectableNode)` 自動載入內容
- [x] 1.4 若 tasks 子節點不存在，只執行 `setColumnsChangeId`，不觸發 `handleSelect`

## 2. 即時進度同步確認與修補

- [x] 2.1 在 `App.tsx` 的 `project-files-changed` 事件 handler 中，加入路徑正規化比較：將 `event.payload` 與 `activeProject.pathLabel` 皆轉為小寫並將 `\\` 替換為 `/` 後再比較
- [x] 2.2 確認 `refreshProjectPlansSpecs` 傳入的 `projectDir` 字串與 `openspecQuery` queryKey 中的 `activeProject.pathLabel` 格式一致（若已一致則此步驟標記為完成）
- [x] 2.3 手動測試：更新 tasks.md checkbox → 觀察左側進度是否在 1 秒內自動更新

## 3. 驗證

- [x] 3.1 測試 Cols 模式：點擊有 tasks.md 的 change → 確認右側自動顯示 tasks 內容
- [x] 3.2 測試 Cols 模式：點擊無 tasks.md 的 change → 確認不報錯，右側顯示其他 artifact 或空狀態
- [x] 3.3 測試 Cols 模式：從 change A 切換到 change B → 確認 tasks 內容正確切換
- [x] 3.4 測試進度即時更新：在外部編輯器修改 tasks.md → 確認左側 badge 與進度條即時更新
