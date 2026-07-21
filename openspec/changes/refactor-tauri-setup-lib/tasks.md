## 1. 前置分析

- [x] 1.1 逐行讀取 `src-tauri/src/lib.rs` 第 281-617 行，標記 quota monitoring 啟動、背景輪詢執行緒、tray icon 建構、視窗事件處理、command 註冊各區塊的精確行號邊界
- [x] 1.2 記錄每個 `std::thread::spawn` closure 捕獲的變數與其所有權轉移方式（`.clone()` 呼叫位置）作為核對基準
- [x] 1.3 記錄 4 個 tray 選單字串目前的精確文字內容與使用位置

## 2. 建立新模組與抽出常數

- [x] 2.1 新增 `src-tauri/src/app_setup.rs`，於 `lib.rs` 加入 `mod app_setup;` 宣告
- [x] 2.2 於 `app_setup.rs` 頂部新增 `TRAY_MENU_SHOW_WINDOW`、`TRAY_MENU_TOGGLE_OVERLAY`、`TRAY_MENU_TOGGLE_OVERLAY_LOCK`、`TRAY_MENU_QUIT` 常數（依 design.md D2，含說明註解）
- [x] 2.3 將 `MenuItemBuilder::new(...)` 呼叫改用上述常數

## 3. 拆分 setup 函式至 app_setup.rs

- [x] 3.1 於 `app_setup.rs` 建立 `pub(crate) fn setup_quota_monitoring(app, settings)`，搬入原 `lib.rs` 第 317-382 行邏輯（讀 DB 快取 + spawn 首次刷新執行緒），並補上該函式所需的 `use` 引入
- [x] 3.2 於 `app_setup.rs` 建立 `pub(crate) fn spawn_quota_poller_thread(app_handle)`，搬入原第 384-418 行邏輯（背景輪詢執行緒），確認 60 秒檢查間隔、30 分鐘刷新間隔常數不變
- [x] 3.3 於 `app_setup.rs` 建立 `pub(crate) fn build_tray_icon(app)`，搬入原第 420-513 行邏輯（icon 建構、選單建立、事件綁定）
- [x] 3.4 改寫 `lib.rs` 的 `run()` 的 `.setup(|app| { ... })` closure，依序呼叫 `app_setup::setup_quota_monitoring(...)`、`app_setup::spawn_quota_poller_thread(...)`、`app_setup::build_tray_icon(...)`，保持原始執行順序（依 design.md 風險項）
- [x] 3.5 確認啟動時 tray/overlay 初始化區塊（原第 515-523 行，呼叫 `create_quota_overlay` 等 `lib.rs` 既有函式）與新函式的呼叫順序關係正確；此區塊本身不搬移（依 design.md Non-Goals，`create_quota_overlay` 等屬範圍外）
- [x] 3.6 確認新模組函式的可見性為 `pub(crate)`（符合既有 `rust-module-structure` spec 的最小公開原則要求），僅 `lib.rs` 呼叫

## 4. 核對與驗證

- [x] 4.1 逐一比對步驟 1.2 記錄的執行緒捕獲變數，確認拆分後語意相同
- [x] 4.2 執行 `cargo check`（於 `src-tauri/`）確認編譯成功
- [x] 4.3 執行 `cargo clippy` 確認無新增警告
- [x] 4.4 執行既有 Rust 測試（`cargo test`）確認無回歸
- [x] 4.5 手動測試：啟動應用程式，確認 tray icon 正確顯示、選單四個項目文字與功能正常（顯示視窗、切換 overlay、切換 overlay 鎖定、退出）、quota 背景刷新正常運作（可縮短輪詢間隔常數暫時驗證後改回）
