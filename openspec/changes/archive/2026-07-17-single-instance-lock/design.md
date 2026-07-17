## Context

SessionHub 目前沒有任何單一實例機制，重複啟動會產生多個獨立實例。每個實例都會建立 tray icon、啟動檔案 watcher、開啟 SQLite `metadata.db` 連線並讀寫 `settings.json`，多開時會出現 tray 圖示重複、資料庫寫入競爭與設定互相覆寫的問題。

現況（`src-tauri/src/lib.rs`）：

- 主視窗 label 為 `"main"`；另有 quota overlay 與 tray panel 兩個輔助視窗
- Builder 目前依序註冊 `dialog`、`notification`、`opener`、`window_state` 四個 plugin
- 關閉主視窗時的行為是隱藏至 tray（因此「聚焦既有視窗」必須包含從隱藏狀態還原）

## Goals / Non-Goals

**Goals**

- 同一時間僅允許一個 SessionHub 實例執行
- 重複啟動時，既有實例的主視窗被顯示、取消最小化並帶到前景；第二個實例立即退出
- 主視窗隱藏至 tray 時，重複啟動同樣能還原視窗

**Non-Goals**

- 不處理跨使用者（不同 Windows 帳號）的實例互斥 — plugin 預設即為 per-user 範圍，符合需求
- 不實作命令列參數轉發（deep link / CLI args 轉交既有實例），目前無此需求
- 不變更關閉視窗隱藏至 tray 的既有行為

## Decisions

### 1. 使用官方 `tauri-plugin-single-instance`

採用 Tauri 官方 plugin 而非自行實作（如具名 mutex 或 lock file）。

- 理由：官方維護、跨平台、與 Tauri 2 生命週期整合；Windows 底層即為 CreateMutex + 訊息通知，自行實作沒有額外好處（KISS）
- 替代方案：`windows-sys` 手寫具名 mutex — 需自行處理第二實例通知既有實例的 IPC，複雜度高且易出錯，捨棄

### 2. Plugin 註冊順序：放在第一個

`tauri_plugin_single_instance::init(...)` 註冊於 Builder 最前面（在 `dialog` 等 plugin 之前）。官方文件明確建議 single-instance 必須是第一個註冊的 plugin，確保第二實例在初始化其他資源（tray、watcher、DB）之前就被攔截退出。

### 3. second-instance callback 只做「還原並聚焦主視窗」

callback 內以 `app.get_webview_window("main")` 取得主視窗，依序呼叫 `show()`、`unminimize()`、`set_focus()`。忽略帶入的 argv/cwd 參數（見 Non-Goals）。

## Risks / Trade-offs

- [plugin 與 `tauri-plugin-window-state` 或 deep-link 的順序衝突] → 依官方建議將 single-instance 放最前即可，目前未使用 deep-link，無實際衝突
- [開發模式 `bun tauri dev` 與已安裝的正式版互斥（同 identifier）] → 屬預期行為；開發時若正式版常駐 tray，需先關閉正式版，於 tasks 驗證步驟中註明

## Migration Plan

單純新增相依與少量程式碼，無資料遷移。rollback 即移除 plugin 註冊與相依。

## Open Questions

（無）
