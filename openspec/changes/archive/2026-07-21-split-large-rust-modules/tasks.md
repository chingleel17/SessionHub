## 1. agents_config.rs — 搬移測試

- [x] 1.1 建立 `src-tauri/src/agents_config/tests.rs`，搬入原 `agents_config.rs` 第 1847 行起的 `#[cfg(test)] mod tests { ... }` 內容（不含外層 `#[cfg(test)] mod tests { }` 包裝，內容直接放在新檔案頂層）
- [x] 1.2 在 `agents_config.rs` 底部改為 `#[cfg(test)] #[path = "agents_config/tests.rs"] mod tests;`（依 design.md D1）
- [x] 1.3 執行 `cargo check` 與 `cargo test agents_config` 確認編譯與測試皆通過，測試數量與重構前一致

## 2. stats.rs — 依 provider 拆分

- [x] 2.1 建立 `src-tauri/src/stats/` 目錄，將原 `stats.rs` 改名/搬移為 `stats/mod.rs`
- [x] 2.2 建立 `stats/opencode.rs`，搬入 OpenCode 專屬函式（`calculate_opencode_session_stats`、`get_opencode_session_stats_internal`、`parse_opencode_message_json`、`open_opencode_db_for_stats`、`scan_opencode_messages_for_session`、`scan_opencode_parts_for_message` 等）
- [x] 2.3 建立 `stats/claude.rs`，搬入 Claude 專屬函式（`compute_claude_stats`、`build_claude_usage_blocks`、`claude_model_pricing`、`extract_tool_names_from_content` 等）
- [x] 2.4 `stats/mod.rs` 保留共用函式，並加上 `pub(crate) use opencode::*;` / `pub(crate) use claude::*;`（或視實際需要調整 re-export 範圍）確保 `crate::stats::` 既有呼叫路徑不變
- [x] 2.5 依 D1 同樣手法，將測試搬到 `stats/tests.rs`
- [x] 2.6 執行 `cargo check` 與 `cargo test stats` 確認編譯與測試皆通過

## 3. types.rs — 依領域拆分

- [x] 3.1 建立 `src-tauri/src/types/` 目錄與 `mod.rs`
- [x] 3.2 依 design.md D3 分類，逐一建立 `session.rs`、`settings.rs`、`quota.rs`、`provider_integration.rs`、`analytics.rs`、`opencode.rs`、`sisyphus_openspec.rs`、`claude.rs`、`misc.rs`，搬入對應型別定義與其專屬 `default_xxx()` helper
- [x] 3.3 `mod.rs` 以 `pub use` 統一 re-export 全部子模組內容，確保 `crate::types::Xxx` 路徑全專案不需修改
- [x] 3.4 檢查子模組間欄位型別依賴（如 `AppSettings` 用到 `TrayQuotaMode`），一律透過 `crate::types::Xxx` 引用而非子模組互相直接 `use super::`
- [x] 3.5 執行 `cargo check` 確認全專案（含所有 `crate::types::` 呼叫方）編譯成功，無需修改任何呼叫方程式碼
- [x] 3.6 執行 `cargo test` 全量測試確認無回歸

## 4. 收尾驗證

- [x] 4.1 執行 `cargo clippy` 確認無新增警告（62 個既有警告皆位於本次未變動檔案，或為原封不動搬移過去的既有程式碼所觸發，無因拆分產生的新結構性警告）
- [x] 4.2 執行前端 `tsc --noEmit`（無錯誤輸出，通過）
- [x] 4.3 確認三個檔案拆分後單檔行數：`stats/*.rs` 與 `types/*.rs` 皆已降至 <800 行目標規模；`agents_config.rs` 主檔仍為 1849 行——因其生產程式碼本身（約 61 個函式、1846 行）為單一完整邏輯區塊（agents.md/skills/commands 掃描同步），design.md D1 範圍僅規劃搬移測試，未規劃進一步拆分生產程式碼，依 Non-Goals 予以保留
