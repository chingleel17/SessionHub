## 1. 專案初始化（Tauri 2 + React + TypeScript）

- [x] 1.1 安裝 Rust toolchain（stable）與 Tauri CLI，確認 `cargo tauri` 指令可用
- [x] 1.2 使用 `cargo tauri init` 建立專案骨架，選擇 React + TypeScript 前端範本
- [x] 1.3 設定 `tauri.conf.json`：應用程式名稱、視窗尺寸（1280x800 最小）、視窗標題
- [x] 1.4 安裝前端相依套件：`react-router-dom`、`@tanstack/react-query`、`react-window`（虛擬化列表）
- [x] 1.5 建立 CSS 變數主題 token 架構（`src/styles/themes/light.css`），定義顏色、間距、字型 token
- [x] 1.6 建立多語系基礎結構（i18n provider + `zh-TW` 字典），前端骨架文案改用 key 存取

## 2. Rust 後端：資料讀取（session-list）

- [x] 2.1 在 `Cargo.toml` 加入 `serde_yaml`、`serde`（derive）相依
- [x] 2.2 定義 `WorkspaceYaml` struct（id、cwd、summary、summary_count、created_at、updated_at，所有欄位除 id 外皆為 `Option<T>`）
- [x] 2.3 實作 `scan_sessions(root_dir: &str) -> Vec<SessionInfo>` 函式
- [x] 2.4 定義 Tauri command `get_sessions`
- [x] 2.5 實作 `scan_archived_sessions(root_dir: &str)`
- [x] 2.6 在 `Cargo.toml` 加入 `rusqlite`（含 bundled feature）相依
- [x] 2.7 實作 `init_db(db_path: &str)` 建立 `session_meta` 表
- [x] 2.8 實作 `get_session_meta`、`upsert_session_meta`、`delete_session_meta`
- [x] 2.9 修改 `get_sessions`：掃描結果與 SQLite metadata merge 後回傳
- [x] 2.10 定義 Tauri commands：`upsert_session_meta`、`delete_session_meta`

## 3. Rust 後端：設定管理（app-settings）

- [x] 3.1 定義 `AppSettings` struct 並實作預設值
- [x] 3.2 實作 `load_settings()`
- [x] 3.3 實作 `save_settings(settings: AppSettings)`
- [x] 3.4 實作終端自動偵測
- [x] 3.5 實作 `validate_terminal_path(path: &str) -> bool`
- [x] 3.6 定義 Tauri commands：`get_settings`、`save_settings`、`detect_terminal`、`validate_terminal_path`

## 4. Rust 後端：Session 操作（session-actions、terminal-launcher）

- [x] 4.1 實作 `archive_session(root_dir: &str, session_id: &str)`
- [x] 4.2 實作 `delete_session(root_dir: &str, session_id: &str)`
- [x] 4.3 實作 `open_terminal(terminal_path: &str, cwd: &str)`
- [x] 4.4 實作 `check_directory_exists(path: &str) -> bool`
- [x] 4.5 定義 Tauri commands：`archive_session`、`delete_session`、`open_terminal`、`check_directory_exists`

## 5. 前端：路由與分頁架構（tabbed-ui）

- [x] 5.1 建立 `AppLayout`
- [x] 5.2 建立 `TabBar`
- [x] 5.3 實作分頁狀態管理
- [x] 5.4 建立 `openProjectTab(cwd: string)` 函式

## 6. 前端：Session 列表與篩選（session-list、session-grouping）

- [x] 6.1 建立 `useSessionsQuery` hook
- [x] 6.2 建立 `SessionCard`
- [x] 6.3 建立 `SessionList`
- [x] 6.4 建立 `SearchBar`
- [x] 6.5 建立 `SortControl`
- [x] 6.6 建立 `groupSessionsByProject(sessions)` utility

## 7. 前端：Dashboard 頁面（dashboard）

- [x] 7.1 建立 `DashboardPage`
- [x] 7.2 建立 `StatsCard`
- [x] 7.3 建立 `RecentActivityList`
- [x] 7.4 建立 `ProjectDistributionList`

## 8. 前端：Session 操作 UI（session-actions）

- [x] 8.1 建立 `ConfirmDialog`
- [x] 8.2 實作封存按鈕
- [x] 8.3 實作刪除按鈕
- [x] 8.4 實作開啟終端按鈕
- [x] 8.5 實作複製指令按鈕
- [x] 8.6 建立 `Toast`
- [x] 8.7 建立備註編輯 UI
- [x] 8.8 建立標籤 chip UI
- [x] 8.9 新增標籤多選篩選器

## 9. 前端：設定頁面（app-settings）

- [x] 9.1 建立 `SettingsPage`
- [x] 9.2 實作 `useSettingsQuery` hook
- [x] 9.3 實作終端路徑驗證 UI
- [x] 9.4 整合 Tauri 檔案對話框
- [x] 9.5 首次啟動時偵測終端

## 10. 前端：專案分頁頁面

- [x] 10.1 建立 `ProjectPage`
- [x] 10.2 整合 `SearchBar`、`SortControl`、`SessionList`
- [x] 10.3 新增「顯示封存」toggle

## 11. 整合測試與品質

- [x] 11.1 撰寫 Rust 單元測試：`scan_sessions`
- [x] 11.2 撰寫 Rust 單元測試：`validate_terminal_path`
- [x] 11.3 手動測試
- [x] 11.4 使用 `cargo tauri build` 產生 Windows 安裝檔

## 12. Rust 後端：Filesystem Watch（file-watcher）

- [x] 12.1 在 `Cargo.toml` 加入 `notify` crate 相依
- [x] 12.2 實作 `start_watcher(root_dir: &str, app_handle: AppHandle)`
- [x] 12.3 實作 `watch_plan_file(path: &str, app_handle: AppHandle)`
- [x] 12.4 實作 `stop_watcher()` 與 `restart_watcher(new_root: &str)`
- [x] 12.5 Watch 初始化失敗時 log 錯誤並回傳錯誤狀態

## 13. 前端：即時更新整合

- [x] 13.1 在 `useSessionsQuery` hook 中監聽 `sessions-updated` Tauri event
- [x] 13.2 在 Dashboard 右上角顯示「即時更新」狀態指示

## 14. Rust 後端：plan.md 讀寫（plan-viewer）

- [x] 14.1 實作 `read_plan(session_dir: &str) -> Option<String>`
- [x] 14.2 實作 `write_plan(session_dir: &str, content: &str)`
- [x] 14.3 實作 `open_plan_external(plan_path: &str, editor_cmd: Option<&str>)`
- [x] 14.4 實作 `detect_vscode() -> Option<String>`
- [x] 14.5 定義 Tauri commands：`read_plan`、`write_plan`、`open_plan_external`、`detect_vscode`
- [x] 14.6 在 `scan_sessions` 回傳資料中加入 `has_plan: bool`

## 15. 前端：plan.md 編輯器 UI（plan-viewer）

- [x] 15.1 安裝前端相依：`@codemirror/lang-markdown`、`@codemirror/view`、`marked`
- [x] 15.2 建立 `PlanEditorPanel`
- [x] 15.3 實作儲存邏輯
- [x] 15.4 監聽 `plan-file-changed` Tauri event
- [x] 15.5 在 `SessionCard` 顯示 plan 圖示標記
- [x] 15.6 在 `SessionCard` 加入「以外部編輯器開啟 plan」按鈕
- [x] 15.7 在設定頁面加入「外部編輯器路徑」輸入欄與自動偵測 VSCode 按鈕
