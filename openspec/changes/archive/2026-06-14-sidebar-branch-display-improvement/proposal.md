## Why

目前左側欄以 repo_root 為鍵值分組，但同一個 repo 下有多個 worktree（不同分支）時，它們共用相同的 repo_name，導致左側欄出現多個相同名稱的項目，使用者無法辨識。此外，分支資訊目前顯示在每張 session card 上，但同一目錄下的 sessions 分支相同，這是重複的資訊——分支標籤應顯示在 ProjectGroup 標題旁，不應重複放在 card 上。

## What Changes

- **分組鍵值調整**：將 ProjectGroup 的 key 從純 `repo_root` 改為 `repo_root + ":" + git_branch`，確保不同 worktree / 分支各自獨立分組
- **Sidebar 項目顯示**：左側欄項目改為顯示 `repo_name (branch)` 或 `repo_name · branch`，讓同名 repo 的不同分支可明確區分
- **ProjectView 標題顯示**：主內容區標題列改為 `repo_name` 大標 + `branch` 小標（副標題），對齊截圖設計
- **移除 session card 上的分支欄位**：`SessionCard` 不再顯示 `分支` 欄位，因為 group 層已有此資訊
- **`branchLabel` 欄位更新**：`ProjectGroup.branchLabel` 改為單一確定的分支值，不再做 `+N` 聚合（因為已按分支分組）

## Capabilities

### New Capabilities

- `project-group-branch-key`: ProjectGroup 以 `repoRoot:branch` 為唯一鍵，確保不同 worktree 正確分離
- `sidebar-branch-display`: Sidebar 項目標籤顯示 `repo (branch)` 格式，區分同名 repo 的多個分支

### Modified Capabilities

- (無現有 spec 需要 delta，此為全新 UX 行為)

## Impact

- `src/App.tsx`：`getProjectKey`、`getProjectBranchLabel`、`buildProjectGroups` 邏輯修改
- `src/components/Sidebar.tsx`：項目標籤渲染邏輯，加入分支副標籤
- `src/components/ProjectView.tsx`：標題區塊加入分支副標題顯示
- `src/components/SessionCard.tsx`：移除分支欄位顯示
- `src/types/index.ts`：`ProjectGroup.branchLabel` 語義確認（不需型別變更）
