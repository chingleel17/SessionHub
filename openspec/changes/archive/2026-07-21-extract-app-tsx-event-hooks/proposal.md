## Why

`src/App.tsx` 中有一個巨大的 `useEffect`（約第 1170～1440 行）集中訂閱 13 個 Tauri 事件（`copilot-sessions-updated`、`copilot-session-targeted`、`claude-session-targeted`、`copilot-activity-hint`、`claude-activity-hint`、`opencode-activity-hint`、`plan-file-changed`、`project-files-changed`、`quota-snapshots-updated`、`navigate-main-view` 等）。其中 `copilot-session-targeted` 與 `claude-session-targeted` 的處理邏輯幾乎完全相同（複製貼上），三個 `*-activity-hint` 監聽也高度相似的 `setQueriesData` 更新 pattern 重複 5 次以上。這讓 `App.tsx` 難以閱讀與維護，新增 provider 時得再複製一份幾乎相同的監聽程式碼。

## What Changes

- 抽出共用函式 `applySessionActivityHint()`（或等效命名），統一三個 `*-activity-hint` 事件的 `queryClient.setQueriesData` 更新邏輯，消除重複的 `findIndex` + 更新 pattern
- 抽出共用函式 `refreshSessionByCwd()` / `handleSessionTargeted()`，統一 `copilot-session-targeted` 與 `claude-session-targeted` 的處理邏輯（兩者僅事件名稱不同，行為完全相同）
- 將整個事件訂閱 `useEffect` 抽為獨立 custom hook（如 `src/hooks/useSessionRealtimeEvents.ts`），把 `listen()` 訂閱、cleanup、去抖動邏輯搬出 `App.tsx`，`App.tsx` 只呼叫該 hook 並傳入必要的 query client / refs
- 不改變任何事件的訂閱時機、監聽的事件名稱、UI 呈現結果或 realtime 狀態列行為

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

（無 — 純內部重構，不改變 `hook-driven-activity-status`、`session-activity-status`、`targeted-session-refresh`、`live-progress-sync` 等既有 capability 的行為需求，僅搬移/合併實作程式碼位置。）

## Impact

- 受影響程式碼：
  - `src/App.tsx`（移除大型 `useEffect` 內容，改為呼叫新 hook）
  - 新增 `src/hooks/useSessionRealtimeEvents.ts`（或依實作拆為多個 hook 檔案）
- 不影響任何 Tauri command 簽章、後端事件發送邏輯（後端事件名稱與 payload 不變）
- 依賴 change `cleanup-deps-and-settings-defaults` 先行套用（避免重構期間 diff 互相干擾），但技術上無強制先後順序
