## 1. Rust Backend — 型別與 Cargo 設定

- [x] 1.1 在 `src-tauri/Cargo.toml` 新增 `windows-sys` dependency（features: Win32_UI_WindowsAndMessaging, Win32_System_Threading, Win32_Foundation）
- [x] 1.2 在 `lib.rs` 新增 `SessionActivityStatus` struct（fields: session_id, status: String, detail: Option<String>, last_activity_at: Option<String>）並加 `#[serde(rename_all = "camelCase")]`
- [x] 1.3 在 `lib.rs` 的 `AppSettings` struct 新增 `default_launcher: Option<String>` 欄位
- [x] 1.4 在 `src/types/index.ts` 新增 `SessionActivityStatus`、`SessionActivityDetail`、`IdeLauncherType` 型別，並在 `AppSettings` 新增 `defaultLauncher?: string | null`

## 2. Rust Backend — Session 活動狀態偵測

- [x] 2.1 實作 `get_session_activity_status_internal(session: &SessionInfo) -> SessionActivityStatus`：讀取 Copilot `events.jsonl` 末尾 20 行，解析最後 event 的 role / type，推斷 idle / active / waiting / done 狀態與細節
- [x] 2.2 實作 OpenCode session 的狀態推斷邏輯：讀取最新 `msg_*.json` 的 role 欄位與 mtime，套用相同的時間窗口規則（30min active、2h waiting）
- [x] 2.3 新增 `#[tauri::command] get_session_activity_statuses(session_ids: Vec<String>, state: State<AppState>)` command，批次查詢並回傳 `Vec<SessionActivityStatus>`
- [x] 2.4 在 `invoke_handler![]` 中登記 `get_session_activity_statuses`

## 3. Rust Backend — 多工具啟動器

- [x] 3.1 新增 `#[tauri::command] open_in_tool(tool_type: String, cwd: String, session_id: Option<String>, state: State<AppState>)` command 及對應的 `open_in_tool_internal`
- [x] 3.2 實作 `terminal` 路由（複用現有 `open_terminal` 邏輯）
- [x] 3.3 實作 `opencode` 路由（在終端中執行 `opencode --cwd <cwd>`）
- [x] 3.4 實作 `gh-copilot` 路由（在終端中執行 `gh copilot session resume <session_id>`）
- [x] 3.5 實作 `gemini` 路由（在終端中執行 `gemini`，cwd 為工作目錄）
- [x] 3.6 實作 `explorer` 路由（spawn `explorer.exe <cwd>`，不開啟終端）
- [x] 3.7 在 `invoke_handler![]` 中登記 `open_in_tool`

## 4. Rust Backend — 終端機 Focus（Win32 API）

- [x] 4.1 實作 `focus_terminal_window_internal(cwd: &str) -> Result<(), String>`：使用 `EnumWindows` 遍歷頂層視窗，比對 class（ConsoleWindowClass / CASCADIA_HOSTING_WINDOW_CLASS）及標題中的 cwd 路徑名（不區分大小寫）
- [x] 4.2 找到匹配視窗後呼叫 `ShowWindow(SW_RESTORE)` + `SetForegroundWindow`；若失敗嘗試 `AttachThreadInput` 後重試
- [x] 4.3 新增 `#[tauri::command] focus_terminal_window(cwd: String)` command 並在 `invoke_handler![]` 中登記
- [x] 4.4 以 `#[cfg(target_os = "windows")]` 包裝 Win32 相關程式碼

## 5. Rust Backend — OpenSpec 文件讀取

- [x] 5.1 實作 `read_openspec_file_internal(project_cwd: &str, relative_path: &str) -> Result<String, String>`：正規化路徑，驗證結果在 `<project_cwd>/openspec/` 下，回傳 UTF-8 文字內容
- [x] 5.2 新增 `#[tauri::command] read_openspec_file(project_cwd: String, relative_path: String)` command 並在 `invoke_handler![]` 中登記

## 6. 前端 — Session 活動狀態整合

- [x] 6.1 在 `App.tsx` 新增 `useQuery` 呼叫 `get_session_activity_statuses`，傳入 non-archived session ids，`staleTime: 30_000`，並在 sessions 更新時 invalidate
- [x] 6.2 建立 `activityStatusMap: Map<string, SessionActivityStatus>` 並傳入需要的元件

## 7. 前端 — Dashboard Kanban 視圖

- [x] 7.1 在 `DashboardView.tsx` 新增 `viewMode: "list" | "kanban"` state 與切換按鈕（清單 / Kanban）
- [x] 7.2 新增 Kanban 版面：四欄 CSS Grid（`kanban-board`），欄標題顯示狀態名與 session 數量
- [x] 7.3 新增 `KanbanCard` 子元件（或 inline JSX）：顯示 session summary、專案名、provider tag、最後更新時間、activity detail badge
- [x] 7.4 在 `DashboardView.tsx` Props 新增 `activityStatuses: SessionActivityStatus[]` 與 `onOpenInTool: (session, toolType) => void`
- [x] 7.5 在 `App.tsx` 更新 `DashboardView` 的 props 傳遞，加入活動狀態與啟動工具 handler
- [x] 7.6 在 `src/styles/` 或既有 CSS 新增 `.kanban-board`、`.kanban-column`、`.kanban-card` 等樣式

## 8. 前端 — 多工具啟動下拉（SessionCard）

- [x] 8.1 在 `SessionCard.tsx` Props 新增 `onOpenInTool: (session: SessionInfo, toolType: string) => void` 與 `defaultLauncher?: string`
- [x] 8.2 實作工具啟動下拉元件（dropdown button）：主按鈕使用 defaultLauncher 對應的 icon，展開後顯示 5 個工具選項，預設工具旁標示「預設」
- [x] 8.3 在 `App.tsx` 新增 `handleOpenInTool` mutation 呼叫 `open_in_tool`，並傳遞給 `SessionCard`
- [x] 8.4 新增終端機 focus 按鈕至 `SessionCard` actions（呼叫 `focus_terminal_window`，失敗時 showToast 提示）
- [x] 8.5 在 CSS 新增工具啟動下拉的樣式（`.tool-launcher-dropdown`）

## 9. 前端 — OpenSpec 內容閱讀器

- [x] 9.1 在 `PlansSpecsView.tsx` 新增 changes 項目展開邏輯：點擊展開顯示文件列表（proposal / design / tasks，標示是否存在）
- [x] 9.2 新增 spec 項目展開邏輯：點擊展開呼叫 `read_openspec_file` 讀取 spec.md 並以 markdown 渲染
- [x] 9.3 在 `App.tsx` 新增 `handleReadOpenspecFile` 呼叫 `read_openspec_file` command，傳遞給 `PlansSpecsView`
- [x] 9.4 新增展開面板的 CSS 樣式（`.openspec-content-panel`，含 markdown 渲染區塊樣式）

## 10. 前端 — 設定頁預設啟動工具

- [x] 10.1 在 `SettingsView.tsx` 新增「預設啟動工具」選項（`<select>` 或 radio group），對應 `AppSettings.defaultLauncher`
- [x] 10.2 在翻譯檔（`src/i18n/`）新增預設啟動工具相關的翻譯 key

## 11. 翻譯與最終整合

- [x] 11.1 在翻譯檔新增 Kanban 視圖相關 key（欄位名稱、狀態標籤、activity detail 標籤）
- [x] 11.2 新增終端機 focus 失敗的 toast 翻譯 key
- [x] 11.3 執行 `bun run build` 確認 TypeScript 無型別錯誤
- [x] 11.4 執行 `cd src-tauri && cargo test` 確認 Rust 編譯與測試通過

