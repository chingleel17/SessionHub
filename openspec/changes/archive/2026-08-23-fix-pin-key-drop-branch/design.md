## Context

`2026-06-15-multi-branch-pin-support` 將 projectKey 定為 `normalizePath(repoRoot):branch`，理由是「同一 repo 的不同分支（worktree）需各自獨立釘選」。這句描述把「分支」與「worktree」當成同義詞，是設計的錯誤前提。

實際上兩者是不同維度：

- 同一目錄可以用 `git checkout` 在多個分支間切換，目錄不變。
- 一個 repo 的多個 worktree 位於不同目錄，各自 checkout 不同分支。

把 branch 放進 key，等於用「分支」定義專案身分，於是同目錄切換分支被誤判為換了專案。

## Goals / Non-Goals

Goals：

- 同目錄同 repo 切換分支 → 視為同一專案，保持釘選。
- 不同目錄的 worktree → 仍視為不同專案，各自釘選。
- 既有釘選資料平滑升級，不需使用者手動重設。

Non-Goals：

- 不處理「同 repo 但整個專案搬移到新路徑」的識別。該情境已由既有的 `ProjectPathRemap` 機制（`2026-07-24-add-project-path-remap`）處理：`remap_path` 在 `src-tauri/src/sessions/mod.rs` 對 `repo_root` 與 `cwd` 套用重新對應，於前端取得資料前就已正規化路徑。本次不在 key 中重建這層邏輯。

## Decisions

### 決策一：以 repoRoot 作為唯一的專案身分

實測驗證：

```
$ git -C <main>     rev-parse --show-toplevel   → .../wtdemo/main
$ git -C <worktree> rev-parse --show-toplevel   → .../wtdemo/wt-b
```

`repo_root` 於 `src-tauri/src/sessions/git.rs:76` 即取自此指令，故 worktree 天然擁有不同的 `repoRoot`。結論：branch 不是 worktree 的區分依據，`normalizePath(repoRoot)` 單獨即可滿足全部三項需求。

考慮過但否決的方案：以 `repoName` 或 remote URL 作為專案身分（讓專案搬移後仍可合併）。否決原因是 worktree 與主 checkout 屬於同一個 repo、共用同一個 remote，任何基於 repo 身分的 key 都會把 worktree 錯誤合併，違反「不同目錄的 worktree 需分開」的需求。

### 決策二：migration 採「截斷後綴 + 去重」

舊 key 可能有三種形態：`path:branch`、`path:`（branch 為空字串時產生）、以及更早的純 `path`。`lastIndexOf(":")` 搭配 `<= 1` 的判斷可同時涵蓋三者，並避免誤切磁碟機代號（`d:\...`）。

截斷後多個分支 key 會塌成同一路徑，因此載入時必須去重，否則 `pinnedProjects` 出現重複值，Sidebar 釘選區會渲染重複項目。這是本次變更最容易漏掉的實際缺陷。

不另外主動改寫 settings.json：載入時正規化已足以讓行為正確，而 `togglePinProject` 是以完整陣列覆寫儲存，使用者下次釘選操作時舊格式即自然消失。

以實際使用中的 settings.json（30 筆）驗證，遷移後為 12 筆，恰為實際專案數。該筆資料涵蓋全部邊界情況：

- 同目錄多分支累積：`voxnote` 因反覆切換分支累積 6 筆（含 1 筆純路徑舊格式）
- 完全重複的 key：`fb-worktree:feature\calendar` 重複 6 次，反映既有的去重缺漏
- 斜線方向不一致：`feature\calendar` 與 `feature/calendar` 原被視為不同 key
- branch 為空：`...專案:` 形態
- 含中文與空格的路徑：`d:\ching\文件資料\...`、`d:\ching\ai tool setting`

驗證結果：磁碟機代號未被誤切、無殘留 branch 後綴、無重複項目。

### 決策三：分支標籤取最近更新的 session

合併後單一專案可能含多個分支的 session。原 `getProjectBranchLabel` 取「第一個非空 branch」，且在 `buildProjectGroups` 中於 sessions 排序前被呼叫，結果實質上不確定。改為明確依 `updatedAt` 降冪取第一個非空 branch，讓標籤代表使用者目前工作中的分支。

### 決策四：其他持久化狀態不需 migration

已清查全部 per-project 的持久化位置，確認除 `pinnedProjects` 外沒有其他資料以 `path:branch` 格式為 key：

- localStorage 的 per-project key（`explorer-view-mode`、`explorer-sort`、`AgentsConfigView` 與 `McpConfigView` 的展開狀態與分頁）一律以 `projectCwd` 為索引，非 `projectKey`。
- `agents-global-prefs` 為全域設定，無專案維度。
- `openProjectKeys` 與 `projectSubTabStates` 僅存在於 React state，未持久化；重啟後 `openProjectKeys` 由釘選項目重建。

另確認 `gitBranch` 於前端僅用於顯示（分支標籤與 session 列表標註），沒有任何啟動、resume 或終端機流程以分支推導路徑或指令，故單一專案含多分支 session 不影響既有操作。

## Risks / Trade-offs

- 使用者若原本刻意將同目錄的不同分支當成兩個獨立專案釘選，升級後會合併為一項。此為本次變更的明確意圖，且該用法在同目錄下無法真正並行工作。
- 專案 tab 的 session 列表會包含該目錄所有分支的歷史 session，數量增加。分支資訊仍可於各 session 層級呈現。
