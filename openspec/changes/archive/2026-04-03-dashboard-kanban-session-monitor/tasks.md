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


## 12. Bug Fixes & UX Refinements (2026-04-03)

- [x] 12.1 將 iewMode 狀態從 DashboardView 內部提升至 App.tsx，以 props 傳入，預設值改為 "kanban"，確保切換到其他頁面再返回時保留上次選擇的視圖模式
- [x] 12.2 切換按鈕順序調整：Kanban 在左（第一），清單在右（第二）
- [x] 12.3 修正 .kanban-card { overflow: hidden } 導致 launcher 下拉選單被裁切的問題（移除 overflow:hidden，改用 border-radius 上方角）
- [x] 12.4 修正 handleSaveSettings 遺漏 defaultLauncher 欄位，導致預設開啟工具設定無法儲存的問題
- [x] 12.5 統計區域從橫向 stat-bar 改為獨立 card 樣式（各有背景色），解析失敗數為 0 時自動隱藏
- [x] 12.6 新增 check_tool_availability Rust command，啟動時偵測 copilot / opencode / gemini / code 是否在 PATH 中，選單中未安裝項目顯示 disabled
- [x] 12.7 Copilot 指令從 gh copilot session 改為 copilot session（IdeLauncherType: "gh-copilot" → "copilot"）
- [x] 12.8 切換按鈕選中狀態改為藍色（#2563eb），不再依賴未定義的 --color-accent CSS 變數

## 13. Phase 3 — 看板 UX 升級（2026-04-03）

### 看板卡片改為專案分組（ProjectCard）

- [x] 13.1 將 KanbanCard（session 為單位）重構為 `KanbanProjectCard`（專案為單位）：每欄依 cwd 分組，同欄同專案的 sessions 集合在一張 ProjectCard 內
- [x] 13.2 `KanbanProjectCard` 標頭顯示：專案名稱（cwd 最後段）、session 數量 badge、平台標籤群（copilot/opencode）、最後更新時間
- [x] 13.3 `KanbanProjectCard` 支援展開/收折（預設展開），收折後僅顯示標頭
- [x] 13.4 展開後以輕量列表行顯示每個 session：summary（截至 60 字元）、activity badge、啟動按鈕（platform-aware 預設工具）

### Platform-Aware 預設啟動工具

- [x] 13.5 實作 `get_platform_default_launcher(session: &SessionInfo, global_default: Option<&str>) -> &str` 輔助函式：Copilot session → "copilot"，OpenCode session → "opencode"，其他 → global_default → "terminal"
- [x] 13.6 前端 `getDefaultLauncher(session, toolAvailability, globalDefault)` helper：依 provider 決定預設工具，若工具不可用則退回 terminal
- [x] 13.7 KanbanProjectCard 與 SessionCard 的主按鈕 icon/label 反映 platform-aware 預設工具（而非固定 icon）

### Done 欄位數量限制

- [x] 13.8 Done 欄初始只渲染最新 10 個 ProjectCard（依 lastActivityAt 排序）
- [x] 13.9 Done 欄底部新增「載入更多」按鈕，每次追加 10 個；或支援捲動觸底自動載入

### 看板欄位寬度持久化

- [x] 13.10 四欄預設平均分配（25% each），以 CSS grid `fr` 單位實作
- [x] 13.11 實作欄位寬度拖拉調整（mousedown on column divider）
- [x] 13.12 欄寬儲存至 localStorage key `sessionhub.kanban.columnWidths`，頁面初始化時讀取並套用

### 統計週期切換排版

- [x] 13.13 將「本週 / 本月」切換按鈕從 stat card 抽離，改為獨立橫向列，置於 Kanban/清單切換按鈕的正下方、統計卡片上方
- [x] 13.14 調整 DashboardView layout：視圖切換按鈕列 → 週期切換列 → 統計卡片列 → 看板/清單主內容

### Copilot CLI 指令修正

- [x] 13.15 調查並確認正確的 Copilot CLI resume 指令格式（可能為 `copilot resume <session_id>` 或 `copilot session <session_id>`），修正 Rust open_in_tool 的 copilot 路由
