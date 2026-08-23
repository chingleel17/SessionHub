## 1. 移除 projectKey 的 branch 後綴

- [x] 1.1 `src/App.tsx` 的 `getProjectKey` 改為僅回傳 `normalizePath(repoRoot ?? cwd)`，移除 `:${branch}` 後綴
- [x] 1.2 補上註解說明 worktree 已由 `rev-parse --show-toplevel` 天然區分，branch 非必要

## 2. 舊格式 key migration

- [x] 2.1 `normalizePinnedProjectKey` 改為截去 `:branch` 後綴，僅回傳 `normalizePath` 後的路徑部分
- [x] 2.2 保留 `branchSeparatorIndex <= 1` 判斷，避免誤切磁碟機代號（`d:\...`）
- [x] 2.3 settings 載入時以 `Set` 對正規化後的 key 去重，避免同路徑多分支塌成重複項目

## 3. 分支標籤語意明確化

- [x] 3.1 `getProjectBranchLabel` 改為依 `updatedAt` 降冪取最近更新 session 的分支

## 4. 驗證

- [x] 4.1 實測 `git rev-parse --show-toplevel` 於 worktree 回傳 worktree 自身路徑，確認設計前提
- [x] 4.2 `tsc --noEmit` 型別檢查通過
- [x] 4.3 `oxlint --react-plugin src` 無新增警告
- [x] 4.4 啟動應用程式（`bun run tauri:dev`），確認編譯與啟動正常
- [x] 4.5 於同一目錄切換分支，確認釘選與專案 tab 保持不變（需手動操作 git checkout）
- [x] 4.6 確認不同目錄的 worktree 仍為獨立專案，可各自釘選（需手動操作）
- [x] 4.7 以實際 settings.json（30 筆，含 `path:branch`、`path:`、純 `path` 三種形態）驗證 migration：正規化去重後為 12 筆，磁碟機代號未被誤切、無殘留 branch 後綴、無重複項目
