## 1. 相依安裝

- [x] 1.1 於 `src-tauri/` 執行 `cargo add tauri-plugin-single-instance` 新增相依（Tauri 2 對應版本 `2.x`）

## 2. Plugin 註冊

- [x] 2.1 在 `src-tauri/src/lib.rs` 的 `tauri::Builder::default()` 鏈最前面（`dialog` plugin 之前）註冊 `tauri_plugin_single_instance::init(...)`
- [x] 2.2 在 second-instance callback 中取得 `"main"` 視窗，依序呼叫 `show()`、`unminimize()`、`set_focus()`；忽略 argv/cwd 參數

## 3. 驗證

- [x] 3.1 `cargo check`（或 `bun tauri build` 的 Rust 編譯階段）通過
- [ ] 3.2 手動驗證：啟動 app 後再次啟動，第二個實例退出且不出現重複 tray icon
- [ ] 3.3 手動驗證：主視窗最小化與隱藏至 tray 兩種狀態下重複啟動，主視窗皆被還原並帶到前景（注意：測試前先關閉已安裝的正式版，避免 dev 與正式版互斥干擾）
