## 1. App.tsx 狀態與 Callback 調整

- [x] 1.1 在 App.tsx 新增 `onCloseProject` handler：從 `openProjectKeys` 移除指定 key，若 `activeView === key` 則切換至 `"dashboard"`
- [x] 1.2 將 `openProjectKeys` 與 `onCloseProject` 傳入 `<Sidebar>` 元件的 props

## 2. Sidebar 元件更新

- [x] 2.1 在 `Sidebar` props 介面新增 `openProjectKeys: string[]` 與 `onCloseProject: (key: string) => void`
- [x] 2.2 在釘選專案區塊下方新增「已開啟的專案」區塊（`openProjectKeys` 非空時才渲染）
- [x] 2.3 已開啟項目展開模式：渲染 sidebar-link button（含 active 樣式），右側附加 × 關閉按鈕（`stopPropagation` 防止觸發導覽）
- [x] 2.4 已開啟項目折疊模式：渲染 icon button（首字母），hover 時右上角顯示微型 × 按鈕
- [x] 2.5 「已開啟」區塊標題加入翻譯 key（`sidebar.openProjects`）

## 3. i18n 翻譯

- [x] 3.1 在 `src/i18n/` 翻譯檔中新增 `"sidebar.openProjects"` 繁中譯文（「已開啟」）
- [x] 3.2 新增 `"sidebar.closeProject"` 按鈕 aria-label 譯文（「關閉專案」）

## 4. 移除橫排 Tab 列

- [x] 4.1 移除 `App.tsx` 中渲染 `.tab-bar` 的 JSX 區塊（Dashboard tab + 專案 tab 列表）
- [x] 4.2 移除或停用 CSS 中 `.tab-bar`、`.tab-item`、`.tab-item-project`、`.tab-close-button` 等相關規則

## 5. Sidebar 已開啟區塊樣式

- [x] 5.1 新增 `.sidebar-open-projects` 區塊樣式（與 `.sidebar-section` 一致的間距與標題字型）
- [x] 5.2 新增 `.sidebar-open-item-close` × 按鈕樣式（展開模式：右側小圓形 ×；折疊模式：hover 時右上角微型 ×）
- [x] 5.3 確認深色主題下樣式正常顯示

## 6. 驗證

- [x] 6.1 執行 `bun run build` 確認 TypeScript 無錯誤
- [x] 6.2 手動測試：開啟多個專案 → 均顯示於 sidebar 已開啟區塊 → × 可正確關閉
- [x] 6.3 手動測試：折疊 sidebar → hover 已開啟項目 → × 顯示且可關閉
- [x] 6.4 手動測試：橫排 tab 列已不存在，主內容區版面正常
