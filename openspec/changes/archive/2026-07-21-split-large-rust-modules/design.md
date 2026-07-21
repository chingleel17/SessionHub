## Context

三個檔案的現況（已讀取原始碼確認）：

**`agents_config.rs`**（2730 行）：
- 第 1-1846 行：生產程式碼（agents.md/skills/commands 掃描與同步邏輯，含 `scan_agents_md_internal`、`sync_agents_items_internal`、`link_agents_root_internal` 等約 50 個函式）
- 第 1847-2730 行：`#[cfg(test)] mod tests { ... }`，佔全檔 68%

**`stats.rs`**（1459 行）：
- 第 1-370 行：generic session stats（`get_session_stats_cache`、`upsert_session_stats_cache`、`is_live_session`、`parse_session_stats_internal` 等，Copilot/Claude/OpenCode 共用的快取與通用解析）
- 第 370-694 行：OpenCode 專屬（`calculate_opencode_session_stats`、`get_opencode_session_stats_internal`、DB 讀取等）
- 第 696-1163 行：Claude 專屬（`compute_claude_stats`、`build_claude_usage_blocks`、pricing 表等）
- 第 1163-1327 行：泛用入口（`get_session_stats_internal`、`backfill_missing_stats_internal`）
- 第 1327+ 行：測試

**`types.rs`**（1029 行）：約 65 個 struct/enum，無分區註解，依語意可歸類為：session／settings／quota／provider bridge & integration／analytics events／opencode json／sisyphus & openspec／claude／misc（如 `ToolAvailability`、`WatcherState`、`ScanCache`）

## Goals / Non-Goals

**Goals:**
- 降低三個檔案的單檔行數至可一次性閱讀的規模（目標各 <800 行）
- 保持所有既有 public/`pub(crate)` 介面路徑不變（`crate::types::SessionInfo`、`crate::stats::get_session_stats_internal` 等呼叫方不需修改）
- 保持所有既有測試案例不變（只搬移位置，不改斷言內容）

**Non-Goals:**
- 不重新設計型別欄位或函式簽章
- 不合併/拆分既有函式的邏輯邊界
- 不強制三個檔案都要拆——如果實作過程中發現某個檔案拆分後可讀性沒有明顯提升（例如型別彼此耦合度太高難以乾淨分區），允許該檔案的拆分任務標記為跳過並在 PR 說明原因
- 不處理其他大檔案（如 `db.rs` 834 行、`watcher.rs` 751 行）——本次僅鎖定健檢報告中列出的三個最大檔案

## Decisions

### D1：`agents_config.rs` 測試搬移方式——同目錄 `#[path]` 引入，而非 `tests/` 整合測試
測試大量依賴 `pub(crate)` 甚至 private 函式（如 `scan_agents_md_root`、`fingerprint_file`），若搬到 `tests/` 整合測試目錄會失去存取權限，需要把大量函式改為 `pub(crate)` 甚至 `pub`，這改變了現有的可見性設計，不符合 Non-Goals。因此採用同目錄單元測試慣例：
```rust
// agents_config.rs 底部
#[cfg(test)]
#[path = "agents_config/tests.rs"]
mod tests;
```
新增 `src-tauri/src/agents_config/tests.rs` 存放原本的測試內容。

替代方案（放棄）：搬到 `tests/agents_config_test.rs` 整合測試。放棄原因：需大量放寬可見性，與「不改變既有介面/可見性」的 Non-Goals 衝突。

### D2：`stats.rs` 拆為目錄模組，維持 `pub(crate) fn` 簽章原樣
```
src-tauri/src/stats/
  mod.rs       — 共用：get_session_stats_cache / upsert_session_stats_cache /
                 session_events_mtime / session_id_from_dir / is_live_session /
                 parse_session_stats_internal / get_session_stats_internal /
                 backfill_missing_stats_internal，並 `pub(crate) use opencode::*;`
                 `pub(crate) use claude::*;` re-export 讓外部呼叫方路徑不變
  opencode.rs  — OpenCode 專屬函式
  claude.rs    — Claude 專屬函式（含 pricing 表）
  tests.rs     — 原有測試（依 D1 同樣手法，用 #[path] 引入）
```
`lib.rs` 的 `mod stats;` 宣告不需改動（Rust 會自動找 `stats/mod.rs`）。

### D3：`types.rs` 拆為目錄模組，`mod.rs` 全部 `pub use` re-export
```
src-tauri/src/types/
  mod.rs                    — pub use session::*; pub use settings::*; ... (依領域全部 re-export)
  session.rs                — SessionInfo, SessionTodo, SessionActivityStatus, SessionMeta, SessionEvent, ...
  settings.rs                — AppSettings, TrayQuotaMode, OverlayTheme, OverlayStyle
  quota.rs                   — QuotaWindow, QuotaSnapshot, QuotaCache, LocalTokenUsage, ExtraCredits, ResetCredits, ...
  provider_integration.rs    — ProviderIntegrationState/Status, ProviderBridgeRecord, CopilotIntegrationConfig, ...
  analytics.rs                — AnalyticsDataPoint, SessionStartData, ToolExecutionStartData, ... (bridge 事件 payload 群)
  opencode.rs                 — OpencodeProjectJson, OpencodeSessionJson, OpencodeMessage, ...
  sisyphus_openspec.rs        — SisyphusBoulder/Plan/Notepad/Data, OpenSpecTaskProgress/Change/Spec/Data
  claude.rs                   — ClaudeEntry, ClaudeMessage, ClaudeUsage, ClaudeUsageBlock, ...
  misc.rs                     — ToolAvailability, WatcherState, ScanCache, ProviderCache（不易歸類的雜項）
```
所有 `default_xxx()` helper 函式（`default_provider()`、`default_true()` 等）隨其主要使用的型別歸入對應子模組，若被多個子模組共用則留在 `mod.rs`。

替代方案（放棄）：只拆最大的幾個 struct，其餘留在 `mod.rs`。放棄原因：部分拆分無法達成「單檔 <800 行」的目標，且會產生「為什麼這個型別在這裡、那個型別在別處」的不一致觀感。

## Risks / Trade-offs

- **[風險] `types.rs` 型別間彼此有欄位型別依賴（例如 `AppSettings` 用到 `TrayQuotaMode`），拆分後子模組間需要互相 `use super::xxx::Yyy`，可能產生循環引用或大量樣板 `use`** → 緩解：`mod.rs` 統一 `pub use` 後，子模組內部互相引用一律走 `use crate::types::Xxx`（透過 re-export 後的路徑），避免子模組互相直接 `use super::other_module::Xxx` 造成耦合混亂
- **[風險] 三個檔案分別由不同人／不同 PR 處理時，若同時進行可能互相衝突（尤其 `types.rs` 幾乎被全專案引用）** → 緩解：tasks.md 按檔案分組，建議依序（先 `agents_config.rs` 測試搬移風險最低 → 再 `stats.rs` → 最後 `types.rs`）不要並行
- **[風險] 這是本次五個 change 中風險最高、報酬相對最低的一項（不影響任何功能行為，純可讀性）** → 緩解：proposal.md 已明確標註「優先度最低、可延後或跳過」，執行前應重新確認團隊是否仍要投入

## Migration Plan

無資料遷移。純後端檔案組織重構。建議分三次獨立 commit（依 D1/D2/D3 順序），每次 commit 後都需通過 `cargo check` + `cargo test` 再進行下一步，任一步驟卡住可獨立回退而不影響已完成的部分。
