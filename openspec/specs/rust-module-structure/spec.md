## ADDED Requirements

### Requirement: 模組目錄結構符合職責邊界

Rust 後端 SHALL 將所有程式碼依職責邊界拆分為獨立模組檔案，`lib.rs` 僅保留模組宣告與 `pub fn run()`，行數不超過 70 行。

#### Scenario: lib.rs 僅包含 mod 宣告與 run()
- **WHEN** 開發者開啟 `src-tauri/src/lib.rs`
- **THEN** 檔案僅含 `mod` 宣告、`use` re-export 與 `pub fn run()`，不含任何業務邏輯或 struct 定義

#### Scenario: 各模組檔案存在於正確路徑
- **WHEN** 建置系統執行 `cargo build`
- **THEN** 以下模組路徑均存在且可正常引入：`types`, `db`, `settings`, `watcher`, `stats`, `activity`, `plan`, `sisyphus`, `openspec_scan`, `sessions/mod`, `sessions/copilot`, `sessions/opencode`, `provider/mod`, `provider/bridge`, `provider/copilot`, `provider/opencode`, `commands/mod`, `commands/sessions`, `commands/settings`, `commands/plan`, `commands/tools`, `commands/provider`, `platform/mod`, `platform/win32_focus`

### Requirement: Tauri command 公開簽名保持不變

所有 26 個 `#[tauri::command]` 函式的名稱與參數型別 SHALL 保持不變，前端 `invoke()` 呼叫無需修改。

#### Scenario: 前端 invoke 呼叫正常運作
- **WHEN** 前端呼叫任意 `invoke("command_name", args)` 且 command 已搬移至 `commands/` 子模組
- **THEN** Tauri invoke handler 正確路由至對應函式，回傳結果與重構前相同

#### Scenario: invoke_handler 包含所有 commands
- **WHEN** 執行 `cargo build`
- **THEN** `run()` 中的 `invoke_handler![]` 巨集包含所有原有 command 函式，無遺漏

### Requirement: 跨模組 visibility 使用最小公開原則

內部 helper 函式 SHALL 使用 `pub(crate)` 而非 `pub`；只有 `#[tauri::command]` 函式與序列化型別 SHALL 使用 `pub`。

#### Scenario: 內部函式不對外暴露
- **WHEN** 外部 crate 嘗試直接呼叫 `_internal` 結尾的 helper 函式
- **THEN** 編譯器回報 visibility 錯誤，函式無法被外部存取

#### Scenario: Tauri command 函式可被 invoke_handler 引用
- **WHEN** `commands/mod.rs` 使用 `pub use` re-export 所有 command 函式
- **THEN** `lib.rs` 中的 `invoke_handler![]` 可透過 `commands::*` 引用所有 command

### Requirement: 所有現有測試通過

重構後 `cargo test` SHALL 輸出全數通過，無新增測試失敗。

#### Scenario: 既有單元測試不受模組搬移影響
- **WHEN** 執行 `cd src-tauri && cargo test`
- **THEN** 所有測試通過，exit code 為 0，無 compilation error 或 test failure

#### Scenario: 測試模組可存取被測函式
- **WHEN** `mod tests` 區塊引用已搬移至子模組的 `_internal` 函式
- **THEN** 透過 `use crate::<module>::<fn>` 或 `super::` 正確引用，測試正常編譯

### Requirement: 增量拆分每步可獨立編譯

每個模組拆分步驟 SHALL 獨立形成可編譯狀態，不允許中間狀態存在編譯錯誤。

#### Scenario: 每步 cargo build 成功
- **WHEN** 完成任意單一模組的搬移後執行 `cargo build`
- **THEN** 編譯成功，無 error（warning 可接受）

#### Scenario: 依賴圖為有向無環圖
- **WHEN** 各模組透過 `use crate::` 引用其他模組
- **THEN** 不存在 A 依賴 B 且 B 依賴 A 的循環依賴情況
