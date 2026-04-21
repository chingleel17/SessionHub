## 1. 建立模組目錄結構

- [x] 1.1 建立 `src-tauri/src/sessions/` 目錄（含空白 `mod.rs`）
- [x] 1.2 建立 `src-tauri/src/provider/` 目錄（含空白 `mod.rs`）
- [x] 1.3 建立 `src-tauri/src/commands/` 目錄（含空白 `mod.rs`）
- [x] 1.4 建立 `src-tauri/src/platform/` 目錄（含空白 `mod.rs`）

## 2. 拆分 types.rs（零依賴基礎層）

- [x] 2.1 建立 `src-tauri/src/types.rs`，搬移所有 struct/enum 定義（`WorkspaceYaml`, `SessionInfo`, `AppSettings`, `SessionActivityStatus`, `ProviderIntegrationStatus`, `ProviderBridgeRecord`, `SessionStats`, `WatcherState`, `ProviderCache`, `ScanCache` 等）
- [x] 2.2 在 `lib.rs` 加入 `mod types; use types::*;`，執行 `cargo build` 確認通過

## 3. 拆分 db.rs（資料庫層）

- [x] 3.1 建立 `src-tauri/src/db.rs`，搬移 `open_db_connection`, `open_db_connection_and_init`, `init_db`, `migrate_legacy_session_cache`, `settings_file_path`, `metadata_db_path`, `ensure_parent_dir` 及所有 `load_*/save_*` DB CRUD helpers
- [x] 3.2 在 `lib.rs` 加入 `mod db; use db::*;`，執行 `cargo build` 確認通過

## 4. 拆分 settings.rs（設定管理層）

- [x] 4.1 建立 `src-tauri/src/settings.rs`，搬移 `AppSettings` impl、`load_settings_internal`, `save_settings_internal`, `detect_terminal_path`, `detect_vscode_path`, `collect_provider_integration_statuses`, `default_copilot_root`, `default_opencode_root`, `default_app_data_dir` 等函式
- [x] 4.2 在 `lib.rs` 加入 `mod settings; use settings::*;`，執行 `cargo build` 確認通過

## 5. 拆分 platform/win32_focus.rs（平台層）

- [x] 5.1 建立 `src-tauri/src/platform/win32_focus.rs`，搬移 `win32_focus` mod 內容（`find_window_by_title`, `set_foreground_window` 等 WIN32 FFI）
- [x] 5.2 建立 `src-tauri/src/platform/mod.rs` 宣告 `pub mod win32_focus;`
- [x] 5.3 在 `lib.rs` 加入 `mod platform;`，執行 `cargo build` 確認通過

## 6. 拆分 stats.rs（統計計算層）

- [x] 6.1 建立 `src-tauri/src/stats.rs`，搬移 `parse_session_stats_internal`, `get_session_stats_cache`, `upsert_session_stats_cache`, `session_events_mtime`, `calculate_opencode_session_stats`, `get_opencode_session_stats_internal`, `get_session_stats_internal` 及相關輔助函式
- [x] 6.2 在 `lib.rs` 加入 `mod stats; use stats::*;`，執行 `cargo build` 確認通過

## 7. 拆分 activity.rs（活動狀態層）

- [x] 7.1 建立 `src-tauri/src/activity.rs`，搬移 `get_copilot_activity_status`, `get_opencode_activity_status`, `get_session_activity_statuses_internal` 及相關 `OpenCodeMessageFile` struct
- [x] 7.2 在 `lib.rs` 加入 `mod activity; use activity::*;`，執行 `cargo build` 確認通過

## 8. 拆分 plan.rs（計畫檔案層）

- [x] 8.1 建立 `src-tauri/src/plan.rs`，搬移 `read_plan_internal`, `write_plan_internal`, `open_plan_external_internal`, `watch_plan_file_internal`, `read_plan_content` 及 `read_openspec_file_internal`
- [x] 8.2 在 `lib.rs` 加入 `mod plan; use plan::*;`，執行 `cargo build` 確認通過

