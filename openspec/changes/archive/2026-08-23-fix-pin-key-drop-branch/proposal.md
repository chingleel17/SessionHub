## Why

目前 `projectKey` 格式為 `normalizePath(repoRoot):branch`，把分支名納入專案身分。這個設計來自 `2026-06-15-multi-branch-pin-support`，當時假設「分支」與「worktree」是一對一關係，用 branch 作為 worktree 的區分依據。

該假設不成立，導致實際使用上的缺陷：在同一個目錄用 `git checkout` 切換分支後，projectKey 隨之改變，原本的釘選、已開啟的專案 tab 與 sub-tab 狀態全部失效，使用者必須重新釘選。

實測確認（`git rev-parse --show-toplevel`）：git worktree 會回傳 worktree 自身的目錄路徑，而非主 checkout 的路徑。因此 `repoRoot` 本身已足以區分不同 worktree，branch 在 key 中不但多餘，還造成上述缺陷。

## What Changes

- **`getProjectKey` 移除 branch 後綴**：改為僅 `normalizePath(repoRoot ?? cwd)`。同目錄切換分支不再改變專案身分。
- **舊格式 key migration**：`normalizePinnedProjectKey` 將既有 `path:branch` key 截去 branch 後綴，僅保留路徑；保留磁碟機代號的 `branchSeparatorIndex <= 1` 判斷。
- **migration 後去重**：舊資料中同一路徑的多個分支 key（如 `path:main`、`path:dev`）正規化後會塌成同一個 key，載入時以 `Set` 去重，避免 Sidebar 釘選區出現重複項目。
- **分支標籤語意明確化**：合併後同一專案可能含多個分支的 session，`getProjectBranchLabel` 由「取第一個非空 branch」（排序前呼叫，結果不定）改為「取最近更新 session 的 branch」。

## Capabilities

### New Capabilities

（無新能力，為既有功能的行為修正）

### Modified Capabilities

- `pinned-projects`：專案身分改以工作目錄為準，不再包含分支。同目錄切換分支保持釘選；不同目錄的 worktree 仍各自獨立。

## Impact

- `src/App.tsx`：`getProjectKey`、`normalizePinnedProjectKey`、`getProjectBranchLabel`、settings 載入時的 `setPinnedProjects`
- 專案分組行為連帶改變：`buildProjectGroups` 會將同目錄不同分支的 session 合併為單一專案 tab（符合預期）
- 不影響後端 Rust 程式碼（`pinned_projects` 僅儲存字串陣列）
- 不需要資料庫 migration；settings.json 的舊格式 key 於載入時正規化，下次釘選操作時自動寫回新格式
