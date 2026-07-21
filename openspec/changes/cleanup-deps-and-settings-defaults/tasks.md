## 1. 移除未使用套件

- [ ] 1.1 再次確認 `react-router-dom`、`@codemirror/lang-markdown`、`@codemirror/view` 在 `src/` 全域無任何引用（`grep -rn "react-router\|codemirror" src/`）
- [ ] 1.2 使用 `bun remove react-router-dom @codemirror/lang-markdown @codemirror/view` 移除套件（依 CLAUDE.md 規範不得手改 `package.json`）
- [ ] 1.3 確認 `package.json` / `bun.lock`（或對應 lockfile）已更新且無殘留引用

## 2. 前端：建立 DEFAULT_APP_SETTINGS 與合併函式

- [ ] 2.1 讀取 `src/App.tsx` 目前 `settingsForm` 初始值（約第 522 行起）、`useEffect` 映射（約第 639 行起）、`buildSettingsPayload`（約第 998 行起），逐欄記錄目前實際預設值作為核對基準
- [ ] 2.2 新增 `src/utils/appSettingsDefaults.ts`，定義 `DEFAULT_APP_SETTINGS`（僅涵蓋選填欄位，型別依 design.md D1）與 `mergeAppSettings()` 輔助函式（依 design.md D2）
- [ ] 2.3 改寫 `settingsForm` 的 `useState` 初始值：必填欄位維持原地 placeholder，選填欄位改為展開 `DEFAULT_APP_SETTINGS`
- [ ] 2.4 改寫 `useEffect` 中 `settingsQuery.data` → `settingsForm` 的映射邏輯，改用 `mergeAppSettings(settingsQuery.data)` 取代逐欄 `?? 預設值`
- [ ] 2.5 改寫 `buildSettingsPayload()`，改用 `mergeAppSettings()` 取代逐欄 `?? 預設值`，保留其 override 優先序（overrides → settingsForm → 預設值）
- [ ] 2.6 逐欄比對步驟 2.1 記錄的基準值與重構後行為，確認無任何欄位預設值被意外改變

## 3. 後端：fallback 改用 AppSettings::default()

- [ ] 3.1 讀取 `src-tauri/src/settings.rs:224` 的 `AppSettings::default()` 簽章與可能的失敗情境
- [ ] 3.2 改寫 `src-tauri/src/lib.rs:336` 背景執行緒中 settings 載入失敗的 fallback，改為呼叫 `AppSettings::default()`，並依其簽章妥善處理 `Result`（不可 panic、不可讓背景執行緒中止 quota 監控迴圈）
- [ ] 3.3 移除原本手寫重建整個 `AppSettings` struct 的程式碼
- [ ] 3.4 確認此路徑欄位值與 `AppSettings::default()` 一致（無行為變更）

## 4. 驗證

- [ ] 4.1 執行 `tsc --noEmit`（或專案既有 typecheck script）確認前端型別檢查通過
- [ ] 4.2 執行 `vite build`（或專案既有 build script）確認前端建置成功
- [ ] 4.3 執行 `cargo check`（於 `src-tauri/`）確認後端編譯成功
- [ ] 4.4 執行既有前後端測試套件（若有）確認無回歸
- [ ] 4.5 手動啟動應用程式，開啟設定頁確認各欄位顯示值與變更前一致，儲存設定後重啟應用程式確認持久化行為正常
