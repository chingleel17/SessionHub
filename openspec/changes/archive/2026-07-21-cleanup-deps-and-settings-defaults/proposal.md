## Why

前端相依套件中有三個已完全未被使用（`react-router-dom`、`@codemirror/lang-markdown`、`@codemirror/view`），增加不必要的安裝與 bundle 體積。同時，`AppSettings` 的預設值散落在四處手寫（`src/App.tsx` 三處、`src-tauri/src/lib.rs` 一處），每新增一個設定欄位都要同步改四個地方，容易遺漏造成前後端預設值不一致。這是一次純重構，不涉及任何使用者可見行為變更。

## What Changes

- 移除 `package.json` 中未使用的相依套件：`react-router-dom`、`@codemirror/lang-markdown`、`@codemirror/view`
- 前端新增單一 `DEFAULT_APP_SETTINGS` 常數（與對應合併輔助函式），並讓以下三處改為共用同一份預設值來源：
  - `settingsForm` 初始 state
  - `settingsQuery.data` 回寫 `settingsForm` 的 `useEffect`
  - `buildSettingsPayload()`
- 後端 `src-tauri/src/lib.rs` 背景執行緒中 settings 載入失敗的 fallback，改為直接呼叫既有的 `AppSettings::default()`（`src-tauri/src/settings.rs`），移除手寫重複建構整個 struct 的程式碼
- 不改變任何欄位的實際預設值，純粹消除重複定義來源

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

（無 — 本次變更為內部重構，不修改任何既有 capability 的行為需求／spec 內容。`app-settings` 與 `rust-module-structure` 的既有需求維持不變，僅實作方式調整。）

## Impact

- 受影響程式碼：
  - `package.json`（移除 3 個未使用套件）
  - `src/App.tsx`（新增/改用 `DEFAULT_APP_SETTINGS`，三處預設值改寫）
  - `src-tauri/src/lib.rs`（fallback 改用 `AppSettings::default()`）
- 不影響任何 Tauri command 簽章、IPC 介面或資料庫結構
- 風險低：純重構，需以現有設定值驗證行為不變（見 design.md 的驗證方式）
