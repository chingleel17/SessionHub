## Context

SessionHub 已有 `session-activity-status` 規格實作活動狀態偵測（idle / active / waiting / done），並透過前端定時輪詢取得各 session 狀態。當偵測到 `waiting` 狀態（AI 完成回應等待使用者介入）時，目前僅在 UI 圖示上顯示，使用者若未聚焦視窗則完全無感知。

Hook 腳本（已安裝於 provider bridge）在每次 AI agent 狀態切換時執行，是距離事件最近的觸發點。SessionHub 應用程式同樣可在輪詢到 `waiting` 狀態切換時發送通知。

## Goals / Non-Goals

**Goals:**
- 當 AI Agent 進入 `waiting` 狀態時，透過 Windows Toast 通知使用者需要介入
- 通知由 **SessionHub 應用端**發送（透過 tauri-plugin-notification）
- Hook 腳本亦可**獨立**發送 PowerShell WinRT Toast（SessionHub 未開啟時仍可運作）
- 使用者可在設定頁以單一開關控制通知啟閉
- 通知點擊後，SessionHub 視窗前景化並聚焦對應 session
- 同一 session 的相同狀態切換不重複通知（去重）

**Non-Goals:**
- macOS / Linux 通知（Windows Only 專案）
- 通知分級（目前統一為「需要介入」）
- 推送到手機或其他裝置
- 通知歷史紀錄
- 通知排程或靜音時段

## Decisions

### 決策 1：以 `tauri-plugin-notification` 作為應用端 Toast 實作

**選項比較**：
- `tauri-plugin-notification`：官方 Tauri 2 plugin，API 簡潔，維護由 Tauri 官方負責
- `windows-sys` WinRT COM 手動實作：彈性高但需大量 unsafe 程式碼，維護成本高
- 呼叫外部 PowerShell 行程：可行但啟動延遲 ~200ms，且依賴 shell 執行環境

**選擇**：`tauri-plugin-notification`。專案已使用 `tauri-plugin-opener` / `tauri-plugin-dialog`，架構一致；API 直接，不需額外 unsafe 程式碼。

### 決策 2：Hook 腳本使用 PowerShell WinRT 發送（獨立於 SessionHub）

Hook 腳本在 AI agent process 執行，與 SessionHub 為不同行程，無法直接呼叫 Tauri IPC。Hook 端透過 PowerShell 呼叫 `[Windows.UI.Notifications.ToastNotificationManager]` 發送原生 Toast，不依賴 SessionHub 是否開啟。

若 SessionHub 同時開啟，兩端均偵測到狀態 → 應用端通知覆蓋 hook 通知（Windows Toast 去重依 tag/group 實作）。

### 決策 3：去重機制使用記憶體 HashSet + 狀態比對

在 Rust `AppState`（或前端 React state）維護 `HashMap<session_id, last_notified_status>`。當新的輪詢結果顯示 session 從非 `waiting` 狀態**轉換**為 `waiting` 時才觸發通知；若已是 `waiting` 且未改變則不再重複通知。重啟應用後 HashSet 清空，可再次通知（合理行為）。

### 決策 4：通知點擊 → 視窗前景化聚焦對應 session

使用 Tauri window focus API（`window.set_focus()`）加上前端路由切換至對應 project tab，以 `session_id` 作為定位 key。通知攜帶 session_id 作為 action data。

## Risks / Trade-offs

- **[Risk] Hook 腳本與應用端重複通知** → 以相同 Toast `tag`（session_id）避免，同一 tag 新通知覆蓋舊通知而非疊加
- **[Risk] tauri-plugin-notification 在 Windows 需 AppUserModelID** → Tauri build 設定中的 identifier（`com.ching.sessionhub`）已滿足此要求
- **[Risk] 狀態輪詢頻率決定通知延遲** → 現有輪詢間隔若為 30s，使用者最多等 30s 才收到通知；可接受
- **[Risk] 使用者禁用系統通知** → 無法繞過；僅記錄 debug log，不顯示錯誤 toast

## Migration Plan

1. 新增 `tauri-plugin-notification` 依賴
2. 在 `AppSettings` 新增 `enable_intervention_notification` 欄位（預設 `true`）
3. 在 `lib.rs` 新增 `send_intervention_notification` command
4. 前端輪詢邏輯新增狀態轉換偵測 + 呼叫 notify command
5. 設定頁新增通知開關 toggle
6. Hook 腳本樣板新增 PowerShell WinRT Toast 片段（可選安裝）
7. 無資料遷移需求（純新增功能）

## Open Questions

- 通知文字語言：跟隨 SessionHub 介面語言（繁中）或固定英文？（目前計畫：繁中）
- Hook 腳本的 WinRT 片段是否預設啟用，或需使用者在設定頁點擊「重新安裝 Hook」後才生效？
