## Why

`src-tauri/src/lib.rs` 的 `pub fn run()`（第 281-617 行）把 quota 監控啟動、背景輪詢執行緒、系統匣圖示與選單建立、視窗事件處理全部寫在同一個 `setup` closure 與函式本體中，職責混雜、難以獨立閱讀或測試其中一塊邏輯。此外，系統匣選單的 4 個文字標籤（「顯示視窗」「顯示/隱藏 Quota Overlay」「編輯 / 鎖定 Overlay 位置」「退出 SessionHub」）直接硬編碼在 `MenuItemBuilder::new(...)` 呼叫中，且與前端 `src/locales/zh-TW.ts` 已存在的 `quota.tray.quit` 等文案語意重複但未共用，屬於分散定義的壞味道。

- 將 `pub fn run()` 內的 setup 邏輯拆成數個具名函式（如 `setup_quota_monitoring()`、`spawn_quota_poller_thread()`、`build_tray_icon()`），搬到新檔案 `src-tauri/src/app_setup.rs`，`run()` 本體改為依序呼叫這些函式，職責邊界清楚
- 將 tray 選單的 4 個硬編碼字串抽成具名常數，消除「字串字面值散落在建構呼叫中」的問題
- 不引入完整後端 i18n 框架（系統匣選單為 OS 原生元件，且目前應用僅支援繁體中文一種語言，引入多語言框架屬過度設計）；僅在常數定義處加註解說明「若未來需要多語言 tray menu，需另外設計跨 Rust/前端的語言同步機制」
- 不改變任何背景執行緒的排程邏輯、quota 監控行為、tray 選單的功能或外觀

**與既有 spec 的關係**：`openspec/specs/rust-module-structure/spec.md` 已規定 `lib.rs` SHALL 僅保留模組宣告與 `run()`、不超過 70 行，但目前 `lib.rs` 實際 617 行（不含測試）——此為早於本次健檢即存在的技術債。本 change 把拆出的函式放到新檔案而非留在 `lib.rs`，是朝該既有需求前進的實作選擇，但不足以讓 `lib.rs` 整體達成 70 行目標（`lib.rs` 中仍有本 change 範圍外的 overlay/tray 業務邏輯函式，規模更大，留待後續獨立 change 處理）。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

（無 — 純內部重構，不改變 `tray-quota-widget`、`provider-quota-monitor`、`single-instance-lock` 等既有 capability 的行為需求。）

## Impact

- 受影響程式碼：
  - `src-tauri/src/lib.rs`（移除已拆出的邏輯，新增 `mod app_setup;`，`run()` 改為呼叫新模組函式）
  - 新增 `src-tauri/src/app_setup.rs`
- 不影響任何 Tauri command 簽章、前端程式碼、資料庫結構
- 與其他 change（`cleanup-deps-and-settings-defaults`、`extract-app-tsx-event-hooks`、`extract-embedded-apps-and-settings-hook`）互不重疊，可獨立套用，無先後依賴
- 不完全解決 `lib.rs` 與 `rust-module-structure` spec 的落差（見上方說明），該落差的完整解決需要額外一個 change 處理 overlay/tray 業務邏輯搬移
