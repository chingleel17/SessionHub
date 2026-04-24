## 1. Rust：AppSettings 新增 show_status_bar 欄位

- [x] 1.1 在 `src-tauri/src/types.rs` 的 `AppSettings` struct 新增 `show_status_bar: bool` 欄位，加上 `#[serde(default = "default_true")]` 屬性確保向後相容（缺少欄位時預設 true）
- [x] 1.2 確認 `default_true()` helper fn 已存在於 types.rs，若無則新增

## 2. TypeScript：型別與設定表單

- [x] 2.1 在 `src/types/index.ts` 的 `AppSettings` interface 新增 `showStatusBar: boolean`
- [x] 2.2 在 `src/App.tsx` 的 `DEFAULT_SETTINGS` 常數（或 fallback 初始值）新增 `showStatusBar: true`

## 3. 翻譯 key

- [x] 3.1 在 `src/locales/zh-TW.ts` 新增 `statusBar` 命名空間：`noEvent`（無事件文字）、`active`（進行中）、`waiting`（等待回應）、`showStatusBar`（設定開關標籤）
- [x] 3.2 在 `src/locales/en-US.ts` 新增對應的英文翻譯 key

## 4. StatusBar 元件

- [x] 4.1 建立 `src/components/StatusBar.tsx`，Props 包含：`lastBridgeEvent: { entry: BridgeEventLogEntry; receivedAt: Date } | null`、`onOpenEventMonitor: () => void`、`activeSessions: number`、`waitingSessions: number`、`isLoadingSessions: boolean`
- [x] 4.2 在 StatusBar 左側渲染 Bridge 事件摘要（時間、provider、eventType、狀態色標、cwd 截斷至 40 字元）；無事件時顯示 `t("statusBar.noEvent")`；整個左側區段可點擊觸發 `onOpenEventMonitor`
- [x] 4.3 在 StatusBar 右側渲染 session 計數徽章：「▶ N 進行中」與「⏳ N 等待回應」；loading 時顯示 `-`；計數為 0 時套用低對比樣式

## 5. Session 計數邏輯

- [x] 5.1 在 `src/App.tsx` 新增 `useMemo` 計算 `activeSessions` 與 `waitingSessions`：從 React Query sessions 快取的 `SessionInfo[]` 中，以 `updated_at` 距今 30 分鐘內且非 archived 判斷 active；以 30 分鐘到 2 小時內且非 archived 判斷 waiting（簡易推斷，不觸發額外 invoke）

## 6. App.tsx 整合

- [x] 6.1 在 `src/App.tsx` import `StatusBar`
- [x] 6.2 在 `App.tsx` JSX `<main>` 最底部插入 `{settingsForm.showStatusBar && <StatusBar ... />}`，傳入 `lastBridgeEvent`、`onOpenEventMonitor`、`activeSessions`、`waitingSessions`、`isLoadingSessions`
- [x] 6.3 在 `SettingsView` 的 Props 呼叫處移除 `lastBridgeEvent` 和 `onOpenEventMonitor`（若這兩個 prop 僅用於設定頁底部狀態列）；若 SettingsView 其他地方仍需要，則保留

## 7. 設定頁 UI

- [x] 7.1 在 `src/components/SettingsView.tsx` 的一般設定區塊（`settings.general.title`）新增 `show_status_bar` checkbox，標籤使用 `t("statusBar.showStatusBar")`

## 8. 移除設定頁重複的 bridge-status-bar

- [x] 8.1 從 `src/components/SettingsView.tsx` 移除底部的 `bridge-status-bar` `<div>` 區段（整個 lastBridgeEvent 條件渲染塊）
- [x] 8.2 從 `src/components/SettingsView.tsx` Props 型別移除 `lastBridgeEvent`（`onOpenEventMonitor` 保留，供設定頁「開啟事件監視器」按鈕使用）
- [x] 8.3 從 `src/App.css` 移除或保留 `.bridge-status-bar*` 樣式（移至全域 StatusBar CSS 或重新利用）

## 9. CSS

- [x] 9.1 在 `src/App.css` 新增 `.global-status-bar` 樣式：`position: sticky; bottom: 0; height: 28px; display: flex; align-items: center; z-index: 100; border-top: 1px solid var(--border-color)`，背景使用主題變數
- [x] 9.2 新增 `.global-status-bar-left`、`.global-status-bar-right`、`.global-status-bar-event`（可點擊區域）、`.global-status-bar-count`、`.global-status-bar-count--zero`（低對比）等子元素樣式
- [x] 9.3 確保主內容區域（`.main-content` 或 `.content-area`）加入足夠的 `padding-bottom` 或 StatusBar 使用 sticky 不需額外處理

## 10. 驗證

- [x] 10.1 執行 `cd src-tauri && cargo check` 確認無 Rust 編譯錯誤
- [x] 10.2 執行 `bun run build` 確認前端 TypeScript 無錯誤且 build 成功