## 9. 拆分 sisyphus.rs 與 openspec_scan.rs（掃描工具層）

- [x] 9.1 建立 `src-tauri/src/sisyphus.rs`，搬移 `SisyphusBoulder`, `SisyphusPlan`, `SisyphusNotepad`, `SisyphusData` struct 及 `scan_sisyphus_internal`, `extract_md_heading`, `extract_md_tldr`, `list_files_with_ext`
- [x] 9.2 建立 `src-tauri/src/openspec_scan.rs`，搬移 `OpenSpecChange`, `OpenSpecSpec`, `OpenSpecData` struct 及 `scan_openspec_change`, `scan_openspec_internal`
- [x] 9.3 在 `lib.rs` 加入對應 `mod` 宣告，執行 `cargo build` 確認通過

## 10. 拆分 sessions/ 子模組（Session 掃描核心層）

- [x] 10.1 建立 `src-tauri/src/sessions/copilot.rs`，搬移 `scan_copilot_incremental_internal`, `parse_workspace_file`, `scan_session_dir`, `is_live_session`, `session_id_from_dir`, `dir_mtime_secs`, `should_full_scan`
- [x] 10.2 建立 `src-tauri/src/sessions/opencode.rs`，搬移所有 `OpencodeSession*` struct、`load_opencode_projects`, `build_opencode_events_index`, `scan_opencode_sessions_internal`, `scan_opencode_incremental_internal`, `is_opencode_session_dir`, `is_opencode_session_live`, `parse_opencode_message_json`, `scan_opencode_messages_for_session`, `scan_opencode_parts_for_message`, `unix_ms_to_iso8601`
- [x] 10.3 建立 `src-tauri/src/sessions/mod.rs`，搬移 `get_sessions_internal`, `persist_provider_cache`, `instant_from_unix_secs`, 在 `sessions/mod.rs` 宣告 `pub mod copilot; pub mod opencode;`
- [x] 10.4 在 `lib.rs` 加入 `mod sessions; use sessions::*;`，執行 `cargo build` 確認通過

## 11. 拆分 provider/ 子模組（Provider 整合層）

- [x] 11.1 建立 `src-tauri/src/provider/bridge.rs`，搬移 `process_provider_bridge_event`, `read_last_bridge_record`, `register_provider_bridge_record`, `should_emit_provider_refresh_at`, `emit_provider_refresh`, `coerce_json_string`, `provider_bridge_record_fingerprint`
- [x] 11.2 建立 `src-tauri/src/provider/copilot.rs`，搬移 `detect_copilot_integration_status`, `install_or_update_copilot_integration`, `render_copilot_hook_powershell`, `render_copilot_integration`, `validate_integration_target`, `validate_managed_metadata`, `powershell_single_quoted`
- [x] 11.3 建立 `src-tauri/src/provider/opencode.rs`，搬移 `detect_opencode_integration_status`, `install_or_update_opencode_integration`, `render_opencode_integration`, `parse_opencode_integration_metadata`
- [x] 11.4 建立 `src-tauri/src/provider/mod.rs`，搬移 `build_provider_integration_status`, `managed_provider_metadata`, `recheck_provider_integration_status`, `install_or_update_provider_integration`, `resolve_provider_bridge_path`, `provider_bridge_dir`, `provider_refresh_event_name`, `read_bridge_diagnostics`, `build_install_failure_status`, 宣告三個子模組
- [x] 11.5 在 `lib.rs` 加入 `mod provider; use provider::*;`，執行 `cargo build` 確認通過

## 12. 拆分 watcher.rs（FS Watcher 層）

- [x] 12.1 建立 `src-tauri/src/watcher.rs`，搬移 `create_sessions_watcher`, `create_opencode_watcher`, `create_provider_bridge_watcher`, `restart_session_watcher_internal`, `watch_plan_file_internal`（watcher 部分）及所有 watcher 輔助函式（`build_copilot_watch_snapshot`, `should_emit_copilot_refresh`, `is_relevant_copilot_event` 等）
- [x] 12.2 在 `lib.rs` 加入 `mod watcher; use watcher::*;`，執行 `cargo build` 確認通過

