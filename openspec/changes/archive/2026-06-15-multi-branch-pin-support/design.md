## Context

`pinnedProjects` 儲存於 `settings.json` 中，型別為 `string[]`，每個元素為 `projectKey`（格式：`normalizePath(repoRoot):branch`）。

前端 `App.tsx` 的 `togglePinProject` 函式：

```ts
const togglePinProject = async (projectKey: string) => {
  const next = pinnedProjects.includes(projectKey)
    ? pinnedProjects.filter((k) => k !== projectKey)
    : [...pinnedProjects, projectKey];
  setPinnedProjects(next);
  const settings = buildSettingsPayload({ pinnedProjects: next });
  await invoke("save_settings", { settings });
  await queryClient.invalidateQueries({ queryKey: ["settings"] });
};
```

此處 `pinnedProjects` 來自 closure，在 React 18+ 的 `useState` hook 下，若兩次 `togglePinProject` 呼叫在同一 render cycle 內發生（例如快速連點），第二次呼叫讀到的 `pinnedProjects` 仍是 stale state，導致第二次 toggle 覆蓋第一次的結果。

## Goals / Non-Goals

**Goals:**
- 修正 `togglePinProject` 的 state 讀取 race condition，確保多次快速 toggle 行為正確
- 確保 settings 載入時，`pinnedProjects` 的每個 key 都經過 `normalizePath` 處理
- 同一 repo 的多個分支（`repoRoot:branchA`、`repoRoot:branchB`）可同時被釘選

**Non-Goals:**
- 不變更 `projectKey` 格式（仍保持 `repoRoot:branch`）
- 不做 UI 外觀修改
- 不需要後端 Rust 修改（`pinned_projects: Vec<String>` 行為正確）

## Decisions

### 決策 1：使用 functional state update

**選擇**：`setPinnedProjects(prev => ...)` 而非直接讀取 closure 中的 `pinnedProjects`

**理由**：React 的 functional updater 保證拿到最新 state，避免 stale closure 問題。即使在同一 render cycle 內連續呼叫，每次都會基於前一次 update 的結果計算。

**替代方案**：使用 `useRef` 追蹤最新值。但 functional updater 是 React 官方慣例，更簡潔且符合約定。

**修改後**：
```ts
const togglePinProject = async (projectKey: string) => {
  let next: string[] = [];
  setPinnedProjects(prev => {
    next = prev.includes(projectKey)
      ? prev.filter((k) => k !== projectKey)
      : [...prev, projectKey];
    return next;
  });
  // 需在 setPinnedProjects 之後同步讀取 next 再儲存
  // 但 async 問題：setPinnedProjects 是非同步的
  // 改為先計算 next，再同時更新 state 與儲存
};
```

實際上，正確做法是：直接從 `pinnedProjects` state 計算 `next`，但改為 functional update，並將 `invoke` 的參數直接基於計算出的 `next`：

```ts
const togglePinProject = async (projectKey: string) => {
  const next = pinnedProjects.includes(projectKey)
    ? pinnedProjects.filter((k) => k !== projectKey)
    : [...pinnedProjects, projectKey];
  setPinnedProjects(next);  // 這行本身不是問題
  // 問題在於 pinnedProjects 的值是否是最新的
};
```

根本問題在於：**同一 repo 的兩個不同 `projectKey`（`repoRoot:branchA` vs `repoRoot:branchB`）在 settings 讀取後是否都有被正確地 normalize 並還原**。

### 決策 2：確認 key 格式在 save/load 的完整性

調查發現 `getProjectKey` 已使用 `${normalizePath(raw)}:${branch}`，而 settings 載入時：

```ts
setPinnedProjects((settingsQuery.data.pinnedProjects ?? []).map(normalizePath));
```

此處直接對整個 key 字串做 `normalizePath`，但 key 格式為 `path:branch`，`normalizePath` 只應作用於路徑部分，不應作用於整個 key。

**修正**：settings 載入時，對 `pinnedProjects` 中的每個 key 做正確處理——若 key 包含 `:branch` 後綴，僅對 path 部分做 normalize；若不含（舊格式），則整體 normalize（向後相容）。

## Risks / Trade-offs

- **舊資料相容**：若使用者的 `settings.json` 中存有舊格式（無 `:branch` 後綴）的釘選項目，需正確處理避免釘選消失。
  → 緩解：normalize 時，若 key 不含 `:` 分隔符，視為純路徑，整體做 `normalizePath`。

- **branch 名稱含 `:` 字元**：branch 名稱中通常不含 `:`（Git 不允許），風險低。
  → 確認後無需額外處理。
