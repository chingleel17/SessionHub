## Why

`src/App.tsx`（2768 行）除了主應用邏輯外，還內嵌了兩個與主視窗完全獨立的 React 元件 `EmbeddedQuotaOverlayApp`（第 67～150 行）與 `EmbeddedTrayPanelApp`（第 152～210 行）——它們是 quota overlay / tray mini panel 這兩個獨立 webview 的進入點，透過 URL query `?view=` 分流渲染，與主應用的 state、hooks 完全不共用。此外設定表單相關邏輯（`settingsForm` state、`buildSettingsPayload`、`persistSettingsSilently`、`settingsMutation` 等，散落於 App.tsx 第 522、639、998、1054 行一帶）雖然與主應用相關，但邏輯內聚、可獨立測試，目前混在 `App.tsx` 主體中增加了檔案長度與職責混雜度。

## What Changes

- 將 `EmbeddedQuotaOverlayApp`、`EmbeddedTrayPanelApp` 與路由分派元件 `RoutedApp` 搬到獨立檔案（如 `src/app/EmbeddedQuotaOverlayApp.tsx`、`src/app/EmbeddedTrayPanelApp.tsx`），`App.tsx` 只保留主應用 `App()` 元件本體，並在 `main.tsx`（或新的 entry 檔）中改為引用新位置的 `RoutedApp`
- 將設定表單相關邏輯（`settingsForm` state、`settingsQuery` 映射、`buildSettingsPayload`、`persistSettingsSilently`、`settingsMutation`、`detectTerminalMutation`、`detectVscodeMutation`、`providerIntegrationMutation`）抽為獨立 custom hook（如 `src/hooks/useAppSettingsForm.ts`），供 `App.tsx` 呼叫
- 依賴 change `cleanup-deps-and-settings-defaults` 已完成的 `DEFAULT_APP_SETTINGS` / `mergeAppSettings()`，新 hook 直接複用，不重新定義預設值
- 不改變任何元件的渲染結果、UI 行為、設定儲存/讀取的實際邏輯

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

（無 — 純內部程式碼搬移，不改變 `statusbar-quota-popup`、`tray-quota-widget`、`app-settings`、`provider-integration` 等既有 capability 的行為需求。）

## Impact

- 受影響程式碼：
  - `src/App.tsx`（大幅減少行數，移除 embedded app 元件與設定表單邏輯）
  - 新增 `src/app/EmbeddedQuotaOverlayApp.tsx`、`src/app/EmbeddedTrayPanelApp.tsx`（或依實作合併為單一 `src/app/EmbeddedApps.tsx`）
  - 新增 `src/hooks/useAppSettingsForm.ts`
  - `src/main.tsx`（若路由分派元件搬移，需調整 import 路徑）
- 不影響任何 Tauri command 簽章或後端邏輯
- 建議在 `cleanup-deps-and-settings-defaults` 與 `extract-app-tsx-event-hooks` 之後套用，以減少三份重構之間的 diff 衝突，但無強制依賴順序
