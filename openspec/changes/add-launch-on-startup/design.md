## Context

SessionHub 定位為常駐背景的 AI CLI session 監控工具：watcher 監看各 provider 資料目錄、hook bridge 接收介入提醒、quota poller 每 30 分鐘輪詢配額。這些能力都要求應用程式處於執行狀態，但目前完全仰賴使用者手動啟動。

**現況相關基礎設施（皆已存在，本次為銜接而非重建）**
- `tauri-plugin-single-instance`（`lib.rs:292`）：第二實例啟動時 `show()` + `unminimize()` + `set_focus()` 既有視窗。
- `tauri-plugin-window-state`：記憶視窗位置尺寸。
- `minimize_to_tray`（`lib.rs:351`）：`CloseRequested` 時 `prevent_close()` + `hide()`。
- `app_setup::build_tray_icon`：系統匣圖示與選單（含「顯示視窗」）。
- `tauri.conf.json` 的 `main` 視窗未設定 `"visible"`，預設為 `true`。

**約束**
- 目標平台為 Windows；`AGENTS.md` 明訂禁止硬編路徑分隔符、禁止 production code 使用 `unwrap()`、所有 command 回傳 `Result<T, String>`。
- 所有 IPC 集中於 `src/App.tsx`；`SettingsView.tsx` 為 props 驅動的純顯示元件。
- 文案一律走 `t("key")`，zh-TW / en-US 兩份 locale 必須同步。

## Goals / Non-Goals

**Goals**
- 使用者可於設定頁單一勾選啟用「開機時自動啟動」，無需重啟即生效。
- 開機自動啟動時預設隱藏主視窗，僅常駐系統匣，讓背景監控能力真正接續開機。
- `settings.json` 為唯一真實來源；能自我修復外部工具（工作管理員）造成的狀態漂移。

**Non-Goals**
- 不支援 macOS / Linux 的實際驗證（程式碼可跨平台，但驗收僅涵蓋 Windows）。
- 不提供系統層級（HKLM，全機所有使用者）自動啟動。
- 不提供延遲啟動秒數、開機後自動執行特定動作等進階排程能力。
- 不改動 `minimize_to_tray` 設定本身的語意；但關閉視窗的判斷條件會增加一個觸發來源（見 D7）。

## Decisions

### D1：使用 `tauri-plugin-autostart`，而非自行寫註冊表

**決定**：後端引入 `tauri-plugin-autostart` 2.5.1，以 `ManagerExt` 提供的 `app.autolaunch()` 取得 `enable()` / `disable()` / `is_enabled()`。目前 lockfile 的 Tauri 為 2.11.5，符合該外掛要求的 Tauri 2.8.2 以上版本。

**理由**：這是 Tauri 官方外掛，正是為此情境設計；封裝了 Windows `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` 的路徑跳脫與引號處理，也涵蓋 macOS LaunchAgent / Linux `.desktop`。自行實作等同重造輪子且要自行處理路徑含空格的引號規則。

**替代方案（已否決）**：以既有的 `windows-sys` 依賴直接寫 `HKCU\...\Run`。專案已依賴 `windows-sys`，可省一個外部依賴；但需自行處理登錄機碼開關、Unicode 字串、路徑引號與錯誤映射，維護成本高於省下的一個依賴。僅在外掛與本專案 Tauri 版本不相容時才回退此方案。

**實作契約**：使用 2.5.1 的 `tauri_plugin_autostart::Builder::new().args(["--autostart"]).build()` 註冊外掛。前端不安裝 JS guest bindings。

### D2：由 Rust 端操作 autostart，前端不直接呼叫外掛 JS API

**決定**：不安裝 `@tauri-apps/plugin-autostart` 前端套件；autostart 的啟用／停用一律在 `save_settings` command 內以 `app.autolaunch()` 完成。

**理由**：專案慣例是「所有 IPC 集中在 `App.tsx`」，而 autostart 狀態本質上是 `AppSettings` 的一部分。若前端另外呼叫外掛 API，等於開出第二條寫入路徑，與 D3 的單一真實來源相衝突，也會讓 `SettingsView` 的表單狀態與實際註冊狀態脫鉤。走 `save_settings` 可沿用既有的「表單 → 儲存 → 副作用」流程，不需新增任何 command。

