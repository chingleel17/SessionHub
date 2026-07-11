## 1. i18n 文案

- [x] 1.1 於 `src/locales/zh-TW.ts` 新增鍵：`agents.chip.loaded`（已載入）、`agents.chip.needsSync`（需同步）、`agents.chip.notInstalled`（未安裝）、`agents.chip.tooltip`（`{platform}：{state}`）、`agents.legend.loaded` / `agents.legend.needsSync` / `agents.legend.notInstalled`，以及 commands 版的清單說明文字鍵（若沿用 `agents.skills.compatNote` 則確認語意通用）。
- [x] 1.2 於 `src/locales/en-US.ts` 同步新增對應鍵，維持鍵集合一致。

## 2. 晶片呈現邏輯（AgentsConfigView）

- [x] 2.1 新增純函式：固定平台順序常數與 `chipStateFromStatus(status, enabled)`，回傳 `loaded | needsSync | notInstalled`（對應規則見 design D1）。
- [x] 2.2 在 `renderListGroup` 中，移除原本 `enabledStatuses` / `outOfSyncCount` / `badgeLabel` 的聚合徽章區塊，改以四平台順序逐一查 `resolveTargetStatuses(tab, data, entry)` 與 `prefs.enabledTargets` 建構晶片資料（label、狀態、tooltip）。
- [x] 2.3 於列內渲染四個 `agents-target-chip` 晶片（含色點與平台名、`title` tooltip），保留整列 `openPreview(entry)` 點擊，晶片不攔截點擊。

## 3. 全域清單標頭圖例與同步入口

- [x] 3.1 擴充 skills 的 `agents-skills-compat-note`（及 commands 對應列）為含三色圖例（`agents-sync-legend`）與「同步」「重新整理」按鈕的版面；同步按鈕沿用 `setSyncModalTab(tab)`，重新整理沿用 `onRefresh`。
- [x] 3.2 圖例三色點顏色與晶片色點一致，文案走 `t()`。

## 4. 樣式

- [x] 4.1 於 `src/App.css` 新增 `.agents-target-chip` 與 `--loaded` / `--needsSync` / `--notInstalled` 修飾類（固定寬度、色點、柔和底色/邊框），沿用綠/藍/灰色票。
- [x] 4.2 新增 `.agents-sync-legend` 圖例樣式，確保 sync-note 列在窄視窗下排版正常（晶片容器 `overflow:hidden`）。

## 5. 驗證

- [x] 5.1 執行 `npm run build`（或 `tsc --noEmit` + `vite build`）確認型別與建置通過。
- [x] 5.2 於實際 app 開啟 Agents → Skills / Commands，驗證四種狀態組合（已載入/需同步/未安裝）晶片與 tooltip 正確，全域群組圖例與同步入口可用。
