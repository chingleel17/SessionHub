## 1. Provider 縮寫代碼統一

- [x] 1.1 於 `src/utils/providerLabel.ts` 新增 `PROVIDER_ABBR` 對照表與 `getProviderAbbr()` 函式（含未知 provider 的後備邏輯）
- [x] 1.2 `src/components/ProviderIcon.tsx` 移除本地 `initialsByProvider`，改用 `getProviderAbbr()`（修正 claude 誤植為 `CL`）
- [x] 1.3 `src/components/DashboardView.tsx` 移除 provider 標籤的 ternary 判斷，改用 `getProviderAbbr()` 並補上 `title`
- [x] 1.4 `src/components/QuotaOverlay.tsx` 移除本地 `PROVIDER_ABBR` 常數，改用共用函式
- [x] 1.5 `src/components/StatusBar.tsx` 移除本地 `PROVIDER_ABBR` 常數（兩處使用），改用共用函式

## 2. Dashboard Kanban ProjectCard 標頭版面

- [x] 2.1 `src/App.css` 新增 `.kanban-project-branch` class（可壓縮、省略號截斷），與既有 `.kanban-project-time`（日期用）分離
- [x] 2.2 `.kanban-project-name` 設定 `min-width: 5ch` 下限，避免被擠壓至不可見
- [x] 2.3 新增 `PROVIDERS_PER_PROJECT_CARD` 常數（上限 2），provider 標籤超過上限以 `+N` 摘要呈現，並在 `title` 列出完整清單
- [x] 2.4 `.kanban-project-providers` 維持 `flex-shrink: 0`，新增 `.kanban-project-providers-more` 樣式
- [x] 2.5 專案名稱與分支名稱補上 `title` 屬性

## 3. 驗證

- [x] 3.1 `tsc --noEmit` 型別檢查通過
- [x] 3.2 `npm run lint` 於變更檔案無新增警告
- [x] 3.3 以 Playwright 於 `npm run dev` 實際頁面注入多組情境（1–5 個 provider、超長分支名、超長專案名，卡片寬度 220–300px）驗證：無溢出、無標籤裁切、專案名稱恆可見
- [x] 3.4 grep 確認無殘留的舊縮寫常數定義（`PROVIDER_ABBR` 僅存在於 `providerLabel.ts`）與 `provider-tag--antigravity` CSS 規則存在
