## 1. 型別與設定擴充

- [x] 1.1 在 `src/types/index.ts` 的 `AppSettings` 型別加入 `pinnedProjects?: string[]` 欄位
- [x] 1.2 在 `src-tauri/src/lib.rs` 的 `AppSettings` struct 加入 `#[serde(default)] pinned_projects: Vec<String>` 欄位

## 2. 後端：新增 delete_empty_sessions command

- [x] 2.1 在 `src-tauri/src/lib.rs` 新增 `delete_empty_sessions_internal(copilot_root: &str) -> Result<usize, String>` 輔助函式，掃描 session-state 目錄並刪除 summaryCount = 0 的 session 資料夾
- [x] 2.2 新增 `#[tauri::command] fn delete_empty_sessions(...)` wrapper，委派給 `_internal` 並回傳 `Result<usize, String>`
- [x] 2.3 在 `invoke_handler![]` 中登記 `delete_empty_sessions`
- [x] 2.4 撰寫 Rust 單元測試驗證 `delete_empty_sessions_internal` 邏輯（含無空 session、有空 session、刪除失敗情境）

## 3. 前端：Icon 元件

- [x] 3.1 建立 `src/components/Icons.tsx`，實作 SVG icon 元件：`TerminalIcon`、`CopyIcon`、`ArchiveIcon`、`UnarchiveIcon`、`DeleteIcon`、`PinIcon`、`UnpinIcon`
- [x] 3.2 在 `src/styles/` 新增或更新 icon button 相關 CSS class（含 tooltip 顯示、hover 效果）

## 4. 前端：Session Action Buttons 改為 Icon

- [x] 4.1 找到 session 卡片元件（`SessionCard` 或相關元件），將「開啟終端」、「複製指令」、「封存」、「取消封存」、「刪除」按鈕改為使用 Icons.tsx 的 icon button
- [x] 4.2 為每個 icon button 加上 `aria-label` 與 `title`（tooltip）屬性，文字從 i18n `t()` 取得
- [x] 4.3 更新 `src/locales/zh-TW.json`（或對應翻譯檔）加入 icon button 的 tooltip 翻譯 key（如已存在則確認對應）

## 5. 前端：空 Session 篩選 UI

- [x] 5.1 在 `App.tsx` 新增 `hideEmptySessions` state（boolean，預設 false）
- [x] 5.2 在 session 過濾邏輯中加入：當 `hideEmptySessions` 為 true 時，排除 `summaryCount === 0` 的 session
- [x] 5.3 在篩選列元件（DashboardView 或 ProjectView）加入「隱藏空 session」切換開關，並顯示目前隱藏的空 session 數量提示
- [x] 5.4 更新翻譯 key，加入「隱藏空 session」、「已隱藏 N 個空 session」等文字

## 6. 前端：批次刪除空 Session

- [x] 6.1 在 `App.tsx` 新增 `deleteEmptySessionsMutation`（呼叫 `delete_empty_sessions` command）
- [x] 6.2 在篩選列加入「刪除空 session」按鈕，當無空 session 時設為 disabled
- [x] 6.3 點擊「刪除空 session」時顯示確認對話框（使用 `setConfirmDialog`），告知預計刪除數量
- [x] 6.4 確認後執行 mutation，成功後顯示 Toast「已刪除 N 個空 session」並 invalidate session query
- [x] 6.5 更新翻譯 key，加入刪除相關文字

## 7. 前端：釘選專案功能

- [x] 7.1 在 `App.tsx` 從 settings query 取得 `pinnedProjects`，新增 `togglePinProject(key: string)` 函式（toggle 後呼叫 `save_settings`）
- [x] 7.2 在 project group header 元件加入釘選 icon button（使用 `PinIcon` / `UnpinIcon`），點擊呼叫 `togglePinProject`
- [x] 7.3 更新翻譯 key，加入「釘選專案」、「取消釘選」等文字

## 8. 前端：釘選專案 Tab 置頂

- [x] 8.1 修改 tab 列排序邏輯：釘選的 project group 排列於 Dashboard tab 之後、其他 tab 之前
- [x] 8.2 釘選的 project tab 以 CSS class 標示（加上 PinIcon 小圖示）

## 9. 前端：Sidebar 釘選專案快速入口

- [x] 9.1 在 `Sidebar.tsx` 加入「釘選專案」區段，接收 `pinnedProjects` 與 `projectGroups` props
- [x] 9.2 僅顯示實際存在對應 project group 的釘選項目（過濾已不存在的 key）
- [x] 9.3 點擊 sidebar 釘選項目時切換 `activeView`，效果等同點擊 tab
- [x] 9.4 當無釘選專案時隱藏此區段
- [x] 9.5 更新翻譯 key，加入「釘選專案」區段標題

## 10. 測試與品質確認

- [x] 10.1 執行 `cd src-tauri && cargo test` 確認 Rust 測試全部通過
- [x] 10.2 執行 `bun run build` 確認前端型別無錯誤
- [ ] 10.3 手動驗證：空 session 篩選、批次刪除、icon tooltip、釘選專案 sidebar 與 tab 行為
