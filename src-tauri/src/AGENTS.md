# src-tauri/src/ — Rust 後端

## OVERVIEW
Tauri 2 後端。單一大檔 `lib.rs`（802 行），包含所有 commands、業務邏輯、SQLite 操作與 FS watcher。

## FILES

| 檔案 | 用途 |
|------|------|
| `lib.rs` | 所有後端邏輯（structs、commands、helpers、tests） |
| `main.rs` | 進入點，僅呼叫 `session_hub_lib::run()`；禁止加業務邏輯 |

## lib.rs 結構（依行號）

| 範圍 | 內容 |
|------|------|
| 1–65 | use 引用、常數（CREATE_NEW_CONSOLE）、核心 struct 定義 |
| 66–130 | 路徑 helpers（default_copilot_root、settings_file_path、metadata_db_path） |
| 131–250 | FS watcher 管理（create_sessions_watcher、restart_session_watcher_internal） |
| ~250–450 | Session 掃描邏輯（scan_sessions、archive_session、delete_session） |
| ~450–600 | SQLite 操作（open_db_connection、upsert_session_meta、delete_session_meta） |
| ~600–695 | Plan 相關（read_plan、write_plan、open_plan_external） |
| 696–730 | `run()` — Tauri builder，登記所有 command |
| 731–802 | `#[cfg(test)]` 單元測試 |

## COMMAND PATTERN（必須遵守）

```rust
// 1. 公開 command（薄包裝）
#[tauri::command]
fn my_command(arg: String) -> Result<T, String> {
    my_command_internal(&arg)
}

// 2. _internal fn（含邏輯，可直接在 tests 呼叫）
fn my_command_internal(arg: &str) -> Result<T, String> {
    // 實際邏輯在此
}

// 3. 在 invoke_handler![] 登記（lib.rs:708）
```

## ALL REGISTERED COMMANDS

`get_sessions`, `get_settings`, `save_settings`, `detect_terminal`, `detect_vscode`,  
`restart_session_watcher`, `watch_plan_file`, `stop_plan_watch`, `validate_terminal_path`,  
`archive_session`, `delete_session`, `open_terminal`, `check_directory_exists`,  
`read_plan`, `write_plan`, `open_plan_external`, `upsert_session_meta`, `delete_session_meta`

## KEY STRUCTS

| Struct | Serde | 用途 |
|--------|-------|------|
| `SessionInfo` | `camelCase` | 回傳給前端的 session 資訊 |
| `AppSettings` | `camelCase` | settings.json 序列化 |
| `WorkspaceYaml` | 預設 | 解析 workspace.yaml |
| `WatcherState` | — | Mutex 包裝兩個 watcher（sessions + plan） |

## DATA PATHS

- Settings：`%APPDATA%\SessionHub\settings.json`
- SQLite：`%APPDATA%\SessionHub\metadata.db`
- Sessions：`{copilotRoot}\session-state\<id>\workspace.yaml`
- Archive：`{copilotRoot}\session-state-archive\`
- Plan：`{sessionDir}\plan.md`

## ANTI-PATTERNS

- `unwrap()` 在 production code — 必須 `.map_err(|e| format!(...))`
- Hardcode 路徑分隔符 `\` — 用 `PathBuf::join()`
- `#[tauri::command]` 內直接寫業務邏輯 — 應呼叫 `_internal` fn
- `main.rs` 內加業務邏輯 — 只能呼叫 `run()`

## TESTING

- 測試用環境變數：`COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE` 覆蓋 AppData 路徑
- 每個測試用 `unique_test_dir(name)` 建立隔離臨時目錄，測試後自行清理
- 全域 `test_lock()` 防止並行測試競爭
