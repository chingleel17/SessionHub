## 1. Provider 標籤色票（fix 2）

- [x] 1.1 於 `src/styles/themes/dark.css` 與 `light.css` 新增 `--color-provider-claude-bg/text`、`--color-provider-antigravity-bg/text` 變數（claude 以 `#D97757`、antigravity 以 `#4285F4` 為基準，contrast ≥ 4.5:1）
- [x] 1.2 於 `src/App.css` 新增 `.provider-tag--claude`、`.provider-tag--antigravity` 規則（模式同既有 copilot/opencode/codex）
- [x] 1.3 補 antigravity 顯示名稱：`src/utils/providerLabel.ts` 與 `src/App.tsx` 的 `getProviderLabel` 新增 antigravity 分支，新增 locale key `settings.fields.providerAntigravity`（zh-TW、en-US），設定頁整合卡片與 toast 顯示「Antigravity」

## 2. 一般設定移除 Claude Hook 腳本路徑欄位（fix 3）

- [x] 2.1 移除 `src/components/SettingsView.tsx` 一般設定區的 hookScriptsPath label/input/「選擇資料夾」按鈕區塊，並自 `onBrowseDirectory` field union 移除 `"hookScriptsPath"`
- [x] 2.2 更新 `src/App.tsx`：`handleBrowseDirectory` 型別同步移除 `"hookScriptsPath"`，但保留 settingsForm 中 `hookScriptsPath` 值的載入與儲存透傳（既有自訂路徑不遺失）
- [x] 2.3 清理不再使用的 locale key `settings.fields.hookScriptsPath`（zh-TW、en-US）

## 3. Quota 視窗標籤本地化 helper（fix 4）

- [x] 3.1 新增共用模組 `src/utils/quotaWindowLabel.ts`：`localizedWindowLabel(provider, windowKey, rawLabel, t)` — copilot 回傳 rawLabel；其他 provider 依 windowKey 對映 i18n key（`5h`/`five_hour`→fiveHour、`7d`/`seven_day`/`weekly`/`secondary`→sevenDay、`primary`→fiveHour、sonnet/opus 變體），查無對映時回退 rawLabel
- [x] 3.2 `src/components/QuotaOverview.tsx` 改用共用 helper（移除本地 `windowLabelKey`），修正 copilot Premium/Chat 被誤映為時間視窗的既有問題
- [x] 3.3 `src/components/StatusBar.tsx` 的 `QuotaSnapshotChip` tooltip 改用共用 helper 顯示本地化視窗標籤（需將 `t` 或已翻譯標籤傳入 chip）

## 4. 狀態列 quota chip 精簡顯示（fix 1）

- [x] 4.1 新增小型 SVG 圓環元件（約 12–14px，`stroke-dasharray` 依 pct 繪製，stroke 色沿用 `quotaBarColor()` 門檻）
- [x] 4.2 `QuotaSnapshotChip` 移除水平進度條，改為「縮寫 + 圓環 + 變色百分比數字」；百分比不可得時僅顯示縮寫
- [x] 4.3 `QuotaChip`（本地彙總）比照調整：有 limit 時以圓環 + 百分比取代進度條，cost 文字保留
- [x] 4.4 更新 `src/App.css` 狀態列 quota chip 樣式（移除 `global-status-bar-quota-bar/fill` 相關寬度佔用、新增圓環對齊樣式），並遵循 sessionhub-minimal-ui 設計 token

## 5. 驗證

- [x] 5.1 `npm run build`（tsc + vite）通過，無型別錯誤與未使用變數
- [x] 5.2 啟動 app 驗證：視窗縮小 + nav 展開時狀態列四個 provider chip 不擁擠；tooltip 顯示「5 小時」「1 週」中文標籤；設定頁 claude/antigravity 標籤有品牌色；一般設定無 Claude Hook 腳本路徑欄位且儲存設定後 settings.json 的 `hook_scripts_path` 原值保留
