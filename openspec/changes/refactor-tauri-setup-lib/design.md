## Context

`pub fn run()`（`lib.rs:281-617`）目前結構：
1. `tauri::Builder::default()` 起手，掛 `single_instance` plugin、`.manage()` 四個 state、四個 plugin
2. `.setup(|app| { ... })` closure（298-525 行），內含：
   - 載入 settings、確保 hook scripts 安裝、重啟 session watcher
   - Quota monitoring 啟動區塊（317-382 行）：讀 DB 快取 + spawn 一次性背景執行緒做首次刷新
   - 背景 quota 輪詢執行緒（384-418 行）：每 60 秒檢查、每 30 分鐘實際刷新一次
   - Tray icon 建構（420-513 行）：建立 icon、選單項目（4 個硬編碼字串）、選單事件處理、tray icon 點擊事件處理
   - 啟動時 tray/overlay 初始化（515-523 行）
3. `.on_window_event(...)`（527-544 行）：tray panel 失焦隱藏、主視窗關閉時最小化到 tray
4. `.invoke_handler(tauri::generate_handler![...])`（545-614 行）：command 註冊列表（本次不動）

已存在 `tray_icon.rs` 模組，但職責是「icon 圖片渲染與 tooltip/百分比計算」（`render_tray_icon_png`、`build_tooltip`、`compute_primary_pct`、`update_tray_from_cache`），與 `lib.rs` 中「tray icon 物件建構、選單綁定、事件處理」是不同職責——後者依賴 `app.handle()` / `TrayIconBuilder` 等 setup 階段才有的物件，不適合硬塞進 `tray_icon.rs`。

**與既有 spec 的關係（重要）**：`openspec/specs/rust-module-structure/spec.md` 已明確規定「`lib.rs` SHALL 僅保留模組宣告與 `pub fn run()`，行數不超過 70 行，不含任何業務邏輯或 struct 定義」。但目前 `lib.rs` 實際 617 行（不含測試），代表此既有需求早已未被滿足——這是先於本次健檢就存在的技術債，不是本次變更造成的。本 change 若把拆出的函式**留在 `lib.rs`**，不會讓現況朝該既有需求靠攏；因此改變原設計，將拆出的函式放到新檔案，使本次重構同時是對既有 `rust-module-structure` 需求的實質貢獻（不修改該 spec 的要求內容，只是這次的實作選擇不再與其牴觸）。`lib.rs` 中其餘同樣不屬於 `run()` 本體、與本 change 無關的業務邏輯（如 `create_quota_overlay`、`toggle_tray_panel` 等 overlay/tray panel 相關函式，第 33-282 行）規模更大，超出本 change 範圍，留給後續獨立 change 處理，本次不觸碰。

## Goals / Non-Goals

**Goals:**
- 把 `setup` closure 內明顯可獨立的三個區塊拆成具名函式：quota monitoring 啟動、背景輪詢執行緒、tray icon 建構
- 抽出 tray 選單的 4 個硬編碼字串為具名常數
- `run()` 函式本體讀起來像一份「啟動步驟清單」，而非一大坨 closure
- 保持所有背景執行緒的排程參數（3 秒延遲、60 秒檢查間隔、30 分鐘刷新間隔）與行為完全不變

**Non-Goals:**
- 不引入後端 i18n 框架（見 proposal.md「Why」的說明——4 個字串、單一語言，框架化是過度設計）
- 不改變 `.invoke_handler` 的 command 註冊方式
- 不改變 `tray_icon.rs` 既有函式的職責邊界
- 不處理 `.setup()` 中設定載入與 hook script 安裝的部分（範圍已明確鎖定在 quota/tray 三塊）
- 不處理 `lib.rs` 中既存、與本次拆分無關的 overlay/tray panel 業務邏輯函式（`create_quota_overlay`、`toggle_tray_panel`、`position_panel_near_tray` 等，第 33-282 行）——這些函式規模較大、涉及視窗生命週期管理，屬於獨立的重構範圍，不在本 change 內處理
- 不承諾本次讓 `lib.rs` 達成既有 spec 規定的「70 行」目標——本 change 只確保新拆出的部分不繼續留在 `lib.rs`，`lib.rs` 整體行數是否達標取決於後續是否有其他 change 處理上述 overlay/tray 業務邏輯

