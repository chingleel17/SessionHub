## 1. 依賴引入與驗證

- [x] 1.1 於 `src-tauri/` 執行 `cargo add tauri-plugin-autostart@2.5.1`，確認 lockfile 的 Tauri 2.11.5 維持相容，並記錄實際版本號至 `Cargo.toml`
- [x] 1.2 以 `tauri_plugin_autostart::Builder::new().args(["--autostart"]).build()` 註冊外掛；不得安裝 `@tauri-apps/plugin-autostart`，避免產生第二條前端寫入路徑
- [x] 1.3 執行 `cargo build` 確認依賴引入後可編譯通過

## 2. 後端設定型別

- [x] 2.1 於 `src-tauri/src/types/settings.rs` 新增 `default_launch_on_startup()`（回傳 `false`）與 `default_start_minimized_on_startup()`（回傳 `true`）兩個預設函式
- [x] 2.2 於 `AppSettings` struct 新增 `launch_on_startup: bool` 與 `start_minimized_on_startup: bool`，各自標註對應的 `#[serde(default = "...")]`
- [x] 2.3 於 `src-tauri/src/settings.rs:223` 的 `AppSettings::default()` 補上 `launch_on_startup: false` 與 `start_minimized_on_startup: true`

## 3. 修復既有 AppSettings literal 編譯錯誤

- [x] 3.1 補齊 `src-tauri/src/commands/mod.rs:43` 的 fallback literal 新欄位
- [x] 3.2 補齊 `src-tauri/src/quota/antigravity.rs:304` 與 `:351` 兩處測試 fixture 新欄位
- [x] 3.3 補齊 `src-tauri/src/quota/claude.rs:524` 測試 fixture 新欄位
- [x] 3.4 補齊 `src-tauri/src/quota/codex.rs:603` 測試 fixture 新欄位
- [x] 3.5 補齊 `src-tauri/src/quota/copilot.rs:234` 測試 fixture 新欄位
- [x] 3.6 執行 `cargo build` 與 `cargo test`，確認無遺漏的 literal 站點

## 4. Autostart 註冊邏輯

- [x] 4.1 於 `src-tauri/src/lib.rs` 的 `run()` builder 以 `tauri_plugin_autostart::Builder::new().args(["--autostart"]).build()` 註冊外掛
- [x] 4.2 於 `src-tauri/src/app_setup.rs` 新增 `sync_autostart_registration(app, settings) -> Result<(), String>` 輔助函式：依 `launch_on_startup` 呼叫 `enable()` 或 `disable()`，錯誤以 `Result<_, String>` 回傳（不得 `unwrap()`）
- [x] 4.3 於 `src-tauri/src/commands/settings.rs` 的 `save_settings` 中，在 `save_settings_internal()` 成功後呼叫 `sync_autostart_registration`，失敗時回傳 `Err(String)`（與 tray／overlay 副作用並列）
- [x] 4.4 於 `src-tauri/src/app_setup.rs` 新增 `reconcile_autostart_on_startup(app, settings)`：設定為啟用時一律呼叫 `enable()` 以更新目前執行檔路徑；設定為停用且 `is_enabled()` 為真時呼叫 `disable()`；失敗僅記錄不中止啟動
- [x] 4.5 於 `lib.rs` 的 `setup()` 中呼叫 `reconcile_autostart_on_startup`

## 5. 隱藏啟動至系統匣

