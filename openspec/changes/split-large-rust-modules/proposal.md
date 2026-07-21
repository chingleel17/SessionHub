## Why

`src-tauri/src/` 有三個檔案明顯偏大：`agents_config.rs`（2730 行，其中約 1847～2730 行、逾六成是測試）、`stats.rs`（1459 行，內含 generic/OpenCode/Claude 三種 provider 的 stats 計算邏輯混在同一檔）、`types.rs`（1029 行，約 65 個型別定義集中一檔、無分區）。這些檔案本身邏輯正確、無已知 bug，純粹是「單檔案過長」影響可讀性與導覽效率。這是本次健檢中優先度最低的一項，僅在有餘力時處理。

## What Changes

- `agents_config.rs`：將 `#[cfg(test)] mod tests { ... }`（約佔全檔 68%）搬到獨立測試檔案（Rust 慣例：同目錄下 `agents_config/tests.rs` 並以 `#[path]` 引入，或搬至 `tests/` 整合測試視性質而定，實作時依測試是否需要 `pub(crate)` 內部函式存取決定放法），主檔只保留生產程式碼（預期降至約 880 行）
- `stats.rs`：依 provider 拆成 `stats/mod.rs`（共用函式：`get_session_stats_cache`、`upsert_session_stats_cache` 等）、`stats/opencode.rs`、`stats/claude.rs`，維持 `crate::stats::` 既有 public 介面不變（外部呼叫方不需改 import）
- `types.rs`：依領域拆成子模組（如 `types/session.rs`、`types/settings.rs`、`types/quota.rs`、`types/provider_integration.rs`、`types/analytics.rs`、`types/opencode.rs`、`types/claude.rs`），`types/mod.rs` 用 `pub use` 統一 re-export，維持 `crate::types::Xxx` 既有 import 路徑完全不變
- 不改變任何型別欄位、函式簽章、序列化格式、測試內容或測試結果

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

（無 — 純內部檔案組織重構，`rust-module-structure` capability 描述的是「新增 provider 時的模組慣例」，本次不改變該慣例本身，只是把既有超大檔案依同樣的分模組精神拆開；不影響任何依賴 `types.rs` / `stats.rs` / `agents_config.rs` 現有 public 介面的呼叫方。）

## Impact

- 受影響程式碼：
  - `src-tauri/src/agents_config.rs` → 拆出測試部分
  - `src-tauri/src/stats.rs` → 改為 `src-tauri/src/stats/` 目錄（`mod.rs` + `opencode.rs` + `claude.rs`）
  - `src-tauri/src/types.rs` → 改為 `src-tauri/src/types/` 目錄（`mod.rs` + 領域子模組）
  - `src-tauri/src/lib.rs` 的 `mod types;` / `mod stats;` 宣告方式可能需視改為目錄模組後的慣例調整（Rust 目錄模組宣告方式不變，僅檔案位置改變）
- 不影響任何 Tauri command 簽章、前端程式碼、資料庫結構、既有測試案例的斷言內容
- **本次為選擇性重構，優先度最低**：可視團隊時間安排延後或跳過，不阻塞其他四個 change 的套用
- 風險考量：`types.rs` 的 65 個型別分類邊界具主觀性，且 Rust 的 `pub use` re-export 若疏漏會造成編譯錯誤（非執行期風險，會在 `cargo check` 階段立即發現）