## 13. 拆分 commands/ 子模組（Tauri Command 層）

- [x] 13.1 建立 `src-tauri/src/commands/sessions.rs`，搬移 `get_sessions`, `archive_session`, `unarchive_session`, `delete_session`, `delete_empty_sessions`, `get_session_stats`, `upsert_session_meta`, `delete_session_meta`, `get_session_activity_statuses`, `get_project_plans`, `get_project_specs`, `read_plan_content`, `check_directory_exists` command 函式
- [x] 13.2 建立 `src-tauri/src/commands/settings.rs`，搬移 `get_settings`, `save_settings`, `detect_terminal`, `detect_vscode`, `validate_terminal_path` command 函式
- [x] 13.3 建立 `src-tauri/src/commands/plan.rs`，搬移 `read_plan`, `write_plan`, `open_plan_external`, `watch_plan_file`, `stop_plan_watch`, `read_openspec_file` command 函式
- [x] 13.4 建立 `src-tauri/src/commands/tools.rs`，搬移 `open_terminal`, `open_in_tool`, `focus_terminal_window`, `check_tool_availability`, `restart_session_watcher` command 函式
- [x] 13.5 建立 `src-tauri/src/commands/provider.rs`，搬移 `install_provider_integration`, `update_provider_integration`, `recheck_provider_integration` command 函式
- [x] 13.6 建立 `src-tauri/src/commands/mod.rs`，`pub use` re-export 所有 5 個子模組的 command 函式
- [x] 13.7 在 `lib.rs` 加入 `mod commands; use commands::*;`，執行 `cargo build` 確認通過

## 14. 清理 lib.rs 與最終驗證

- [x] 14.1 移除 `lib.rs` 中所有業務邏輯代碼，確認只剩 `mod` 宣告、必要 `use` 及 `pub fn run()`（目標 < 70 行）
- [x] 14.2 執行 `cargo build` 確認無 error
- [x] 14.3 執行 `cargo test` 確認所有測試通過
- [x] 14.4 執行 `bun run build` 確認前端型別檢查與建置通過

---

**完成時間**：2026-04-21（最終子目錄結構完成於 2026-04-21）

## 最終模組結構

```
src-tauri/src/
├── lib.rs                  ✅ mod 宣告 + pub(crate) use + pub fn run() + tests（保留原地）
├── types.rs                ✅ 所有 struct/enum/const
├── db.rs                   ✅ SQLite 操作層
├── settings.rs             ✅ 設定管理、路徑解析
├── watcher.rs              ✅ FS Watcher 層
├── stats.rs                ✅ 統計計算層
├── activity.rs             ✅ Session 活動狀態
├── sisyphus.rs             ✅ Sisyphus 計畫掃描
├── openspec_scan.rs        ✅ OpenSpec 掃描
├── sessions/
│   ├── mod.rs              ✅ get_sessions_internal
│   ├── copilot.rs          ✅ Copilot 掃描、session 操作、plan/terminal
│   └── opencode.rs         ✅ OpenCode 掃描
├── provider/
│   ├── mod.rs              ✅ 共用函式、recheck/install 聚合
│   ├── bridge.rs           ✅ Bridge 事件、watcher snapshots
│   ├── copilot.rs          ✅ Copilot integration
│   └── opencode.rs         ✅ OpenCode integration
├── commands/
│   ├── mod.rs              ✅ restart_provider_watchers helper
│   ├── sessions.rs         ✅ session 相關 commands
│   ├── settings.rs         ✅ settings 相關 commands
│   ├── plan.rs             ✅ plan 相關 commands
│   ├── tools.rs            ✅ tools 相關 commands
│   └── provider.rs         ✅ provider integration commands
└── platform/
    ├── mod.rs              ✅
    └── win32_focus.rs      ✅ Windows Terminal focus (WIN32 FFI)
```

**驗證結果**：`cargo build` ✅ 0 errors | `cargo test` ✅ 47/47 通過
