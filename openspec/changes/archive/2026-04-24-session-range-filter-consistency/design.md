## Context

目前 Dashboard 以 `dashboardPeriod` 重新計算統計卡，但 `groupedProjects`、`recentSessions` 與 Kanban 使用的資料仍直接來自完整 `sessionsQuery.data`，因此畫面中的 session 日期會和統計卡不一致。ProjectView 也僅有文字、provider、tag 與空 session 過濾，缺少時間維度與分頁，當專案 session 數量增加時會一次 render 全部卡片。

## Goals / Non-Goals

**Goals:**
- 讓 Dashboard 的單一時間區間同時驅動統計卡、最近活動、專案預覽與 Kanban。
- 讓 ProjectView 的 sticky header 在所有子分頁都維持透明感但不穿透的視覺層。
- 以最小範圍加入 session 清單時間篩選與分頁，降低單次渲染數量。

**Non-Goals:**
- 不新增後端查詢或資料庫分頁 API。
- 不引入虛擬列表或第三方 UI 套件。
- 不改動 session 掃描、統計快取與 activity status 判定邏輯。

## Decisions

### 1. 在 App.tsx 建立共用的 dashboard 時間篩選資料集

將現有 `dashboardPeriod` 的起始時間計算抽成共用 helper，基於 `updatedAt` 產出 `filteredDashboardSessions`、`filteredDashboardProjects` 與 `filteredRecentSessions`。這樣統計卡與 Dashboard 各種視圖都從同一批資料衍生，避免「上面統計有篩、下面列表沒篩」的分裂狀態。

**替代方案：**分別在 DashboardView 與 KanbanBoard 各自過濾。這會讓邏輯重複，且容易讓計數與清單再度失去一致性。

### 2. ProjectView 在前端做時間篩選後分頁

Project 頁既有資料已完整載入，因此直接在 `ProjectView` 的 `useMemo` 中追加時間條件，再以固定 page size 切出目前頁資料即可。分頁狀態會在篩選條件變更時自動重設到第 1 頁，避免使用者停留在已不存在的頁碼。

**替代方案：**只做時間篩選不做分頁。這能解決部分使用性問題，但無法處理使用者對大量卡片渲染與效能疑慮的核心訴求。

### 3. Sticky 子分頁容器改為單一透明感 shell

目前 sticky 容器若直接使用整塊實底色，雖可遮住內容，卻會形成一塊厚重的背景板。改法是把 `sticky-project-header` 保持透明，另以單一 `sticky-project-shell` 提供圓角、半透明遮罩、輕量陰影與必要的 backdrop blur，並把 Sessions 與 Plans & Specs 共用同一個 shell。這樣能維持 sticky 的遮罩效果，同時移除 `toolbar-card` 與 `tag-filter-bar` 的巢狀卡片感。

**替代方案：**只對 Sessions 工具列加背景。這無法覆蓋 Plans & Specs 與子分頁列本身的透明問題，也會保留使用者反映的多層小卡片視覺。

## Risks / Trade-offs

- **[Risk]** 以 `updatedAt` 作為時間區間基準，可能讓剛建立但未更新的 session 不在本周/本月內顯示。 → **Mitigation:** 延續目前統計卡已使用的 `updatedAt` 規則，確保整體一致；不在本次變更混入新的時間語意。
- **[Risk]** 分頁新增後，搜尋或切換 filter 時頁碼可能落在空頁。 → **Mitigation:** 在所有 filter dependency 變更時重置頁碼。
- **[Risk]** Kanban Done 欄原有載入更多是依 bucket 增量，加入期間篩選後若不重置會導致顯示數量異常。 → **Mitigation:** 當 filtered project buckets 變動時重設 doneLimit。
