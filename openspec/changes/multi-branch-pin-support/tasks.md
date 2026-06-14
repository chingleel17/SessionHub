## 1. 修正 settings 載入時的 key 正規化

- [x] 1.1 在 `src/App.tsx` 中找到 `setPinnedProjects((settingsQuery.data.pinnedProjects ?? []).map(normalizePath))` 這行
- [x] 1.2 將此行改為：對每個 key，若含 `:` 則只對 `:` 前的路徑部分做 `normalizePath`，branch 部分保留原樣；不含 `:` 的舊格式 key 整體做 `normalizePath`（向後相容）
- [x] 1.3 驗證：修改後的 key 格式與 `getProjectKey` 產生的格式一致

## 2. 驗證修復效果

- [ ] 2.1 啟動應用程式，釘選同一 repo 的兩個不同分支，確認兩者均出現在 Sidebar 釘選區且互不取消
- [ ] 2.2 重新啟動應用程式，確認兩個分支的釘選狀態均從 settings.json 正確還原
- [ ] 2.3 確認舊格式（僅路徑，無 `:branch` 後綴）的釘選項目在升級後仍可正常顯示（向後相容）
