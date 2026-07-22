## 本次變更無需求層級（spec）異動

本 change（`split-large-rust-modules`）為純內部檔案組織重構：

- `agents_config.rs` 的測試搬到獨立測試檔案
- `stats.rs` 依 provider（generic / OpenCode / Claude）拆為目錄模組
- `types.rs` 依領域拆為目錄模組，並透過 `pub use` 維持既有 import 路徑

不新增、不修改、不移除任何既有 capability 的行為需求，不改變任何型別欄位、函式簽章或測試斷言內容。`rust-module-structure` 既有 spec 描述的模組慣例（新增 provider 時各層各加一個檔案）維持不變，本次僅將既有超大檔案依同樣精神拆開。因此本目錄不含任何 `ADDED / MODIFIED / REMOVED Requirements` delta 檔案。

驗證方式見 `tasks.md`：每一步拆分後皆需通過 `cargo check` + `cargo test`。
