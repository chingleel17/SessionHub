## 1. 狀態提升：App.tsx 管理選單互斥

- [x] 1.1 在 `App.tsx` 新增 `openLauncherSessionId` state（`string | null`），初始值為 `null`
- [x] 1.2 新增 `handleToggleLauncher(sessionId: string)` 函式：若傳入 id 等於目前 `openLauncherSessionId` 則設為 null（toggle），否則設為該 id
- [x] 1.3 將 `openLauncherSessionId` 與 `onToggleLauncher` 透過 props 傳遞到 `KanbanView`（或直接到 `SessionCard`）

## 2. SessionCard 改用外部受控狀態

- [x] 2.1 在 `SessionCard` props 新增 `isLauncherOpen: boolean` 與 `onToggleLauncher: () => void`，移除內部 `showLauncher` useState
- [x] 2.2 將 `onClick={() => setShowLauncher(v => !v)}` 改為 `onClick={() => onToggleLauncher()}`
- [x] 2.3 將 `{showLauncher ? ...}` 改為 `{isLauncherOpen ? ...}`

## 3. 選單改為 fixed 定位

- [x] 3.1 在 `SessionCard` 的「⋯」按鈕上綁定 `ref`（`useRef<HTMLButtonElement>`），用於取得按鈕位置
- [x] 3.2 選單開啟時以 `getBoundingClientRect()` 計算 `top` / `left` 座標，存為 component state
- [x] 3.3 將 `.launcher-menu` 改以 `ReactDOM.createPortal` 渲染到 `document.body`，套用 inline style `position: fixed; top: <y>px; left: <x>px; z-index: 9999`
- [x] 3.4 在 CSS 移除 `.launcher-dropdown` 的 `position: relative`（如有），並確認 `.launcher-menu` 樣式不依賴父容器定位

## 4. Click-outside 自動關閉

- [x] 4.1 在 `App.tsx` 新增 `useEffect`：當 `openLauncherSessionId !== null` 時，綁定 `document.addEventListener("mousedown", handleClickOutside)`；清理時移除
- [x] 4.2 實作 `handleClickOutside`：檢查 `event.target` 是否在選單 DOM 之內（透過傳遞選單 ref 或 data attribute），若不在則呼叫 `setOpenLauncherSessionId(null)`
- [x] 4.3 同時監聽 `scroll` 事件（在 `.kanban-board` 容器），捲動時關閉選單，避免位置偏移

## 5. 驗證

- [x] 5.1 執行 `bun run build` 確認 TypeScript 型別無誤
- [x] 5.2 手動測試：開啟選單 → 點擊外部 → 確認關閉
- [x] 5.3 手動測試：同時點擊兩個不同 SessionCard 的「⋯」→ 確認只有一個選單展開
- [x] 5.4 手動測試：選單不被相鄰卡片遮蓋（在卡片密集的欄位驗證）
