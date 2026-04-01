## 1. Plan Sub-tab 狀態提升（跨專案保留）

- [x] 1.1 在 `App.tsx` 新增 `projectSubTabStates: Map<string, { openPlanKeys: string[]; activeSubTab: string }>` state
- [x] 1.2 新增 helper `getProjectSubTabState(projectKey)` 回傳該專案的 sub-tab state（預設 `{ openPlanKeys: [], activeSubTab: "sessions" }`）
- [x] 1.3 新增 `handleSubTabStateChange(projectKey, state)` callback，更新 Map 並呼叫 `setProjectSubTabStates`
- [x] 1.4 更新 `ProjectView` Props 介面：新增 `openPlanKeys: string[]`、`activeSubTab: string`、`onSubTabStateChange: (state) => void`
- [x] 1.5 將 `ProjectView` 內部 `openPlanKeys`、`activeSubTab` 改為由 props 控制（移除這兩個 `useState`，改用 props 值 + props callback）
- [x] 1.6 在 `App.tsx` 的 `<ProjectView>` 傳入 `openPlanKeys`、`activeSubTab`、`onSubTabStateChange`
- [x] 1.7 移除 `<ProjectView key={activeProject.key}>`（或保留 key 只含 project path，確認不影響其他 state）

## 2. Sessions 載入骨架樣式

- [x] 2.1 在 `ProjectView` Props 新增 `sessionsLoading: boolean`
- [x] 2.2 在 `App.tsx` 傳入 `sessionsLoading={sessionsQuery.isLoading}` 給 `<ProjectView>`
- [x] 2.3 在 `ProjectView` sessions 列表區：當 `sessionsLoading` 為 true 時，渲染 3 個 `<div className="skeleton-card" />` 取代 session 卡片
- [x] 2.4 在 `app.css`（或對應 CSS 檔）新增 `.skeleton-card` 樣式：固定高度（80px）、圓角、帶 shimmer 動畫（`@keyframes shimmer`）

## 3. Plan Sub-tab 可關閉標籤 CSS

- [x] 3.1 在 `app.css` 補上 `.sub-tab-item--closeable` 樣式：`display: inline-flex; align-items: center; padding: 0`（移除 button padding，改由子元素承擔）
- [x] 3.2 補上 `.sub-tab-label` 樣式：繼承文字顏色、`padding: 6px 8px 6px 12px`、`background: none; border: none; cursor: pointer`
- [x] 3.3 補上 `.sub-tab-close` 樣式：`width: 18px; height: 18px; opacity: 0.45; background: none; border: none; cursor: pointer; border-radius: 3px; font-size: 14px; line-height: 1`；hover 時 `opacity: 1; background: var(--color-surface-raised)`

## 4. App Icon 更新

- [x] 4.1 設計 SessionHub icon SVG（深藍 `#1a2744` 背景圓角矩形，白色 Consolas/等寬字型「S」置中）
- [x] 4.2 將 SVG 轉存為 `icon.png`（1024×1024），放至 `src-tauri/icons/icon.png`
- [x] 4.3 執行 `bunx tauri icon src-tauri/icons/icon.png` 自動產生所有尺寸的 png 與 ico 覆蓋 `src-tauri/icons/`

## 5. Plan 編輯器雙欄等高

- [x] 5.1 在 `app.css` 更新 `.plan-editor-layout`：`display: grid; grid-template-columns: 1fr 1fr; gap: 16px; align-items: stretch; min-height: 400px`
- [x] 5.2 更新 `.plan-textarea`：`height: 100%; resize: none; box-sizing: border-box; min-height: 0`
- [x] 5.3 更新 `.plan-preview`（或 `.plan-preview-markdown`）：`overflow-y: auto; height: 100%`

## 6. 驗證

- [x] 6.1 執行 `bun run build`，確認 TypeScript 型別檢查無錯誤
- [ ] 6.2 手動驗證：開啟專案 A 的 plan tab → 切到專案 B → 切回專案 A，plan tab 仍存在
- [ ] 6.3 手動驗證：重新整理 sessions 時（或首次開啟專案）可看到骨架動畫
- [ ] 6.4 手動驗證：plan sub-tab 外觀正確，× 按鈕不破版
- [ ] 6.5 手動驗證：plan 編輯器兩欄等高，textarea 撐滿左欄
- [ ] 6.6 手動驗證（build 後）：taskbar / 開始選單顯示新版 SessionHub icon
