## Why

SessionHub 目前的 Dashboard 僅提供統計數字與最近 session 清單，無法跨專案統一監控多個 AI coding session 的即時狀態。隨著「同時處理多專案多 session」成為主要工作模式，需要一個可以一眼掌握所有 session 活動狀態、快速切換工具、並深入查看 spec 內容的管理介面。

## What Changes

- **新增 Dashboard Kanban 視圖**：與現有 Dashboard 並列，提供切換選項；以 Idle / Active / Waiting / Done 四欄 Kanban 方式展示所有跨專案 session，狀態由系統自動從 events.jsonl / workspace.yaml 推斷
- **Session 活動狀態偵測**：Rust backend 解析 session 目錄的 events 或 message 檔案，推斷每個 session 的當前活動狀態（Thinking、Tool Call、File Op、Waiting for User、Done、Idle）
- **多工具啟動器**：SessionCard 與 ProjectCard 的「開啟」按鈕擴充為下拉選單，支援 OpenCode、Copilot CLI（gh copilot session resume）、Gemini CLI、Terminal、File Explorer 五種選項；設定頁可設定預設啟動工具
- **終端機 Bring-to-Front**：新增 `focus_terminal` Tauri command，透過 Win32 API（EnumWindows + SetForegroundWindow）嘗試找到並提前對應 cwd 的終端視窗（best-effort，不保證 Windows Terminal tab 切換）
- **OpenSpec 內容閱讀器**：PlansSpecsView 中的 changes 與 specs 項目可展開，顯示 proposal.md、design.md、tasks.md、spec.md 等 markdown 文件的渲染內容

## Capabilities

### New Capabilities

- `dashboard-kanban`: 跨專案 Kanban 看板視圖，依 session 活動狀態分四欄展示所有 session，與現有 Dashboard 並列切換
- `session-activity-status`: Session 活動狀態偵測——Rust backend 讀取 events.jsonl（Copilot）或 message/part JSON（OpenCode），推斷 Idle / Active / Waiting / Done 狀態及細節（Thinking、Tool Call、File Op 等）
- `multi-ide-launcher`: 多工具啟動選單（opencode / gh-copilot / gemini / terminal / explorer），SessionCard 與 ProjectCard 均支援；設定中可設定 `defaultLauncher`
- `terminal-focus`: Win32 API 實作的終端機視窗 bring-to-front，透過 EnumWindows 比對視窗 title 或 process 找到終端，呼叫 SetForegroundWindow（best-effort）
- `openspec-content-viewer`: PlansSpecsView 展開閱讀 OpenSpec md 檔案內容，含 markdown 渲染

### Modified Capabilities

- `dashboard`: 新增 Kanban 視圖切換按鈕，補充現有統計列與專案清單視圖
- `terminal-launcher`: 擴充為多工具啟動器入口，保留現有終端啟動邏輯
- `app-settings`: 新增 `defaultLauncher` 欄位（可選值：opencode / gh-copilot / gemini / terminal / explorer）
- `openspec-reader`: 擴充 Tauri command 以讀取並回傳指定 md 檔案內容（純文字）

## Impact

- `src-tauri/src/lib.rs`：新增 `get_session_activity_statuses`、`read_openspec_file`、`focus_terminal_window` commands；`open_in_tool` 取代 / 擴充現有 `open_terminal`；新增 Win32 API 依賴（`windows-sys` crate）
- `src-tauri/Cargo.toml`：新增 `windows-sys` dependency（features: Win32_UI_WindowsAndMessaging, Win32_System_Threading, Win32_Foundation）
- `src/types/index.ts`：新增 `SessionActivityStatus`、`SessionActivityDetail`、`IdeLauncherType` 型別；`AppSettings` 新增 `defaultLauncher`
- `src/components/DashboardView.tsx`：新增 Kanban 版面及視圖切換
- `src/components/SessionCard.tsx`：新增多工具啟動下拉選單
- `src/components/PlansSpecsView.tsx`：新增 md 內容展開閱讀器
- `src/components/SettingsView.tsx`：新增預設啟動工具選項
- `src/App.tsx`：新增對應 invoke 呼叫與 mutation handlers