**替代方案（已否決）**：前端 checkbox 直接呼叫 `enable()`/`disable()`。即時性略好，但引入雙寫入路徑與競態。

### D3：`settings.json` 為單一真實來源，儲存時推送 + 啟動時對帳

**決定**：
1. `save_settings` 在 `save_settings_internal()` 成功後，依 `settings.launch_on_startup` 呼叫 `enable()` 或 `disable()`，與現有 `update_tray_from_cache` / `sync_overlay_visibility` 副作用並列。
2. `setup()` 中新增一次性對帳：設定為停用時僅在 `is_enabled()` 為真時解除註冊；設定為啟用時依第 3 點重新寫入註冊。
3. 設定為啟用時，啟動對帳一律再次呼叫 `enable()`，即使 `is_enabled()` 已為 `true`，使外掛以目前執行檔路徑及 `--autostart` 參數覆寫舊註冊。

**理由**：使用者可從工作管理員「啟動」分頁停用 SessionHub，此時 OS 狀態與 `settings.json` 會漂移。若無對帳，設定頁會長期顯示「已啟用」而實際未生效。以設定為準的單向對帳規則明確且可測試。

**權衡**：對帳等同「使用者從工作管理員的停用會在下次手動啟動時被還原」。這是刻意選擇——設定頁是本應用程式的權威介面；若不接受，替代做法是啟動時反向以 OS 為準覆寫 `settings.json`，但那會讓設定頁的儲存行為變得不可預期。

**錯誤處理**：`save_settings` 的同步失敗回傳 `Err(String)`（使用者主動操作，必須知道失敗）；`setup()` 的對帳失敗僅記錄、不中止啟動（背景行為，不應讓應用程式開不起來）。

**儲存失敗後的狀態**：`save_settings` 先持久化 `settings.json`，再同步作業系統註冊。若同步失敗，已持久化的設定保留為使用者要求的狀態，command 回傳錯誤且前端不得顯示儲存成功；下一次儲存或啟動對帳必須重試同步。此規則維持設定檔作為唯一真實來源，並避免因部分失敗將設定回退為過期值。

### D4：以 `--autostart` 啟動參數識別啟動來源

**決定**：註冊 autostart 時透過外掛的 launch-args 帶入 `--autostart`；`setup()` 中以 `std::env::args()` 檢查此參數。僅當「有 `--autostart`」、「`launch_on_startup` 為 `true`」及「`start_minimized_on_startup` 為 `true`」皆成立時隱藏主視窗。

**理由**：Windows 沒有可靠的「本次是否由登入自動啟動」API，啟動參數是 Tauri 生態的標準做法。以參數而非「首次啟動」等啟發式判斷，語意明確且可手動重現測試（直接以 `--autostart` 執行即可）。

**替代方案（已否決）**：一律隱藏、由使用者從系統匣喚出。會讓手動啟動的行為變得反直覺。

### D5：視窗隱藏時機——`setup()` 內 `hide()`，接受可能的短暫閃現

**決定**：保留 `tauri.conf.json` 中 `main` 視窗預設可見，於 `setup()` 開頭（在 watcher、quota 等較慢的初始化之前）判斷並呼叫 `window.hide()`。

**理由**：改成在 conf 設定 `"visible": false` 會影響所有啟動路徑，屆時每個手動啟動流程都得記得 `show()`，且與 `tauri-plugin-window-state` 的還原時序交互複雜，容易產生「手動啟動卻看不到視窗」的嚴重退化。在 `setup()` 早期 `hide()` 的風險上限只是一瞬間的視窗閃現。

**權衡**：若實測閃現明顯，備案是改為 conf `"visible": false` + 在 `setup()` 末端對非 autostart 啟動明確呼叫 `show()`。此備案列為實作時的觀察項，不預先採用。

### D6：`start_minimized_on_startup` 為獨立欄位，預設 `true`

**決定**：兩個布林欄位而非單一三態列舉；`start_minimized_on_startup` 預設 `true` 且僅在 `launch_on_startup` 為 `true` 時有意義，UI 上以 disabled 表達相依關係。

