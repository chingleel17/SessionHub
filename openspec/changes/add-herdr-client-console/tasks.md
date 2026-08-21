## 0. client 附著偵測（socket API）

- [x] 0.1 以 `herdr status server` 取得 socket 路徑，推導 Windows named pipe 名稱（不硬編碼）
- [x] 0.2 以 `client.window_title.clear` 探測 client 附著狀態，解析 `no_foreground_client`
- [x] 0.3 `ensure_herdr_client_console` 優先採用 socket 判定，無法取得時退回子程序存活判定
- [x] 0.4 新增解析與判定的單元測試（socket 路徑、附著/未附著、非預期回應）

## 1. herdr client console 生命週期

- [x] 1.1 在 `src-tauri/src/sessions/herdr.rs` 新增 client console 追蹤（`Mutex<Option<Child>>` 全域單例），以 `try_wait()` 判定存活
- [x] 1.2 新增 spawn client console 函式：`CREATE_NEW_CONSOLE` + MSYS stackdump 緩解環境
- [x] 1.3 新增 server 就緒輪詢（200ms 間隔、5 秒上限），逾時回報明確錯誤
- [x] 1.4 新增取得 client console PID 的函式供視窗提升使用

## 2. 啟動流程調整

- [x] 2.1 `terminal.rs` 的 `launch_via_herdr` 移除「服務未執行即報錯」分支，改為確保 client console 存在
- [x] 2.2 冷啟動時先 spawn console、等待 server 就緒，再 `tab create --focus`
- [x] 2.3 重用既有 console 時於 tab 建立後提升 client console 視窗

## 3. 視窗提升

- [x] 3.1 `platform/win32_focus.rs` 新增依 PID 比對頂層視窗並提升前景的函式
- [x] 3.2 `commands/tools.rs` 的 `focus_terminal_window_internal` 在 herdr 分支補上視窗提升，失敗不阻斷

## 4. 前端與文案

- [x] 4.1 `SettingsView.tsx` 移除 `herdrServerRunning` 標示分支
- [x] 4.2 移除 `settings.launcher.herdrServerStopped`（zh-TW / en-US）

## 5. 驗證

- [x] 5.1 `cargo test` 全數通過（227 passed）
- [x] 5.2 `cargo clippy` 無新增警告
- [x] 5.3 `tsc --noEmit` 無錯誤
- [x] 5.4 冷啟動情境手動驗證：herdr server 停止後，於 SessionHub 觸發啟動應自動開出 TUI 並顯示新 tab
      （需 `herdr server stop`，會中斷使用者當前 herdr session，待使用者許可後執行）
- [x] 5.5 重用情境手動驗證：dev 版實測通過，已有 client 附著時不再重複開啟 console

### 驗證環境注意事項

以 `npm run tauri dev` 從 herdr pane 內啟動時，SessionHub 會繼承 `HERDR_*` 巢狀偵測變數，
其 spawn 的 client 因而異常結束——此為開發環境特有情境，已由 1.2 的 `env_remove` 處理。
實際使用情境（`launchOnStartup` 由 Windows 啟動，或直接執行 exe）不會繼承這些變數。

`target/debug/session-hub.exe` 內部指向 Vite dev server（`devUrl: http://localhost:1420`），
單獨執行會顯示無法連線，必須搭配 `npm run tauri dev`；production 版則將前端打包進 exe，不需 host。
