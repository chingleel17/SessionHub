## 1. 後端：provider resume 命令

- [x] 1.1 在 `src-tauri/src/commands/tools.rs` 新增 `resume_session_in_terminal_internal(provider, session_id, cwd, terminal_path)`：依 provider 對照表（claude / codex / copilot / opencode）組出 resume 指令，複用 copilot/gemini 分支的終端啟動模式（pwsh `-NoExit` / cmd `/K`、`CREATE_NEW_CONSOLE`）；不支援的 provider 回傳 Err
- [x] 1.2 新增 `#[tauri::command] resume_session_in_terminal` 包裝函式，並於 `src-tauri/src/lib.rs` 的 invoke_handler 註冊
- [x] 1.3 在 Rust 對照表與 `src/App.tsx` 的 `getSessionOpenCommand` 兩處加上互相指向的同步註解
- [x] 1.4 `cargo check` 通過

## 2. 前端：session 開啟改為 provider 綁定

- [x] 2.1 在 `src/App.tsx` 新增 `handleResumeSession(session)`：檢查 cwd（`check_directory_exists`）後 invoke `resume_session_in_terminal`，成功/失敗顯示 toast
- [x] 2.2 `SessionCard.tsx`：移除 `LAUNCHER_OPTIONS`、launcher dropdown、portal 與 click-outside 邏輯及相關 props（`onOpenInTool` / `defaultLauncher` / `toolAvailability` / `isLauncherOpen` / `onToggleLauncher`）
- [x] 2.3 `SessionCard.tsx`：主開啟按鈕改為呼叫 `onResumeSession`，tooltip / aria-label 標明「以 <Provider> 開啟」（使用 `getProviderLabel`）
- [x] 2.4 `ProjectView.tsx`：移除 `openLauncherSessionId` 狀態與 click-outside 邏輯，更新傳給 SessionCard 的 props
- [x] 2.5 `DashboardView.tsx`：看板卡片主 chip 改為呼叫 `onResumeSession`，移除 `getDefaultLauncher` 與卡片上的下拉選單

## 3. 前端：專案層級開啟選單

- [x] 3.1 在 `src/App.tsx` 新增 `handleOpenProjectInTool(project, tool)`：以 `project.pathLabel` 為 cwd，檢查目錄存在後 invoke 既有 `open_in_tool`
- [x] 3.2 `ProjectView.tsx`：sticky header 的 sub-tab bar 右側新增「開啟專案」split button（主按鈕以 `defaultLauncher`（預設 Terminal）開啟；箭頭展開選單：Terminal / VS Code / File Explorer / Copilot CLI / OpenCode / Gemini CLI）
- [x] 3.3 選單依 `toolAvailability` 停用未安裝工具並標示；`defaultLauncher` 選項標示「預設」；點擊外部自動關閉
- [x] 3.4 `App.css` 新增/沿用 `.launcher-menu` 樣式支援 header 位置的選單（含 dark mode）

## 4. i18n 與設定頁

- [x] 4.1 `zh-TW.ts` / `en-US.ts` 新增翻譯鍵：專案開啟按鈕、以 provider 開啟的 tooltip（含 provider 名稱插值）、不支援 provider 的 toast
- [x] 4.2 `SettingsView.tsx`：更新「預設啟動工具」欄位說明文字，標明作用對象為專案開啟選單

## 5. 驗證

- [x] 5.1 `npm run build`（tsc + vite）與 `cargo check` 全數通過
- [x] 5.2 手動驗證：claude / codex / copilot / opencode session 卡片開啟各自 resume 指令；看板卡片行為一致
- [x] 5.3 手動驗證：專案頁頂部選單以 Terminal / VS Code / Explorer 開啟專案路徑；未安裝工具被停用；cwd 不存在時 toast 提示
