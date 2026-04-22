## Context

`SessionCard` 的「⋯」按鈕展開 `.launcher-menu`，目前以 `position: absolute` 在卡片 `position: relative` 容器內渲染。看板欄位（`.kanban-column`）的卡片堆疊在 DOM 中，後面的卡片 `z-index` 可能高於選單，導致選單被遮蓋。此外，`showLauncher` 狀態儲存在各個 `SessionCard` 內部，父層無法感知也無法統一關閉，造成多選單同時展開的問題。

## Goals / Non-Goals

**Goals:**
- 選單永遠顯示在所有卡片之上（不被其他 DOM 元素遮蓋）
- 點擊選單外部任意位置自動關閉選單
- 同一時間只能有一個選單展開（互斥）
- 選單位置跟隨觸發按鈕，不跑版

**Non-Goals:**
- 不改變選單內容或現有啟動邏輯
- 不引入第三方 floating UI 函式庫
- 不更動 Rust 後端

## Decisions

### 決策一：用 `position: fixed` + `document` click 監聽取代 absolute 定位

**選擇**：點擊「⋯」時，記錄按鈕的 `getBoundingClientRect()`，將選單以 `position: fixed; top: <y>; left: <x>; z-index: 9999` 渲染。

**理由**：`fixed` 脫離所有 stacking context，不受祖先 `z-index` 影響，是不引入 Portal 的最簡方案。

**替代方案**：React Portal（`ReactDOM.createPortal`）掛到 `document.body`，效果相同但需要額外的 ref 傳遞與 unmount 清理，YAGNI。

---

### 決策二：狀態提升至 App.tsx，用 `openLauncherSessionId` 管理互斥

**選擇**：在 `App.tsx` 新增 `openLauncherSessionId: string | null` state，透過 props 向下傳遞。`SessionCard` 呼叫 `onToggleLauncher(session.id)` 通知父層，父層決定哪個選單展開。

**理由**：符合本專案「子元件不含本地 UI 狀態之外的業務狀態」慣例；集中管理讓 click-outside 關閉邏輯只需在一個地方實作（App.tsx 或頂層 useEffect）。

**替代方案**：React Context（`LauncherMenuContext`）—適合深層嵌套，但此場景 props 鏈僅一層，無需 context。

---

### 決策三：click-outside 偵測用 `useEffect` + `document.addEventListener`

**選擇**：在 `SessionCard`（或 `App.tsx`）的 `useEffect` 中，當 `showLauncher === true` 時掛載 `mousedown` 監聽器，偵測點擊目標是否在選單 DOM 之外，若是則關閉。監聽器在選單關閉後移除。

**理由**：標準 React 做法，無副作用，與現有程式碼風格一致。

## Risks / Trade-offs

- **[Risk] 選單位置在捲動後偏移** → 選單開啟時不支援頁面捲動（`overflow: hidden` on kanban container），或偵測 scroll 事件同步關閉選單。
- **[Risk] 狀態提升增加 SessionCard props 介面複雜度** → 影響範圍僅 `App.tsx` → `KanbanView` → `SessionCard` 一條鏈，可接受。
