## 1. 依賴安裝

- [x] 1.1 在 `src-tauri/Cargo.toml` 新增 `tauri-plugin-notification` 依賴
- [x] 1.2 在 `src-tauri/src/lib.rs` 的 `run()` 中加入 `.plugin(tauri_plugin_notification::init())`

## 2. Rust 後端 — AppSettings 擴充

- [x] 2.1 在 `types.rs` 的 `AppSettings` struct 新增 `enable_intervention_notification: bool` 欄位，serde default 為 `true`
- [x] 2.2 確認 `settings.rs` 的 `get_settings` 對缺少欄位的舊 JSON 使用 serde default 正確回退

## 3. Rust 後端 — 通知 Command

- [x] 3.1 在 `commands/` 新增 `notifications.rs` 模組，實作 `send_intervention_notification(session_id, project_name, summary)` command
- [x] 3.2 通知標題固定為 `SessionHub — 需要您介入`，內文為 `{project_name}: {summary 前 60 字元}`
- [x] 3.3 通知使用 `tag = "sessionhub-{session_id}"`, `group = "intervention"` 去重
- [x] 3.4 在 `commands/mod.rs` 登記新模組，在 `lib.rs` `invoke_handler![]` 新增 command

## 4. 前端 — 狀態轉換偵測與通知觸發

- [x] 4.1 在 `App.tsx` 新增 `useRef<Map<string, string>>(new Map())` 追蹤各 session 上一次的 activity status
- [x] 4.2 在活動狀態輪詢結果處理中，比對前後狀態：若 session 從非 `waiting` 轉為 `waiting`，且 `settings.enableInterventionNotification === true`，則呼叫 `invoke("send_intervention_notification", {...})`
- [x] 4.3 通知呼叫失敗時 silent fail（catch 後 console.warn，不顯示 toast 錯誤）

## 5. 前端 — 通知點擊聚焦

- [x] 5.1 在 `tauri-plugin-notification` 的 notification action callback 中（或透過 Tauri event），接收點擊事件並取得 `session_id`
- [x] 5.2 呼叫 Tauri `window.__TAURI__.window.getCurrentWindow().setFocus()` 帶視窗至前景
- [x] 5.3 根據 `session_id` 找到對應的 `projectKey`，呼叫 `setActiveView(projectKey)` 切換至對應 tab

## 6. 設定頁 — 通知開關

- [x] 6.1 在 `SettingsDialog.tsx`（或對應設定元件）的通知區塊新增 `enable_intervention_notification` toggle，label 翻譯 key 為 `settings.enableInterventionNotification`
- [x] 6.2 在 `src/i18n/` 加入翻譯字串：`settings.enableInterventionNotification`（繁中：`AI 介入通知`）與說明文字
- [x] 6.3 確認 toggle 狀態與 `settings` state 雙向綁定，儲存後寫入 settings.json

## 7. Hook 腳本 — PowerShell WinRT Toast 片段

- [x] 7.1 在 provider bridge hook 腳本樣板中新增 PowerShell WinRT Toast 發送函式 `Send-InterventionNotification`，接收 `$sessionId`, `$projectName`, `$summary` 參數
- [x] 7.2 在 hook 偵測到 `waiting` 狀態事件時呼叫該函式，並設定 Toast tag 為 `sessionhub-{sessionId}`
- [x] 7.3 確認函式加上 `try/catch`，發送失敗時不中斷 hook 主流程

## 8. 測試驗收

- [x] 8.1 手動測試：觸發 Copilot session 進入 `waiting` 狀態，確認 SessionHub 發出 Toast
- [x] 8.2 確認關閉設定開關後不再發送通知
- [x] 8.3 確認同一 session 持續 `waiting` 時只有一筆通知出現在通知中心
- [x] 8.4 確認點擊 Toast 後視窗前景化並聚焦到正確 project tab
- [x] 8.5 `cargo test` 通過，`bun run build` 無 TypeScript 錯誤
