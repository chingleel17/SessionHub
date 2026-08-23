## Why

herdr 是 server（headless 背景常駐）+ client（TUI）雙程序架構。SessionHub 目前僅透過 socket API 呼叫 `herdr tab create` / `herdr pane run`，這些只在 server 內建立 pane 並送出指令，**不會開啟任何 TUI client**。結果是使用者在 SessionHub 點擊啟動後畫面上什麼都沒有發生，必須自行開一個終端輸入 `herdr` 附著到 session，才看得到剛才建立的 tab。

原設計（`2026-08-19-add-herdr-terminal-launcher`）刻意規定 herdr 模式「不另開作業系統 console 視窗」，用意是避免像 shell 模式那樣每個 session 累積一個零散視窗。但該規定忽略了 herdr 需要一個 client 程序才有畫面，導致 herdr 模式在沒有既存 TUI 時實質不可用。

經查證，herdr 未提供偵測「TUI 是否已附著」的介面：`herdr api schema`、`herdr api snapshot`、`herdr status`、`herdr session list` 皆不含 client / attach 相關欄位。因此重用判斷只能由 SessionHub 自行追蹤它所建立的 client 程序。

## What Changes

- herdr 模式在啟動時確保存在一個由 SessionHub 建立的 herdr TUI client console：以 `CREATE_NEW_CONSOLE` 直接 spawn `herdr`（bare 指令即「Launch or attach to the persistent session」，已驗證不會產生第二個 session）。
- spawn 前清除繼承自 herdr session 的巢狀偵測環境變數（`HERDR_ENV`、`HERDR_PANE_ID`、`HERDR_SOCKET_PATH`、`HERDR_STARTUP_CWD`、`HERDR_TAB_ID`、`HERDR_WORKSPACE_ID`）。SessionHub 若由 herdr session 內啟動會繼承這些變數，herdr client 便以 `nested herdr is disabled by default` 拒絕啟動並立即結束——實測確認這是「背景有動作但終端不會開」的直接成因。
- spawn 後確認子程序仍存活才視為成功；若隨即結束則回報明確錯誤，避免回報「已開啟」但畫面上什麼都沒發生。
- 啟動前以 socket API `client.window_title.clear` 詢問 server 是否已有 client 附著：無 client 時回報 `reason: "no_foreground_client"`，有則回報 `cleared`。此判定涵蓋**使用者手動開啟的 TUI** 與本應用程式重啟前建立的 client，是重用決策的主要依據。選用 `clear` 是因其冪等且無殘留副作用（herdr 本身會持續設定該標題）。
- 該判定無法取得時（socket 無法連線等），退回以 `std::process::Child` 的 `try_wait()` 判定本程序自建 client 是否存活。
- 移除「herdr 已安裝但服務未執行」即回傳錯誤的行為：spawn client console 本身即會啟動 server。改為 spawn 後輪詢 server 就緒（間隔 200ms、上限 5 秒）再建立 tab；逾時才回報錯誤。
- `ToolAvailability.herdr_server_running` 不再作為 herdr 模式的可用性條件，設定頁 launcher 標示不再顯示「服務未執行」。
- herdr socket API 呼叫（`status` / `api snapshot` / `tab create` / `pane run` / `tab focus`）加上 `CREATE_NO_WINDOW`，避免閃現 console 視窗；此為新增就緒輪詢後每次啟動最多多出 25 次呼叫所突顯的既有問題。

**不在此變更範圍**

- 將 herdr client console 視窗提升至前景。原先規劃此項以避免「tab 在背景切換但視窗沒出現」，經實測確認在本機環境不可行，故移除：
  - herdr client 程序不擁有頂層視窗（`MainWindowHandle` 為 0），無法以 `GetWindowThreadProcessId` 比對，PID 定位不成立。
  - 以 `cmd /c title <標記>` 預設 console 標題的替代方案亦失效——herdr TUI 啟動時會覆寫 console 標題，標記不留存。
  - 本機以 Windows Terminal 為預設終端宿主，所有 console 併入單一 `CASCADIA_HOSTING_WINDOW_CLASS` 視窗並以分頁承載，該 console 本身不是視窗，無視窗層級可供提升。
**已知限制**：當 socket 探測無法取得結果時，退回的子程序存活判定不跨程序保存，此情況下本應用程式重啟後的首次啟動會再建立一個 client console。實測多個 client 可同時附著於同一 `default` session 且不影響運作，故接受此退路行為。

## Capabilities

### New Capabilities

無。本變更修正既有 herdr 啟動流程的行為。

### Modified Capabilities

- `terminal-launcher`: herdr 模式改為會建立一個 client console 程序；「服務未執行」由錯誤改為自動啟動並等待就緒；console creation flags 與 MSYS 緩解環境的適用範圍隨之調整。

`terminal-focus` 不受影響：既有「herdr 模式以 tab 識別碼聚焦、不套用視窗標題比對」的要求維持不變。

## Impact

**後端（Rust）**
- `src-tauri/src/sessions/herdr.rs` — 新增 client console 生命週期管理（spawn / 存活判定 / server 就緒輪詢）
- `src-tauri/src/sessions/terminal.rs` — `launch_via_herdr` 流程調整
- `src-tauri/src/sessions/herdr.rs` — `run_herdr` 補上 `CREATE_NO_WINDOW`，避免每次 socket API 呼叫閃現 console 視窗

**前端（TypeScript）**
- `src/components/SettingsView.tsx` — 移除 `herdrServerRunning` 標示分支
- `src/locales/zh-TW.ts`、`src/locales/en-US.ts` — 移除 `settings.launcher.herdrServerStopped`

**相依性**
- 外部 CLI：herdr（已驗證版本 0.8.0-preview.2026-08-04）
