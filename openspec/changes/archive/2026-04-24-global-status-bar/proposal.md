## Why

目前的 Bridge 最後事件列只顯示在設定頁，使用者在瀏覽 Dashboard 或專案頁面時無法即時感知 AI session 的進行狀態。需要一條應用程式層級、常駐底部的狀態列，提供跨頁可見的 session 狀態摘要與最後事件資訊，並允許使用者選擇是否啟用。

## What Changes

- 新增全域 **StatusBar** 元件，固定顯示於應用程式最底部（sidebar 和主內容區下方）
- 狀態列顯示：
  - 最後一筆 Bridge 事件（時間、provider、事件類型、狀態色標、cwd）
  - 當前進行中 (`active`) 的 session 數量
  - 等待回應 (`waiting`) 的 session 數量（AI 已完成，等待使用者操作）
- `AppSettings` 新增 `show_status_bar: bool`（預設 `true`）
- 設定頁新增開關控制是否顯示狀態列
- 狀態列中的 Bridge 事件區段與設定頁底部的 bridge-status-bar **合併**（移除重複的設定頁狀態列，改為全域版本）
- 點擊狀態列的 Bridge 事件區段可直接開啟事件監視器 Dialog

## Capabilities

### New Capabilities

- `global-status-bar`：應用程式底部常駐狀態列，顯示 Bridge 最後事件與 session 活動統計，可在設定中啟用或關閉。

### Modified Capabilities

- `app-settings`：新增 `show_status_bar: bool` 設定欄位（預設 `true`），設定頁 UI 新增對應開關。

## Impact

- **新增元件**：`src/components/StatusBar.tsx`
- **修改 App.tsx**：新增全域 StatusBar 渲染（在 `<main>` 最底部），傳入 `lastBridgeEvent`、`onOpenEventMonitor`、session 活動計數
- **修改 Rust `AppSettings` struct**：新增 `show_status_bar: bool`（`#[serde(default = "default_true")]`）
- **移除設定頁底部 bridge-status-bar**：`SettingsView.tsx` 中的 bridge-status-bar 區段移至全域 StatusBar
- **Session 活動計數**：利用現有 `session-activity-status` 基礎設施，前端定期（每 30 秒）查詢所有 visible session 的狀態並統計 active / waiting 數量；或直接讀取已快取的 session stats 推斷（零額外後端 command，效能優先）
- **CSS**：`src/App.css` 新增 `.global-status-bar` 相關樣式
- **翻譯**：`zh-TW.ts` / `en-US.ts` 新增 `statusBar.*` keys
