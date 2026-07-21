## 1. 前置分析

- [ ] 1.1 讀取 `src/App.tsx` 第 61-65、67-150、152-210、2756-2768 行，記錄 `EMBEDDED_VIEW` 判斷、兩個 embedded 元件、`RoutedApp` 的完整現況
- [ ] 1.2 讀取設定表單相關程式碼（522、639、819、879、891、903、998、1054 行一帶），列出每個 mutation 的 `mutationFn`、`onSuccess`、`onError`、`onSettled` 內容作為核對基準

## 2. 抽出 embedded 元件

- [ ] 2.1 新增 `src/app/EmbeddedQuotaOverlayApp.tsx`，搬入 `EmbeddedQuotaOverlayApp` 完整實作
- [ ] 2.2 新增 `src/app/EmbeddedTrayPanelApp.tsx`，搬入 `EmbeddedTrayPanelApp` 完整實作
- [ ] 2.3 新增 `src/app/RoutedApp.tsx`，搬入 `EMBEDDED_VIEW` 常數讀取、`document.documentElement.classList.add` 副作用、`RoutedApp` 分派邏輯（依 design.md D1，保持模組載入時機不變）
- [ ] 2.4 更新 `src/main.tsx` 的 import 路徑指向新的 `RoutedApp`
- [ ] 2.5 從 `src/App.tsx` 移除已搬移的元件與 `EMBEDDED_VIEW` 相關程式碼，`App.tsx` 改為 export 主 `App()` 元件

## 3. 抽出設定表單 hook

- [ ] 3.1 新增 `src/hooks/useAppSettingsForm.ts`，簽章依 design.md D2
- [ ] 3.2 搬入 `settingsForm` state、`settingsQuery.data` 映射 `useEffect`（複用 `cleanup-deps-and-settings-defaults` change 產出的 `mergeAppSettings()`）、`buildSettingsPayload`、`persistSettingsSilently`
- [ ] 3.3 搬入 `settingsMutation`、`detectTerminalMutation`、`detectVscodeMutation`、`providerIntegrationMutation`，透過 `onSettingsSaved` callback 保留 `restart_session_watcher` 等外部副作用的觸發點（依 design.md D2，不內化進 hook）
- [ ] 3.4 改寫 `App()`，改為呼叫 `useAppSettingsForm(...)` 並解構所需值/函式，移除原本重複程式碼

## 4. 核對與驗證

- [ ] 4.1 逐一比對步驟 1.2 記錄的每個 mutation callback，確認搬移後邏輯完全一致（toast 文案、invalidateQueries 的 queryKey 等）
- [ ] 4.2 執行 `tsc --noEmit` 確認型別檢查通過
- [ ] 4.3 執行 `vite build` 確認建置成功
- [ ] 4.4 手動測試：開啟 quota overlay 獨立視窗、tray mini panel，確認樣式與互動與重構前一致
- [ ] 4.5 手動測試：開啟設定頁修改各欄位、儲存設定、觸發 provider integration 安裝/更新、terminal/VSCode 自動偵測，確認行為與重構前一致
