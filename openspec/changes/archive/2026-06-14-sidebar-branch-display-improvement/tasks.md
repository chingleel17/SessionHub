## 1. App.tsx 分組邏輯

- [x] 1.1 修改 `getProjectKey()`：回傳 `normalizePath(repoRoot) + ":" + (gitBranch ?? "")` 取代純 repoRoot
- [x] 1.2 修改 `getProjectBranchLabel()`：移除 `+N` 聚合邏輯，直接回傳 sessions 中第一個非空 gitBranch（因已按分支分組，應全部相同）
- [x] 1.3 確認 `getProjectTitle()` 邏輯不受 key 格式變更影響（title 仍從 repoName 或路徑末段取得）

## 2. Sidebar 分支顯示

- [x] 2.1 在 Sidebar 展開狀態的項目標籤中，加入分支副標籤：`repoName · branch`，僅在 `branchLabel` 有值時顯示
- [x] 2.2 折疊狀態的 `title` tooltip 加入分支資訊（`group.title · group.branchLabel`）
- [x] 2.3 加入對應的 CSS：`.sidebar-branch-label` 以較小字體、次要色顯示，與主標籤水平排列

## 3. ProjectView 標題顯示

- [x] 3.1 在 `ProjectView` 標題區（project header）的 repo 名稱旁加入分支 badge（`<span class="project-branch-badge">`）
- [x] 3.2 僅在 `project.branchLabel` 有值時渲染 badge
- [x] 3.3 加入對應 CSS：`.project-branch-badge` 樣式（類似 tag/chip，輕量）

## 4. SessionCard 移除分支欄位

- [x] 4.1 從 `SessionCard` 中找到並刪除 `gitBranch` / 分支欄位的渲染程式碼
- [x] 4.2 確認移除後 card 版面無空白破版
