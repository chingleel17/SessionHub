## Context

目前 `getProjectKey()` 以純 `repoRoot`（正規化後）作為 ProjectGroup 的分組鍵。當使用者開啟同一 repo 的多個 worktree（每個 worktree 對應不同分支），因為 `repoRoot` 相同，所有 worktree session 被合併進同一個 ProjectGroup，sidebar 只會顯示一個項目名稱，無法區分。

目前 `branchLabel` 以 `+N` 聚合方式呈現多個分支，此資訊放在 session card 內是冗餘的——同一個 ProjectGroup 內所有 sessions 是相同目錄，分支只需顯示一次。

## Goals / Non-Goals

**Goals:**
- 同一 repo 的不同分支 / worktree 在 sidebar 顯示為獨立項目，名稱可區分
- 分支資訊移至 ProjectGroup 標題層顯示（ProjectView 大標題旁）
- 移除 session card 上的重複分支欄位

**Non-Goals:**
- 不更動 Rust 後端的 SessionInfo 結構（`gitBranch` 欄位已有）
- 不修改 SQLite schema
- 不處理 session 跨 worktree 移動（僅顯示改版）

## Decisions

**決策 1：分組鍵改為 `repoRoot:branch`**

- 做法：`getProjectKey()` 回傳 `normalizePath(repoRoot) + ":" + (gitBranch ?? "")`
- 替代方案：保留 repoRoot 鍵，在 ProjectGroup 內再細分 branch subgroup
- 選擇理由：現有架構 ProjectGroup 是平面列表，加入鍵值分離是最小改動路徑；sub-group 架構需要大量 UI 重構

**決策 2：Sidebar 標籤格式為 `repoName · branch`**

- 以中間點 `·` 分隔，折疊時縮圖顯示 `initial(branch[:2])`（如 `S·m`）
- 替代方案：括號 `repoName (branch)`
- 選擇理由：符合截圖設計，視覺上更輕量；括號格式較佔寬度

**決策 3：ProjectView 標題區改為雙行**

- `<h2>repoName</h2>` + `<span class="project-branch-badge">branch</span>` 水平並排
- 對齊截圖中 `social-platform` 大標 + 路徑副標題的佈局邏輯

**決策 4：SessionCard 移除分支欄位**

- 直接刪除 `session.gitBranch` 的渲染片段，CSS 不需調整
- ProjectGroup 層已顯示，card 不需重複

## Risks / Trade-offs

- [Risk] 已釘選（pinned）的 ProjectGroup key 若為舊格式（純 repoRoot），升級後會找不到對應 group → **Mitigation**: pinned keys 在找不到時靜默 fallback，不崩潰（現有邏輯 `projectGroups.find(g => g.key === key)` 回傳 undefined 已被過濾）
- [Risk] 無 gitBranch 的 sessions（非 git repo 目錄）key 變為 `repoRoot:`，冒號結尾稍奇怪 → **Mitigation**: 無影響，key 只是內部識別用，不顯示給使用者

## Migration Plan

純前端邏輯修改，無資料庫 migration。應用重啟後新的分組鍵立即生效。釘選列表可能需要使用者重新釘選（接受此 UX 代價）。
