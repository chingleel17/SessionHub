## Why

切換專案時 plan tab 狀態遺失、缺乏載入視覺回饋、plan tab 樣式破版、app icon 未更新、plan 編輯器雙欄高度不一致，這五個問題合計造成明顯的使用體驗缺陷，需要在下一個版本一併修正。

## What Changes

- **Plan Tab 跨專案保留**：將 `openPlanKeys` 與 `activeSubTab` 從 ProjectView 內部 state 提升至 App.tsx，以 `projectKey` 為 key 做 Map 儲存，讓 ProjectView `key` 重新 mount 時不再遺失已開啟的 plan tab。
- **切換專案 Loading 樣式**：在 ProjectView sessions 列表區加入骨架載入（skeleton cards）或 spinner，當 `sessionsQuery.isLoading` 為 true 時顯示，讓使用者知道資料正在載入。
- **Plan Sub-tab 樣式修正**：補上 `.sub-tab-item--closeable`、`.sub-tab-label`、`.sub-tab-close` CSS，讓可關閉的 plan tab 外觀與一般 tab 一致，× 關閉按鈕位於標籤右側、小且不搶眼。
- **應用程式 ICON 更新**：以新設計替換 `src-tauri/icons/` 下的所有 icon 檔（`.ico`、`.png` 各尺寸），風格為 SessionHub 專屬品牌（深色背景 + 白色「S」字母圖示）。
- **Plan 編輯器高度對齊**：調整 `.plan-editor-layout` CSS 為 `display: grid; grid-template-columns: 1fr 1fr; align-items: stretch`，並讓 textarea 與預覽區同高（`height: 100%`）。

## Capabilities

### New Capabilities

- `session-loading-states`: 切換專案時的骨架/spinner 載入狀態視覺元件，覆蓋 sessions 列表區與 stats 區。

### Modified Capabilities

- `tabbed-ui`: plan sub-tab 的跨專案保留邏輯（state 提升至 App.tsx）、可關閉 tab 的樣式規格。
- `plan-viewer`: plan 編輯器雙欄等高佈局規格、plan sub-tab 樣式規格。

## Impact

- `src/App.tsx`：新增 `projectSubTabState: Map<string, { activeSubTab, openPlanKeys }>` state；透過 props 傳入 ProjectView；移除 `key={activeProject.key}` 讓 ProjectView 不重新 mount（或改為傳入 initialState）。
- `src/components/ProjectView.tsx`：接收 `initialOpenPlanKeys`、`initialActiveSubTab`、`onSubTabStateChange` props，state 改為受控或以 initialState 初始化；sessions 列表加入 `isLoading` prop 驅動 skeleton。
- `src/styles/app.css`（或對應 CSS 檔）：補上 closeable sub-tab 樣式、plan-editor-layout 等高 CSS。
- `src-tauri/icons/`：替換所有 icon 檔。
- 無 Rust command 變更；無新 IPC。
