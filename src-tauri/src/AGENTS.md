# src-tauri/src/ — Rust 後端

## OVERVIEW

Tauri 2 後端。採模組化架構，`lib.rs` 僅保留 `mod` 宣告與 `pub fn run()`；業務邏輯分散於各子模組。

## MODULE STRUCTURE（依職責分層）

```
src-tauri/src/
├── lib.rs              ← 只有 mod 宣告 + pub fn run()（< 70 行）
├── main.rs             ← 進入點，僅呼叫 run()；禁止加業務邏輯
├── types.rs            ← 所有 pub struct/enum/const（無邏輯）
├── db.rs               ← SQLite 連線、init_db、migrate、CRUD helpers
├── settings.rs         ← load/save settings、detect_terminal/vscode、路徑解析
├── watcher.rs          ← WatcherState + 所有 create_*_watcher、防抖快照邏輯
├── stats.rs            ← parse_session_stats_internal、calculate_opencode_session_stats
├── activity.rs         ← get_copilot/opencode_activity_status
├── plan.rs             ← read/write/watch plan file helpers、read_openspec_file_internal
├── sisyphus.rs         ← scan_sisyphus_internal（Sisyphus plan 掃描）
├── openspec_scan.rs    ← scan_openspec_internal（OpenSpec change 掃描）
├── sessions/
│   ├── mod.rs          ← get_sessions_internal、persist_provider_cache
│   ├── copilot.rs      ← scan_copilot_incremental_internal、parse_workspace_file
│   └── opencode.rs     ← scan_opencode_incremental_internal、OpencodeSession* structs
├── provider/
│   ├── mod.rs          ← 共用 helpers、路徑解析、matched_bridge_providers
│   ├── bridge.rs       ← process_provider_bridge_event、read_last_bridge_record
│   ├── copilot.rs      ← detect/install copilot integration
│   └── opencode.rs     ← detect/install opencode integration
├── commands/
│   ├── mod.rs          ← pub use 所有子模組（re-export）
│   ├── sessions.rs     ← get_sessions、archive/delete session、stats commands
│   ├── settings.rs     ← get_settings、save_settings、detect_* commands
│   ├── plan.rs         ← read_plan、write_plan、open_plan_external、watch_plan_file
│   ├── tools.rs        ← open_terminal、open_in_tool、focus_terminal_window、check_tool_availability
│   └── provider.rs     ← install/update/recheck provider integration
└── platform/
    ├── mod.rs
    └── win32_focus.rs  ← WIN32 FFI：EnumWindows、SetForegroundWindow
```

## 依賴方向（單向，禁止循環）

```
commands/* → sessions/ → db, settings, stats, types
commands/* → provider/ → settings, types
commands/* → watcher  → sessions/, provider/, types
commands/* → plan, activity, sisyphus, openspec_scan → types
db, settings → types
```

## WHERE TO LOOK

| 任務 | 位置 |
|------|------|
| 新增 Tauri command | `commands/<group>.rs` + 在 `commands/mod.rs` pub use + 在 `lib.rs` invoke_handler 登記 |
| 新增 struct/enum | `types.rs`（或放在最相關的模組內用 `pub(crate)`） |
| 修改 Session 掃描邏輯 | `sessions/copilot.rs` 或 `sessions/opencode.rs` |
| 修改 Provider 整合 | `provider/copilot.rs` 或 `provider/opencode.rs` |
| 修改 SQLite schema | `db.rs` → `init_db()` |
| 修改 FS Watcher 邏輯 | `watcher.rs` |
| 新增設定欄位 | `types.rs`（AppSettings struct）+ `settings.rs` 邏輯 + 前端 `src/types/index.ts` |

## COMMAND PATTERN（必須遵守）

