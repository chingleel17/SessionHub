## Why

SessionHub 的核心價值來自持續在背景接收各 provider 的 hook 事件（介入提醒、session 結束通知、quota 監控輪詢）。目前使用者每次開機後都必須手動啟動應用程式，在手動啟動前發生的 hook 事件與 quota 變化完全遺失，讓「常駐監控」的定位無法成立。

已有的 `minimize_to_tray` 設定讓應用程式在關閉視窗後能常駐系統匣，但缺少「開機自動接手」這一段，常駐鏈路仍有缺口。

## What Changes

- 新增 `launchOnStartup` 應用程式設定（預設關閉），啟用後由作業系統於使用者登入時自動啟動 SessionHub。
- 新增 `startMinimizedOnStartup` 應用程式設定（預設啟用，僅在 `launchOnStartup` 啟用時生效），控制開機自動啟動時主視窗是否隱藏、僅常駐系統匣。
- 導入 `tauri-plugin-autostart` 2.5.1（僅 Rust）作為註冊機制，並在自動啟動時附帶 `--autostart` 啟動參數供後端識別啟動來源。
- `save_settings` 於每次儲存時將設定狀態同步推送至作業系統註冊（啟用時 `enable()`、停用時 `disable()`），與現有 tray／overlay 即時生效的副作用採同一模式。
- 應用程式啟動時執行一次對帳（reconcile）：以 settings.json 為單一真實來源，修正使用者從工作管理員「啟動」分頁外部關閉所造成的狀態漂移。
- 啟用隱藏啟動時，關閉主視窗改為收合至系統匣而非結束應用程式（與既有 `minimize_to_tray` 為聯集條件），避免使用者喚出視窗後按 X 就中斷背景監控。
- 設定頁「一般」區塊新增兩個對應開關與說明文字（zh-TW / en-US 雙語）。
- 非目標：不支援 Windows 以外平台的自動啟動驗證（專案本身為 Windows 桌面應用）；不提供延遲啟動秒數、不提供每位使用者以外的系統層級（HKLM）註冊。

## Capabilities

### New Capabilities
- `launch-on-startup`: 開機自動啟動的設定項、作業系統註冊與解除註冊、啟動來源識別（`--autostart`）、隱藏啟動至系統匣，以及設定與作業系統狀態的對帳規則。

### Modified Capabilities
- `app-settings`: `AppSettings` 新增 `launchOnStartup` 與 `startMinimizedOnStartup` 兩個欄位，且 `save_settings` 的行為契約擴充為「儲存後同步作業系統自動啟動註冊狀態」。

## Impact

**後端（Rust）**
- `src-tauri/Cargo.toml`：新增 `tauri-plugin-autostart` 依賴。
- `src-tauri/src/types/settings.rs`：`AppSettings` 新增兩個 bool 欄位與其 `#[serde(default = ...)]` 預設函式。
- `src-tauri/src/settings.rs`：`AppSettings::default()` 補上新欄位。
- `src-tauri/src/commands/settings.rs`：`save_settings` 增加 autostart 同步副作用。
- `src-tauri/src/lib.rs`：註冊 autostart plugin、於視窗顯示前判斷 `--autostart` 啟動參數及擴充關閉視窗的條件。
- `src-tauri/src/app_setup.rs`：儲存時同步與啟動時的一次性對帳。
- 既有測試 fixture 中所有手寫的 `AppSettings { .. }` literal 需補欄位（`commands/mod.rs`、`quota/{copilot,claude,codex,antigravity}.rs`）。

**前端（TypeScript / React）**
- `src/types/index.ts`、`src/utils/appSettingsDefaults.ts`：型別與預設值同步。
- `src/components/SettingsView.tsx`：新增兩個 checkbox（第二個依第一個啟用狀態決定是否可用）。
- `src/locales/zh-TW.json`、`src/locales/en-US.json`：新增文案鍵值。

**風險**
- 註冊表寫入失敗（權限／防毒攔截）需回報錯誤而非靜默失敗，避免使用者以為已生效。
- 與 `tauri-plugin-single-instance` 併用：自動啟動與使用者手動點擊同時發生時，須確保第二實例的喚醒邏輯不會意外顯示原本應隱藏的視窗。
