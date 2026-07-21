## Context

SessionHub 目前的 Bridge 事件狀態列只存在於設定頁底部（`SettingsView.tsx`），當使用者切換到 Dashboard 或專案頁面時，便看不到任何 AI session 進行狀態或 Bridge 事件資訊。`session-activity-status` spec 已定義好基於靜態檔案推斷 session 狀態（active/waiting/idle/done）的後端邏輯，但前端尚未利用這些資料做常駐顯示。

目前 App.tsx 已持有：
- `bridgeEventLog: BridgeEventLogEntry[]`（最多 100 筆，含 provider/eventType/status/cwd）
- `lastBridgeEvent: { entry, receivedAt } | null`（5 分鐘自動清除）
- `showEventMonitor: boolean`（切換監視器 Dialog）

這些 state 已在 App 層，只需新增全域 StatusBar 元件並從 App 傳入所需 props。

## Goals / Non-Goals

**Goals:**
- 新增 `StatusBar` 元件，固定在應用程式視窗最底部，跨所有 View 常駐顯示
- 顯示最後一筆 Bridge 事件（時間、provider、事件類型、狀態色標、cwd）
- 顯示當前 active / waiting session 計數（從前端 React Query 快取推斷，不增加額外後端 command）
- `AppSettings` 新增 `show_status_bar: bool`（預設 `true`），可在設定頁關閉
- 點擊 Bridge 事件區段可開啟事件監視器 Dialog
- 移除設定頁底部重複的 `bridge-status-bar`，改由全域 StatusBar 統一負責

**Non-Goals:**
- 不新增即時 Rust polling command（效能考量，使用前端已有快取）
- 不顯示個別 session 名稱（只顯示數量）
- 不支援可拖曳調整高度的 StatusBar
- 不在 StatusBar 做 session 活動的 drill-down 導航

## Decisions

### 1. session 計數來源：前端快取推斷，不增加新 Tauri command

**決定**：利用 React Query 已快取的 `sessions` 資料（`SessionInfo[]`）加上前端定期呼叫現有 `get_session_activity_statuses` command（若已實作）或直接利用 `is_live` flag 推斷。

若 `get_session_activity_statuses` 尚未實作，則以 `SessionInfo.updated_at` 距今時間做簡易推斷（30 分鐘內 = active，2 小時內 = possibly active），避免 StatusBar 造成大量額外 I/O。

**替代方案**：每 10 秒輪詢新 Rust command → 拒絕，因為每個 visible session 各需讀取末尾 30 行，30 個 session 就有 900 次 I/O，嚴重影響效能。

**最終選擇**：以前端 React Query 快取的 sessions 與 `SessionStats.is_live` 欄位做計數，零額外後端呼叫，30 秒間隔重新整理（由已有 webhook 觸發，而非定時器）。

### 2. StatusBar 渲染位置：App.tsx `<main>` 最底部

**決定**：在 `App.tsx` 的 `<main>` JSX 最後插入 `{showStatusBar && <StatusBar ... />}`，StatusBar 用 CSS `position: sticky; bottom: 0` 固定在視窗底部，z-index 低於 Dialog 但高於一般內容。

### 3. 移除設定頁 bridge-status-bar

設定頁已有的 `bridge-status-bar` 與新 StatusBar 功能重疊，合併後移除以避免資訊重複。`SettingsView` props `lastBridgeEvent` 與 `onOpenEventMonitor` 保留（用於其他用途），但 JSX 中的 bridge-status-bar `<div>` 區段移除。

### 4. AppSettings 欄位命名

Rust struct: `show_status_bar: bool`（`#[serde(default = "default_true")]`）→ TypeScript `showStatusBar: boolean`，與現有 `showArchived` 命名風格一致。

## Risks / Trade-offs

- **[Risk] session 計數可能不即時** → 僅用 `SessionInfo.updated_at` 或 `is_live` 推斷，精度為最後一次 webhook 更新時間。接受此限制，狀態列定位為「概覽」而非「精準即時監控」。
- **[Risk] 移除設定頁 bridge-status-bar 時遺漏清理 CSS** → 任務列表明確標記需清除對應 CSS class，`cargo check + bun build` 驗證不留孤立樣式（TS/CSS 編譯不報錯即可接受）。
- **[Risk] StatusBar 高度壓縮主內容空間** → StatusBar 高度設計為 28–32px，使用 CSS `min-height` 限制，主內容區域需加對應 `padding-bottom`。
