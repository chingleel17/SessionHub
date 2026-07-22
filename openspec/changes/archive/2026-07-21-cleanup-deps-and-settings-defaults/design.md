## Context

`AppSettings`（`src/types/index.ts:36`）有 3 個必填欄位（`copilotRoot`、`opencodeRoot`、`codexRoot`、`showArchived`、`enabledProviders`）與約 25 個選填欄位。前端目前有三處各自手寫這些選填欄位的預設值：

1. `src/App.tsx:522` — `settingsForm` 的 `useState` 初始值（必填欄位給空字串／預設陣列，選填欄位給具體預設值）
2. `src/App.tsx:639` — `useEffect` 內，把 `settingsQuery.data`（後端回傳、欄位可能是 `undefined`）映射回 `settingsForm` 時，逐欄 `settingsQuery.data.xxx ?? 預設值`
3. `src/App.tsx:998` — `buildSettingsPayload(overrides)`，逐欄 `overrides.xxx ?? settingsForm.xxx ?? 預設值` 組出要送給後端的完整 payload

後端 `src-tauri/src/lib.rs:336` 在背景 quota 執行緒中，若 `load_settings_internal()` 失敗，會手寫重建一份完整 `AppSettings` struct 當 fallback；但 `src-tauri/src/settings.rs:224` 已存在 `AppSettings::default()`，理論上可直接複用。

三處前端定義目前**必須人工保持一致**（例如 `analyticsRefreshInterval` 預設 30、`trayQuotaMode` 預設 `"icon_only"` 等），新增欄位時很容易漏改其中一處。

## Goals / Non-Goals

**Goals:**
- 建立唯一一份「選填欄位預設值」來源（`DEFAULT_APP_SETTINGS`），供三處前端邏輯共用
- 後端 fallback 改用既有 `AppSettings::default()`，移除重複手寫的 struct 建構
- 移除三個未使用的 npm 套件
- 全程不改變任何欄位的實際預設值（行為不變）

**Non-Goals:**
- 不重新設計 `AppSettings` 的欄位結構或型別
- 不處理 `App.tsx` 其餘壞味道（事件監聽重複、embedded app 抽離、檔案拆分等），留待後續 change
- 不改變 `save_settings` / `get_settings` 的 Tauri command 簽章
- 不新增設定欄位或改變既有欄位語意

## Decisions

### D1：`DEFAULT_APP_SETTINGS` 只涵蓋「選填欄位」，型別為 `Required<Omit<AppSettings, 必填欄位>>`
必填欄位（`copilotRoot`、`opencodeRoot`、`codexRoot`、`showArchived`、`enabledProviders`）沒有語意上「固定不變」的預設值——`copilotRoot` 等是安裝路徑，理論上一定由後端 `get_settings` 回傳實際值；空字串只是 UI 尚未載入時的暫時 placeholder，不屬於本次要收斂的「重複定義」問題。因此 `DEFAULT_APP_SETTINGS` 只收斂選填欄位，必填欄位的初始 placeholder 留在 `settingsForm` 的 `useState` 初始值原地，不納入常數。

替代方案（放棄）：把全部欄位都塞進 `DEFAULT_APP_SETTINGS`。放棄原因：必填欄位語意是「尚未載入」而非「預設值」，混在一起會讓常數名稱與用途不清楚。

### D2：新增 `mergeAppSettings(partial, overrides?)` 輔助函式取代逐欄 `?? 預設值`
```ts
// src/utils/appSettingsDefaults.ts（新檔案，遵循現有 utils/ 慣例）
export const DEFAULT_APP_SETTINGS: Required<Omit<AppSettings,
  "copilotRoot" | "opencodeRoot" | "codexRoot" | "showArchived" | "enabledProviders" | "providerIntegrations"
>> = { terminalPath: null, externalEditorPath: null, pinnedProjects: [], /* ...其餘選填欄位 */ };

export function mergeAppSettings(source: Partial<AppSettings> | undefined): typeof DEFAULT_APP_SETTINGS {
  // 對每個 DEFAULT_APP_SETTINGS 的 key 做 source?.[key] ?? DEFAULT_APP_SETTINGS[key]
}
```
`settingsQuery.data` 映射處與 `buildSettingsPayload` 都改成先展開 `mergeAppSettings(...)` 再疊加必填欄位／override，避免重複的 `?? ` 鏈。

替代方案（放棄）：只抽常數不抽合併函式，三處各自寫展開語法。放棄原因：`buildSettingsPayload` 的 override 優先序（override → settingsForm → 預設值）與 `useEffect` 映射的優先序（後端資料 → 預設值）不同，若沒有函式包裝容易再次各寫各的、產生新的不一致。

### D3：後端 fallback 直接呼叫 `AppSettings::default()`，不手寫 struct
`lib.rs:336` 的 `unwrap_or_else` closure 直接改為 `AppSettings::default().unwrap_or_else(|_| /* 極端 fallback，維持現況邏輯 */)`，或視 `default()` 簽章調整錯誤處理方式。不重新定義任何欄位值。

### D4：三個未使用套件直接以套件管理 CLI 移除
依專案慣例（CLAUDE.md）不得手改 `package.json`；改用 `bun remove react-router-dom @codemirror/lang-markdown @codemirror/view`（若相容性有疑慮則改 `npm uninstall`）。

## Risks / Trade-offs

- **[風險] 手動抽常數時遺漏某個欄位的既有預設值，導致行為改變** → 緩解：以 `git diff` 逐欄比對三處舊程式碼與新 `DEFAULT_APP_SETTINGS`，且 tasks.md 會要求對照原始碼截圖式核對（非猜測）
- **[風險] 移除套件後若有動態 import 或型別殘留未被 grep 掃到** → 緩解：任務清單包含移除後跑 `tsc --noEmit` 與 `vite build` 全量檢查
- **[風險] 後端 `AppSettings::default()` 回傳 `Result`，若簽章與原本手寫 fallback 的容錯行為不同（例如原本永遠不失敗，`default()` 可能因路徑偵測失敗而 Err）** → 緩解：實作前先讀 `settings.rs:224` 確認簽章與可能的失敗情境，設計對應的雙層 fallback，不可讓背景執行緒 panic

## Migration Plan

無資料遷移。純程式碼重構，透過一般 PR/commit 流程套用；若 build 或型別檢查失敗則不合併。無需 rollback 策略之外的「revert commit」。