```rust
// commands/<group>.rs

// 1. 公開 command（薄包裝，放在 commands/<group>.rs）
#[tauri::command]
pub fn my_command(arg: String) -> Result<T, String> {
    my_command_internal(&arg)
}

// 2. _internal fn（含邏輯，放在最相關的模組；pub(crate) 供測試存取）
pub(crate) fn my_command_internal(arg: &str) -> Result<T, String> {
    // 實際邏輯在此
}

// 3. 在 commands/mod.rs 加 pub use
// 4. 在 lib.rs invoke_handler![] 登記
```

## ALL REGISTERED COMMANDS

**sessions：** `get_sessions`, `archive_session`, `unarchive_session`, `delete_session`, `delete_empty_sessions`, `get_session_stats`, `upsert_session_meta`, `delete_session_meta`, `get_session_activity_statuses`, `get_project_plans`, `get_project_specs`, `read_plan_content`, `check_directory_exists`

**settings：** `get_settings`, `save_settings`, `detect_terminal`, `detect_vscode`, `validate_terminal_path`

**plan：** `read_plan`, `write_plan`, `open_plan_external`, `watch_plan_file`, `stop_plan_watch`, `read_openspec_file`

**tools：** `open_terminal`, `open_in_tool`, `focus_terminal_window`, `check_tool_availability`, `restart_session_watcher`

**provider：** `install_provider_integration`, `update_provider_integration`, `recheck_provider_integration`

## KEY TYPES（定義於 types.rs）

| Type | Serde | 用途 |
|------|-------|------|
| `SessionInfo` | `camelCase` | 回傳給前端的 session 資訊 |
| `AppSettings` | `camelCase` | settings.json 序列化 |
| `SessionStats` | `camelCase` | session 統計資料 |
| `SessionActivityStatus` | `camelCase` | session 活動狀態 |
| `WatcherState` | — | Mutex 包裝四個 watcher（sessions/plan/opencode/provider bridge） |
| `ScanCache` | — | 兩個 provider 的記憶體掃描快取 |
| `ProviderIntegrationStatus` | `camelCase` | Provider 整合狀態 |
| `ToolAvailability` | `camelCase` | CLI 工具偵測結果 |

## VISIBILITY 規則

| 情境 | 用法 |
|------|------|
| `#[tauri::command]` 函式 | `pub fn` |
| 序列化 struct（前端可見） | `pub struct` |
| 跨模組 helper/struct（僅 crate 內） | `pub(crate) fn` / `pub(crate) struct` |
| 純內部 helper（單一模組） | `fn`（私有） |
| lib.rs 的 re-export | `pub(crate) use module::*;` |

## DATA PATHS

- Settings：`%APPDATA%\SessionHub\settings.json`
- SQLite：`%APPDATA%\SessionHub\metadata.db`
- Copilot Sessions：`{copilotRoot}\session-state\<id>\workspace.yaml`
- Copilot Archive：`{copilotRoot}\session-state-archive\`
- OpenCode Sessions：`{opencodeRoot}\session\<projectId>\ses_*.json`
- Plan：`{sessionDir}\plan.md`

## ANTI-PATTERNS（此專案禁止）

- `unwrap()` 在 production code — 必須 `.map_err(|e| format!(...))`
- Hardcode 路徑分隔符 `\` — 用 `PathBuf::join()`
- `#[tauri::command]` 內直接寫業務邏輯 — 應呼叫 `_internal` fn
- `main.rs` 內加業務邏輯 — 只能呼叫 `run()`
- 在 `commands/` 以外的模組定義 `#[tauri::command]`
- 將業務邏輯放入 `lib.rs`（lib.rs 只能有 mod 宣告與 run()）
- 跨模組循環依賴（例如 sessions 依賴 commands）

## TESTING

- 測試用環境變數：`COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE` 覆蓋 AppData 路徑
- 每個測試用 `unique_test_dir(name)` 建立隔離臨時目錄，測試後自行清理
- 全域 `test_lock()` 防止並行測試競爭
- `mod tests` 集中在 `lib.rs` 底部，透過 `use super::*;` 存取所有 `pub(crate)` 符號
- 執行：`cd src-tauri && cargo test`
