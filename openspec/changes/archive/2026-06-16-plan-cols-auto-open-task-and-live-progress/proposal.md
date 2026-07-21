## Why

Cols 模式在點擊不同 change 時，右側 detail 面板雖然會切換到對應 change 的子節點，但不會自動選取 tasks artifact 並顯示內容，導致使用者需要手動再點一下；另外，當手動更新 tasks.md 的 checkbox 進度後，左側進度條與徽章未能即時反映，需要切換頁面才能看到更新。

## What Changes

- 點擊 Cols 模式中的 change 項目時，自動選取並展示該 change 的 `tasks` artifact（若存在）
- 修正左側進度更新路徑，確保 tasks.md 檔案變更後能即時觸發 openspecQuery 重整，讓 badge 與進度條即時同步

## Capabilities

### New Capabilities

- `cols-auto-select-task`: Cols 模式點擊 change 時，自動選取並載入 tasks artifact 的內容

### Modified Capabilities

- `live-progress-sync`: 確認並修正 tasks.md 變更後的即時進度同步流程（watcher → project-files-changed → openspecQuery invalidate → UI 更新）

## Impact

- `src/components/PlansSpecsView.tsx`：Cols 模式的 `onClick` handler，新增自動選取 tasks 邏輯
- `src/App.tsx`：確認 `refreshProjectPlansSpecs` 是否正確 invalidate `openspecQuery`（`["project_specs", pathLabel]`）
- `src-tauri/src/watcher.rs`：確認 `is_relevant_project_event` 是否覆蓋 tasks.md 路徑（已覆蓋 openspec/ 目錄，無需修改）
