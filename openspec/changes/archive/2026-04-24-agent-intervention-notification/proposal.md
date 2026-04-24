## Why

當 AI Agent（Copilot / OpenCode）需要使用者介入時（如完成回應等待驗收、提問需要選擇、權限請求需放行），使用者若不在 SessionHub 視窗前，完全無從得知，只能手動切換查看，造成等待浪費。透過 Windows 原生 Toast 通知，可讓使用者在任何情境下即時得知介入需求，同時保持通知可由使用者自行開關，不造成干擾。

## What Changes

- 新增 Windows Toast 通知發送能力（透過 Tauri 後端呼叫 WinRT Notification API）
- 新增介入事件偵測邏輯：依 session 活動狀態從 `waiting` 變化觸發通知
- 在 Hook 腳本中新增通知發送呼叫，讓 hook 執行時即可直接觸發系統通知
- SessionHub 在接收到 `waiting` 狀態更新事件時，同步透過應用端發送通知
- 在應用設定中新增通知開關（啟用/停用 Windows 通知）
- 通知點擊後，SessionHub 視窗浮出前景並聚焦至對應 session

## Capabilities

### New Capabilities

- `intervention-notification`: Windows Toast 通知系統，偵測 AI Agent 進入 `waiting` 狀態（需使用者介入）時發送系統通知，包含 session 摘要與專案名稱；通知可由使用者在設定中開關；點擊通知後 SessionHub 視窗前景化並聚焦對應 session

### Modified Capabilities

- `app-settings`: 新增 `enable_intervention_notification: bool` 設定欄位，控制是否發送 Windows Toast 通知

## Impact

- **Rust 後端**：新增 WinRT / Windows.UI.Notifications Toast 發送邏輯（`src-tauri/src/`）；`AppSettings` struct 新增 `enable_intervention_notification` 欄位
- **Hook 腳本**：provider bridge hook script 在 `waiting` 狀態時呼叫 notify 邏輯（可透過 Tauri command 或直接呼叫 PowerShell `New-BurntToastNotification` / WinRT）
- **前端**：設定頁新增通知開關 toggle；活動狀態輪詢邏輯在偵測到 `waiting` 轉換時呼叫後端 notify command
- **依賴**：`windows-sys` crate（已存在）擴展 Toast 所需 feature flags，或使用 `tauri-plugin-notification`
