## Why

同一個 repo 可能有多個分支或 worktree 同時工作，但目前的釘選功能以 `projectKey`（`repoRoot:branch`）作為識別碼，儲存至 `pinnedProjects: string[]`。問題在於：當使用者對第二個分支進行釘選操作時，`togglePinProject` 讀到的 `pinnedProjects` state 可能尚未包含第一個分支的釘選 key，導致後訂選覆蓋前一個，最終只保留最後一次的釘選。

## What Changes

- **修正 `togglePinProject` 的 race condition**：改為 functional state update，確保每次 toggle 基於最新 state，而非 closure 捕獲的過時值。
- **確保 `normalizePath` 一致應用**：`pinnedProjects` 的儲存與比對均透過統一的 normalize 函式，避免路徑格式差異造成 key 不匹配。
- **釘選 key 格式驗證**：確認 `getProjectKey` 產生的 `repoRoot:branch` 格式在 save/load settings 時完整保留，不被截斷或誤轉義。

## Capabilities

### New Capabilities

（無新能力，為既有功能的 bug fix）

### Modified Capabilities

- `pinned-projects`：修正同一 repo 多分支時，後釘選覆蓋前釘選的問題；釘選狀態需正確支援多個 `repoRoot:branch` key 同時存在。

## Impact

- `src/App.tsx`：`togglePinProject` 函式改用 functional state update
- `src/App.tsx`：settings 載入時確保 `pinnedProjects` 的每個 key 做 `normalizePath`
- 不影響後端 Rust 程式碼（`pinned_projects` 僅儲存字串陣列）
- 不需要資料庫 migration