## Decisions

### D1：拆分後的函式搬到新檔案 `src-tauri/src/app_setup.rs`，不留在 `lib.rs`
依上述「與既有 spec 的關係」說明，`lib.rs` 需朝「僅含模組宣告與 `run()`」靠攏，因此拆出的三個函式連同其直接依賴的 helper 一併搬到新模組：
```rust
// src-tauri/src/app_setup.rs
pub(crate) fn setup_quota_monitoring(app: &tauri::App, settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>>
pub(crate) fn spawn_quota_poller_thread(app_handle: tauri::AppHandle)
pub(crate) fn build_tray_icon(app: &tauri::App) -> tauri::Result<()>
```
`lib.rs` 新增 `mod app_setup;` 宣告，`run()` 內改為呼叫 `app_setup::setup_quota_monitoring(app, &settings)?` 等。函式簽章可視編譯器要求的生命週期/錯誤型別調整，但維持「一函式一職責」的拆分邊界與 `pub(crate)` 可見性（符合既有 spec「最小公開原則」需求）。

替代方案（放棄，原設計）：拆分後的函式留在 `lib.rs`。放棄原因：與既有 `rust-module-structure` spec 的「`lib.rs` 不超過 70 行、僅含模組宣告與 `run()`」需求方向相反，即使本次不強求達成 70 行的最終目標（`lib.rs` 中還有更大量不屬於本 change 範圍的 overlay/tray 業務邏輯），至少不應該讓新拆出的程式碼繼續留在 `lib.rs` 加重既有落差。

替代方案（放棄）：把三個區塊搬進既有 `tray_icon.rs`。放棄原因同前——`tray_icon.rs` 職責是「純運算/渲染」，混入 app 生命週期管理會破壞其單一職責；新建 `app_setup.rs` 專門承接 setup 階段邏輯更符合既有模組命名慣例（`platform/`、`provider/` 等皆以職責命名）。

### D2：tray 選單常數集中定義，並註記與前端 i18n 的關係
```rust
// Tray 選單為 Windows 原生元件，無法直接複用前端 t() 翻譯機制。
// 若未來需要多語言 tray menu，需另外設計跨 Rust/前端的語言同步管道
// （例如啟動時由前端透過 command 傳遞當前語言字串給後端）。
const TRAY_MENU_SHOW_WINDOW: &str = "顯示視窗";
const TRAY_MENU_TOGGLE_OVERLAY: &str = "顯示/隱藏 Quota Overlay";
const TRAY_MENU_TOGGLE_OVERLAY_LOCK: &str = "編輯 / 鎖定 Overlay 位置";
const TRAY_MENU_QUIT: &str = "退出 SessionHub";
```
放在 `lib.rs` 頂部（與其他 module-level 常數並列），供 `build_tray_icon()` 使用。

## Risks / Trade-offs

- **[風險] 拆分函式時不慎改變 setup 執行順序，導致 tray icon 在 quota cache 載入完成前就先渲染，顯示錯誤的初始百分比** → 緩解：`run()` 內函式呼叫順序原樣保留（quota monitoring 啟動 → 背景輪詢 → tray icon 建構 → 啟動時初始化），tasks.md 要求核對呼叫順序
- **[風險] 背景執行緒的 `move` 語意在拆成獨立函式後，捕獲的變數所有權轉移方式改變（例如 `app.handle().clone()` 的呼叫時機）** → 緩解：逐一比對每個 `std::thread::spawn` closure 捕獲的變數，確保拆分後語意相同
- **[風險] tray 選單常數化後，若常數名稱與 `on_menu_event` 的 `event.id()` match 字串（`"show_window"` 等 ID，非顯示文字）搞混** → 緩解：常數只對應顯示文字（label），不影響 menu item 的 `id`，tasks.md 明確區分兩者

## Migration Plan

無資料遷移。純後端程式碼重構，一般 PR 流程套用（含 `cargo build` 驗證）。可透過還原單一 commit rollback。