**理由**：布林對應 checkbox 最直接，也與 `minimize_to_tray`、`show_status_bar` 等既有欄位風格一致。預設 `true` 是因為開機自動啟動的使用者意圖就是背景常駐；每次登入彈出視窗會被視為干擾。

### D7：隱藏啟動時，關閉視窗一律視為隱藏至系統匣

**決定**：`lib.rs` 的 `CloseRequested` 處理改為：`minimize_to_tray || (launch_on_startup && start_minimized_on_startup)` 成立時 `prevent_close()` + `hide()`。

**背景（本設計原本的漏洞）**：現行邏輯僅檢查 `minimize_to_tray`（預設 `false`）。若使用者啟用「開機自動啟動 + 隱藏啟動」但未另外開啟 `minimize_to_tray`，流程會是：開機隱藏常駐（正常）→ 從系統匣喚出視窗 → 按 X 關閉 → **應用程式直接結束**，背景監控中斷至下次開機。這正好抵銷本提案的核心動機（常駐接收 hook 事件）。

**理由**：使用者啟用隱藏啟動的意圖已明確表達為「背景常駐」，關閉視窗理應收合而非結束。以行為隱含相依，比多一個必須手動勾選的設定更符合使用者預期，也不增加設定數量。

**替代方案（已否決）**：
- 儲存時強制 `minimize_to_tray = true`。會靜默改寫使用者的另一個明確設定，且停用自動啟動後無法還原原值。
- UI 上將 `minimize_to_tray` 自動勾選並 disabled。可行但讓兩個概念上獨立的設定在 UI 上綁死，且使用者停用自動啟動後的還原語意不清。

**影響**：`minimize_to_tray` 單獨啟用時的既有行為完全不變；本決定僅為其增加一個額外的觸發條件。

### D8：single-instance 回呼維持現狀

**決定**：不修改 `lib.rs:292` 的 single-instance 回呼——第二實例啟動時仍 `show()` + `set_focus()`。

**理由**：使用者在隱藏常駐狀態下再次點擊捷徑，意圖就是「把視窗叫出來」，現行行為正確。唯一需注意的邊界是自動啟動與手動啟動幾乎同時發生時，後者會把視窗顯示出來——這符合使用者的顯性操作意圖，不需特殊處理。

## Risks / Trade-offs

- **註冊表寫入被防毒／端點防護攔截** → `save_settings` 回傳 `Err` 並由前端 toast 呈現，訊息包含原始錯誤字串，讓使用者知道需加白名單而非誤以為已生效。
- **應用程式路徑變更（升級、搬移安裝目錄）導致舊註冊指向失效路徑** → 每次 `save_settings` 與設定為啟用時的啟動對帳都會重新寫入當前執行檔路徑，確保路徑最新。
- **開機瞬間資料來源尚未就緒**（網路磁碟、使用者 profile 掛載延遲）導致 watcher 註冊失敗 → 屬既有啟動流程風險，不在本次範圍擴大處理；若實測頻繁發生，另開 change 處理啟動重試。
- **主視窗閃現** → 見 D5，已備妥 conf `visible: false` 的回退方案。
- **新增非 `Default` 欄位破壞既有測試 fixture 編譯** → tasks.md 已逐一列出所有手寫 `AppSettings { .. }` literal 的位置。

## Migration Plan

1. 新欄位皆帶 `#[serde(default = "...")]`，舊 `settings.json` 直接相容讀入，無需資料遷移。
2. 功能預設關閉（`launch_on_startup: false`），既有使用者升級後行為完全不變，屬於純加法變更。
3. 首次啟用由使用者在設定頁主動操作，不做任何自動開啟。
4. **回滾**：移除外掛與欄位前，須先呼叫 `disable()` 清除已寫入的 OS 註冊，否則會殘留指向舊版執行檔的登入項目。若直接回滾版本，使用者可於工作管理員「啟動」分頁手動停用。

## Open Questions

無。設定頁顯示作業系統實際註冊狀態不在本次範圍；註冊失敗以儲存錯誤及啟動記錄提供復原資訊。