- [x] 5.1 於 `src-tauri/src/app_setup.rs` 新增 `is_autostart_launch() -> bool`，以 `std::env::args()` 檢查是否含 `--autostart`
- [x] 5.2 於 `lib.rs` 的 `setup()` 開頭（早於 watcher／quota 初始化）判斷 `is_autostart_launch() && settings.launch_on_startup && settings.start_minimized_on_startup`，成立時對 `main` 視窗呼叫 `hide()`
- [x] 5.3 確認隱藏狀態下 `build_tray_icon` 仍正常建立系統匣圖示與「顯示視窗」選單項
- [x] 5.4 修改 `lib.rs` 的 `CloseRequested` 處理（現為 `lib.rs:351` 僅檢查 `minimize_to_tray`），改為 `minimize_to_tray || (launch_on_startup && start_minimized_on_startup)` 成立時 `prevent_close()` + `hide()`（見 design.md D7）
- [ ] 5.5 實測是否出現主視窗閃現；若明顯，改採 design.md D5 回退方案（`tauri.conf.json` 設 `"visible": false` + 非 autostart 啟動時於 `setup()` 末端 `show()`）並更新 design.md

## 6. 前端型別與預設值

- [x] 6.1 於 `src/types/index.ts` 的 `AppSettings` 介面新增 `launchOnStartup?: boolean` 與 `startMinimizedOnStartup?: boolean`
- [x] 6.2 於 `src/utils/appSettingsDefaults.ts` 的 `DEFAULT_APP_SETTINGS` 新增 `launchOnStartup: false` 與 `startMinimizedOnStartup: true`

## 7. 設定頁 UI 與翻譯

- [x] 7.1 於 `src/locales/zh-TW.json` 新增 `settings.fields.launchOnStartup`、`settings.fields.launchOnStartupDesc`、`settings.fields.startMinimizedOnStartup`、`settings.fields.startMinimizedOnStartupDesc` 四組文案
- [x] 7.2 於 `src/locales/en-US.json` 新增相同四組鍵值的英文文案
- [x] 7.3 於 `src/components/SettingsView.tsx` 的 `minimizeToTray` checkbox 之後，新增「開機時自動啟動」checkbox，沿用既有 `checkbox-group` / `settings-field-desc` 樣式，透過 `onFormChange` 更新表單
- [x] 7.4 新增「開機時隱藏至系統匣」checkbox，並在 `settingsForm.launchOnStartup` 為 falsy 時設為 `disabled`
- [x] 7.5 確認所有新增文案皆走 `t("key")`，JSX 中無硬編中文

## 8. 驗證

- [x] 8.1 執行 `cargo test` 與 `npm run lint`、`npm run build`，全部通過
- [ ] 8.2 手動驗證啟用流程：勾選「開機時自動啟動」並儲存後，確認 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` 出現含 `--autostart` 的 SessionHub 項目
- [ ] 8.3 手動驗證停用流程：取消勾選並儲存後，確認註冊表項目已移除
- [ ] 8.4 手動驗證隱藏啟動：以 `--autostart` 參數執行應用程式，確認主視窗不顯示、系統匣圖示存在，且點擊「顯示視窗」可正常喚出
- [ ] 8.5 手動驗證手動啟動不受影響：不帶參數執行時主視窗照常顯示
- [ ] 8.6 手動驗證對帳：於工作管理員「啟動」分頁停用 SessionHub 後重啟應用程式，確認註冊被還原
- [ ] 8.7 手動驗證向後相容：以不含新欄位的舊 `settings.json` 啟動，確認讀取正常且功能預設關閉
- [ ] 8.8 手動驗證關閉行為聯集：在 `minimize_to_tray: false` 但已啟用隱藏啟動的狀態下喚出視窗後點擊關閉鈕，確認應用程式收合至系統匣而非結束
- [ ] 8.9 手動驗證兩者皆停用時關閉鈕仍能正常結束應用程式
- [ ] 8.10 手動驗證 single-instance：隱藏常駐狀態下再次執行應用程式，確認不產生第二實例且視窗被喚出並取得焦點
- [x] 8.11 為 autostart 同步決策新增單元測試，至少覆蓋啟用、停用與同步失敗時保留 `settings.json` 中使用者選擇的狀態；執行 `cargo test` 確認通過
- [ ] 8.12 手動驗證過期參數：在 `launch_on_startup: false` 時以 `--autostart` 啟動，確認主視窗照常顯示
