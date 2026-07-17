## Why

SessionHub 目前可以重複啟動多個實例：專案沒有任何單一實例（single instance）機制。多開會造成 tray icon 重複出現、多個檔案 watcher 同時運作、SQLite `metadata.db` 寫入競爭，以及設定檔互相覆寫等問題。桌面工具型應用程式應保證同一時間只有一個實例在執行。

## What Changes

- 加入官方 `tauri-plugin-single-instance` plugin，保證同一時間只有一個 SessionHub 實例
- 使用者重複啟動時，第二個實例不會建立新視窗，改為通知既有實例後立即退出
- 既有實例收到通知後，將主視窗顯示（若隱藏至 tray 則還原）、取消最小化並帶到前景取得焦點

## Capabilities

### New Capabilities
- `single-instance-lock`: 應用程式單一實例保證 — 重複啟動時聚焦既有視窗，第二個實例自動退出

### Modified Capabilities

（無 — 不影響既有 capability 的需求）

## Impact

- `src-tauri/Cargo.toml`：新增 `tauri-plugin-single-instance` 相依
- `src-tauri/src/lib.rs`：Builder 註冊 plugin 與 second-instance callback（需最先註冊，官方建議放在第一個 plugin）
- 不影響前端程式碼、資料庫 schema 或既有 provider 邏輯
