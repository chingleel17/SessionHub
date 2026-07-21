## Context

Plan 介面的 Cols 模式由 `PlansSpecsView.tsx` 實作，左側 master 面板列出各 change 的手風琴群組，右側 detail 面板用 `ExplorerTree` 顯示選中 change 的 artifact 子節點。

目前點擊 change 僅觸發 `setColumnsChangeId(entryNode.id)`，右側雖會切換到正確的 change，但沒有自動選取任何 artifact；使用者需要再手動點 tasks 才能看到內容。

tasks.md 進度的即時同步路徑如下：

```
tasks.md 寫入
  → watcher.rs is_relevant_project_event (openspec/ 目錄已覆蓋)
  → Tauri emit "project-files-changed"
  → App.tsx unlisten callback → refreshProjectPlansSpecs()
  → queryClient.invalidateQueries(["project_specs", pathLabel])
  → openspecQuery 重新執行 get_project_specs
  → buildOpenSpecTree → TreeNode.progress / badge / tone 更新
  → UI 重繪
```

後端 watcher 與前端監聽機制已正確存在，但需要確認一個潛在問題：`openspecQuery` 的 queryKey 為 `["project_specs", activeProject?.pathLabel ?? ""]`，而 `refreshProjectPlansSpecs` invalidate 時傳入的 `projectDir` 必須與 `activeProject.pathLabel` 完全一致，否則不會觸發重整。

## Goals / Non-Goals

**Goals:**
- Cols 模式點擊 change 後，自動選取並載入 tasks artifact（節點 id 格式：`openspec:change:<name>:tasks`）
- 確保 tasks.md 更新後，左側進度條與 badge 能在 watcher 觸發後約 600ms 內（100ms debounce + 500ms delay）即時更新

**Non-Goals:**
- 不改動 Tree 模式或 List 模式的選取行為
- 不修改 watcher.rs 的監聽邏輯（已正確覆蓋 openspec/）
- 不修改 staleTime（invalidateQueries 會忽略 staleTime 直接重整）

## Decisions

### 決策 1：自動選取 tasks 的觸發時機

**選擇：在 Cols 的 `onClick` 中直接衍生並呼叫 `handleSelect`**

當使用者點擊 change 時，從 `entryNode.children` 中找到 id 包含 `:tasks` 的子節點，取得其最深層可選取節點（`getSelectableNode`），再呼叫 `handleSelect`。

**替代方案：useEffect 監聽 `columnsChangeId` 變化**

拒絕理由：useEffect 會在每次 change 切換時都觸發，包含初始載入時自動選第一個 change 的場景，可能造成非預期的內容載入。直接在 onClick 中觸發更可控，且只在使用者主動點擊時發生。

**條件：只在 tasks 節點存在時才自動選取**，避免尚未建立 tasks.md 的 change 出現錯誤。

### 決策 2：進度即時更新的確認與修補

**選擇：確認 `refreshProjectPlansSpecs` 的 pathLabel 一致性**

檢查 App.tsx 中 `project-files-changed` 事件收到的 payload（即 `project_dir`）與 `activeProject.pathLabel` 是否為同一字串格式（大小寫、斜線方向）。若不一致，需在比較前正規化路徑。

## Risks / Trade-offs

- [風險] 使用者點擊 change 但 tasks 尚未建立 → 緩解：只在 tasks 子節點存在時才呼叫 `handleSelect`，否則僅切換 change
- [風險] Windows 路徑大小寫或斜線不一致導致 invalidate miss → 緩解：比較前先用 `.toLowerCase().replace(/\\/g, "/")` 正規化
- [取捨] 自動選取 tasks 會覆蓋使用者原本選取的其他 artifact → 接受此行為，因 tasks 是切換 change 後最常查看的項目
