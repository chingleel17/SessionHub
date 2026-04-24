## Why

Project 頁的 sticky sub-tab/header 目前會透出底下的 session 卡片，切換 Sessions 與 Plans & Specs 時都會讓畫面顯得混亂。另一個問題是時間篩選只套用在 Dashboard 統計卡，底下的最近活動、專案清單與 Kanban 仍顯示超出區間的 session，而 Project 頁 session 清單也缺少時間篩選與分頁，容易讓大型專案一次渲染過多卡片。

## What Changes

- 讓 ProjectView 的 sticky sub-tab/header 在 Sessions、Plans & Specs 與 Plan 分頁上都使用不透明背景與遮罩，避免內容透出。
- 在 ProjectView 的 Sessions sub-tab 新增更新時間篩選，並為 session 卡片列表加入分頁與範圍摘要。
- 讓 Dashboard 的「本周 / 本月」切換同時套用到統計卡、專案預覽、最近活動與 Kanban 欄位內容。
- 補上篩選後無資料時的顯示規則，確保各區塊數量、日期與清單內容一致。

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `project-subtabs`: 調整 sticky 子分頁列的背景與遮罩需求，避免 Sessions/Plans & Specs 內容穿透。
- `session-list`: 為專案內 session 清單補上更新時間篩選、分頁與篩選結果摘要。
- `dashboard-stats-period`: 將時間區間從統計卡擴大為 Dashboard 資料集的共同篩選條件。
- `dashboard-kanban`: 讓 Kanban 欄位只顯示所選時間區間內的 session，並維持載入更多邏輯。
- `dashboard`: 讓專案預覽與最近活動清單跟隨 Dashboard 時間區間同步更新。

## Impact

- 前端：`src\App.tsx`、`src\components\ProjectView.tsx`、`src\components\DashboardView.tsx`、`src\components\PlansSpecsView.tsx`、`src\App.css`
- 文案：`src\locales\zh-TW.ts`、`src\locales\en-US.ts`
- OpenSpec：更新上述 capability 的 delta specs 與本 change 的 design/tasks
