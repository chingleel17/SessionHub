# Design: session-card-fixed-open-method

## Context

目前 session 開啟相關的元件與流程：

- `SessionCard.tsx`：主開啟按鈕呼叫 `onOpenTerminal`（只開純終端到 cwd），旁邊的「⋯」按鈕展開 `LAUNCHER_OPTIONS` 下拉（terminal / copilot / opencode / gemini / vscode / explorer），選了之後呼叫 `onOpenInTool` → 後端 `open_in_tool`——僅在 cwd 開啟該工具，**不會 resume session**。
- `DashboardView.tsx` 看板卡片：主 chip 依 `getDefaultLauncher()`（provider-aware + `defaultLauncher` 覆蓋）開啟，另有同款下拉選單。
- `App.tsx` 的 `getSessionOpenCommand(provider, sessionId)` 已有各 provider 的 resume 指令對照（供「複製指令」功能使用）：`copilot --resume=`、`opencode session`、`codex resume`、`claude --resume=`。
- 後端 `tools.rs::open_in_tool_internal` 已示範「在新終端視窗內執行 CLI」的模式（copilot / gemini 分支：`pwsh -NoExit -Command "cd '<cwd>'; <tool>"` 或 `cmd /K`）。

問題：session 的開啟方式不該可選——Claude session 只能用 claude CLI resume。自由選擇工具的需求其實是針對「專案路徑」的（開資料夾、用 VS Code 開專案）。

## Goals / Non-Goals

**Goals:**
- SessionCard 與看板卡片的開啟動作固定為「以 session 的 provider CLI resume 該 session」。
- 新後端命令在終端機中執行 provider 對應的 resume 指令。
- ProjectView 頂部新增專案層級「開啟方式」選單（針對專案路徑，非 session）。
- `defaultLauncher` 設定改為專案層級選單的預設值。

**Non-Goals:**
- 不改變「複製指令」、「聚焦終端」等其他卡片動作。
- 不新增 AppSettings 欄位、不做設定遷移。
- 不處理 macOS/Linux 終端啟動（維持現有 Windows 為主的行為）。
- 不改 Sidebar / 其他頁面的專案開啟入口。

## Decisions

### D1. Resume 指令對照放在 Rust 後端

新增 `resume_session_in_terminal(provider, session_id, cwd, terminal_path)` 命令，provider → 指令對照寫在 Rust：

| provider | 指令 |
|---|---|
| `claude` | `claude --resume=<session_id>` |
| `codex` | `codex resume <session_id>` |
| `copilot` | `copilot --resume=<session_id>` |
| `opencode` | `opencode session <session_id>` |
| 其他 | 回傳 `Err("unsupported provider")`，前端 toast 提示 |

指令與前端 `getSessionOpenCommand`（複製指令功能）保持一致；兩處對照表需同步（在兩處各加註解互相指向）。啟動方式複用 `open_in_tool_internal` 中 copilot/gemini 分支的模式：以使用者設定的 `terminal_path`（預設 `pwsh`）開新 console，`-NoExit`（或 cmd `/K`）執行 `cd` + resume 指令，session 結束後終端保留。

替代方案：前端組好指令字串傳給後端執行——被否決，避免 IPC 介面變成任意命令執行，且 provider 對照屬於平台知識，與 `open_in_tool_internal` 放在同一處較內聚。

### D2. SessionCard UI：移除「⋯」選單，主按鈕改為 provider 開啟

- 移除 `LAUNCHER_OPTIONS`、launcher dropdown 與相關 portal / click-outside 邏輯（`SessionCard.tsx`）。
- 原「開啟終端」按鈕改為主開啟動作：「以 <Provider> 開啟」（icon 沿用 TerminalIcon，title 帶 provider 名稱，如「以 Claude Code 恢復此 session」），呼叫新 handler `onResumeSession`。
- 卡片 props 移除 `onOpenInTool` / `defaultLauncher` / `toolAvailability` / `isLauncherOpen` / `onToggleLauncher`（若看板仍需可留在 DashboardView 內部）。
- 「聚焦終端」「複製指令」等按鈕不動。

### D3. 看板卡片同步改為 provider 開啟

`DashboardView.tsx` 看板卡片：主 chip 點擊改為呼叫 `onResumeSession`；移除 `getDefaultLauncher` 與下拉選單。維持與 SessionCard 一致的心智模型。

### D4. 專案層級「開啟方式」選單放在 ProjectView sticky header

- 在 `sticky-project-shell` 的 sub-tab bar 右側加一顆「開啟專案」按鈕 + 下拉選單（樣式沿用現有 `.launcher-menu` / `.launcher-menu-item`）。
- 選項沿用 `IdeLauncherType`：Terminal、VS Code、File Explorer、Copilot CLI、OpenCode、Gemini CLI；以 `toolAvailability` 標示未安裝並 disable。
- 點選後呼叫 `onOpenProjectInTool(tool)` → App.tsx 以 `open_in_tool`（現有命令，不改後端）搭配 `project.pathLabel` 作為 cwd 執行。
- `defaultLauncher` 設定值在選單中以「預設」標示；按鈕主點擊（不展開選單）直接以 defaultLauncher 開啟，未設定時預設 Terminal——保留原 `multi-ide-launcher` 的預設工具語意，只是作用對象從 session 變成專案。

### D5. IPC 仍集中於 App.tsx

依專案慣例，新 handler `handleResumeSession(session)` 與 `handleOpenProjectInTool(project, tool)` 都寫在 `App.tsx`，子元件僅收 callback props。開啟前沿用既有的 `check_directory_exists` 檢查與 toast 錯誤處理。

## Risks / Trade-offs

- [Provider CLI 未安裝時 resume 失敗] → 終端會顯示 command not found；後端 spawn 成功與否無法涵蓋此情況。緩解：沿用 `check_tool_availability` 已涵蓋的工具（copilot/opencode）在前端預先 disable 或 toast；claude/codex 不在 availability 檢查內，維持終端內錯誤即可（使用者可直接看到）。
- [resume 指令隨 CLI 版本變動]（如 ced0358 已修過一次）→ 對照表集中在 Rust 一處 + 前端複製指令一處，兩處互相註解指向，降低漏改風險。
- [`project.pathLabel` 在少數情況可能非實際路徑] → 開啟前以 `check_directory_exists` 驗證，失敗顯示 `toast.cwdMissing`。
- [移除 session 層級自由選擇是行為破壞] → 這正是本變更目的；專案層級選單提供了原功能的合理去處（開 VS Code / Explorer 本來就與特定 session 無關）。

## Open Questions

- 無。opencode resume 指令沿用現有 `getSessionOpenCommand` 的 `opencode session <id>`；若日後 CLI 語法變更，另開 change 處理。
