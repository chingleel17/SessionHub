## Why

`src-tauri/src/lib.rs` 已成長至 7263 行，包含型別定義、SQLite 操作、FS watcher、Session 掃描、Provider 整合、統計計算、Tauri commands 等全部職責在單一檔案中。這嚴重影響程式碼可讀性、可維護性與可測試性，且隨功能持續新增只會更糟。

## What Changes

- 將 `lib.rs` 拆分為多個依職責劃分的子模組（`types`, `db`, `settings`, `watcher`, `sessions/*`, `provider/*`, `stats`, `activity`, `plan`, `platform/*`, `commands/*`）
- `lib.rs` 僅保留 `mod` 宣告與 `pub fn run()` 入口（目標 ~60 行）
- 所有現有 `#[tauri::command]` 保持公開簽名不變，前端 `invoke()` 呼叫無需修改
- 各模組透過 `pub use` re-export，不改變對外 API 介面
- 保留所有現有測試（`mod tests`）並確保 `cargo test` 通過

## Capabilities

### New Capabilities

- `rust-module-structure`: Rust 後端模組化架構——定義各子模組的職責邊界、檔案佈局、模組宣告方式與跨模組 visibility 規則

### Modified Capabilities

## Impact

- **修改檔案**：`src-tauri/src/lib.rs`（大幅縮減）
- **新增檔案**：`src-tauri/src/types.rs`、`src-tauri/src/db.rs`、`src-tauri/src/settings.rs`、`src-tauri/src/watcher.rs`、`src-tauri/src/stats.rs`、`src-tauri/src/activity.rs`、`src-tauri/src/plan.rs`、`src-tauri/src/sessions/mod.rs`、`src-tauri/src/sessions/copilot.rs`、`src-tauri/src/sessions/opencode.rs`、`src-tauri/src/provider/mod.rs`、`src-tauri/src/provider/bridge.rs`、`src-tauri/src/provider/copilot.rs`、`src-tauri/src/provider/opencode.rs`、`src-tauri/src/commands/mod.rs`、`src-tauri/src/commands/sessions.rs`、`src-tauri/src/commands/settings.rs`、`src-tauri/src/commands/plan.rs`、`src-tauri/src/commands/tools.rs`、`src-tauri/src/commands/provider.rs`、`src-tauri/src/platform/mod.rs`、`src-tauri/src/platform/win32_focus.rs`
- **無破壞性變更**：前端 `src/` 代碼完全不受影響；Tauri command 簽名保持不變
- **依賴無變化**：`Cargo.toml` 無需修改
