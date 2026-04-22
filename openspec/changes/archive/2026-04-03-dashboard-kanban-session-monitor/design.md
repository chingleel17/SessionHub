## Context

SessionHub 目前的 Dashboard 為純列表形式，缺乏跨專案的統一 session 狀態視圖。SessionCard 只有單一終端啟動按鈕，PlansSpecsView 只顯示 change/spec 名稱清單。本次變更新增 5 個新能力：Kanban 看板、Session 活動狀態偵測、多工具啟動器、終端機 bring-to-front、OpenSpec 內容閱讀器。

現有相關程式碼：
- `src-tauri/src/lib.rs`：`open_terminal` command 處理終端啟動邏輯（pwsh/cmd/bash 白名單）
- `src/components/DashboardView.tsx`：純顯示元件，props 驅動
- `src/components/PlansSpecsView.tsx`：顯示 OpenSpec changes 與 specs 清單
- `src/types/index.ts`：`AppSettings` 含 `terminalPath`、`externalEditorPath`

## Goals / Non-Goals

**Goals:**
- Dashboard 新增 Kanban 視圖（與現有清單視圖切換）
- Session 活動狀態自動偵測（Idle / Active / Waiting / Done + 細節）
- SessionCard / ProjectCard 新增多工具啟動下拉（5 種工具）
- 設定頁可設定預設啟動工具
- 終端機視窗 bring-to-front（Win32 API，best-effort）
- PlansSpecsView 可展開閱讀 spec/change 的 md 文件內容

**Non-Goals:**
- Windows Terminal 特定 tab 的精確切換（API 不公開）
- Remote/distributed session 監控（非本機 sessions）
- Session 狀態的持久化儲存（每次重新偵測）
- Kanban 卡片拖拉手動修改狀態

## Decisions

### 1. Session 活動狀態偵測來源

**Decision**：Copilot 讀 `events.jsonl`，OpenCode 讀最新的 `msg_*.json` + `prt_*.json`。

**Rationale**：
- Copilot 的 `events.jsonl` 已有解析基礎（`has_events` 欄位），每行是一個 JSON event 物件
- OpenCode 的 message/part 檔案已有 stats 解析，msg 檔案含 `role` 欄位
- 兩者都可由 mtime 判斷活躍程度，不需額外 polling

**狀態推斷邏輯**：
```
done    → session.is_archived 或 last_activity > 24h
waiting → last event/msg role == "assistant"，且 last_activity < 2h
active  → last event/msg role == "user" 或 tool call，且 last_activity < 30min
idle    → 其他（has_events 但無近期活動）
```

**活動細節（active 子狀態）**：
- `thinking`：last event 含 reasoning / thinking type
- `tool_call`：last event type 是 tool_use 或 tool_result
- `file_op`：last tool call 是 read_file / write_file / edit_file 類
- `sub_agent`：last tool call 是 spawn_agent / subagent 類
- `working`：其他 active 情況

**Alternative considered**：輪詢 process list 找 AI agent process。排除原因：不同 provider process 名稱不穩定，且 Tauri sandboxed 環境下 process 列舉有限制。

### 2. 多工具啟動器架構

**Decision**：新增 `open_in_tool(tool_type, cwd, session_id)` command 取代現有 `open_terminal`；`open_terminal` 保留為 alias 向後相容。

**工具類型對應**：
- `terminal`：現有邏輯（settings.terminalPath + pwsh/cmd/bash 白名單）
- `opencode`：在終端中執行 `opencode --cwd <cwd>`（需要 settings.opencodeRoot 路徑或 PATH 中的 opencode）
- `gh-copilot`：在終端中執行 `gh copilot session resume <session_id>`
- `gemini`：在終端中執行 `gemini`（需 gemini CLI 在 PATH 中）
- `explorer`：`explorer.exe <cwd>`（直接 spawn，不需終端）

**設定新增**：`AppSettings.defaultLauncher: Option<String>`，對應 `IdeLauncherType` enum。

**Alternative considered**：在前端組合啟動指令後呼叫 `open_terminal`。排除原因：opencode/gh-copilot/gemini 在終端中執行的參數組合邏輯應由 Rust 端統一處理，避免前端 hardcode 指令格式。

### 3. 終端機 Bring-to-Front（Win32 API）

**Decision**：使用 `windows-sys` crate（已有間接依賴，或直接新增）實作 `focus_terminal_window(cwd)` command。

**實作策略**：
1. `EnumWindows` 遍歷所有頂層視窗
2. 對每個視窗 `GetWindowText` 取得標題，`GetClassName` 取得類別
3. 比對已知終端 class（`CASCADIA_HOSTING_WINDOW_CLASS` = Windows Terminal，`ConsoleWindowClass` = cmd/pwsh）
4. 比對視窗標題是否包含 cwd 的最後一段路徑名（或 session summary）
5. 找到後呼叫 `SetForegroundWindow` + `ShowWindow(SW_RESTORE)`
6. 找不到則回傳 `Err("terminal window not found")`；前端顯示 toast 提示

**已知限制**：
- Windows Terminal 內部多 tab 無法精確切換（tab title 在主視窗 title 中不一定反映）
- `SetForegroundWindow` 在某些 Windows 版本下受前景鎖定保護（可能需要 `AttachThreadInput`）

**Cargo.toml 新增**：
```toml
[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.52", features = ["Win32_UI_WindowsAndMessaging", "Win32_System_Threading", "Win32_Foundation"] }
```

### 4. OpenSpec 內容閱讀器

**Decision**：新增 `read_openspec_file(relative_path)` Tauri command，回傳檔案的 UTF-8 文字內容；前端使用已有的 `marked` + `DOMPurify` 渲染 markdown。

**路徑安全性**：`relative_path` 必須在 `openspec/` 目錄下（backend 驗證），防止路徑穿越。

**Alternative considered**：直接用 `@tauri-apps/plugin-fs` 在前端讀檔。排除原因：plugin-fs 的 scope 設定複雜，且後端驗證路徑更安全。

### 5. Dashboard Kanban 視圖架構

**Decision**：在 `DashboardView` 新增 `viewMode: "list" | "kanban"` state；Kanban 視圖使用 CSS Grid 四欄排版（`kanban-board`）；不使用 DnD（狀態為唯讀自動偵測）。

**欄位定義**：
- `Idle`：session 有 events 但無近期活動（idle 狀態）
- `Active`：偵測到近期活動（active 狀態）
- `Waiting`：等待用戶回應（waiting 狀態）
- `Done`：已封存或超過 24h 無活動（done 狀態）

**Session Activity Status 資料獲取**：
- 新增 `get_session_activity_statuses(session_ids: Vec<String>)` command
- 一次批次查詢多個 session 的活動狀態，回傳 `Vec<SessionActivityStatus>`
- 不快取（每次 Dashboard 開啟時重新讀取），用 React Query 設 `staleTime: 30_000`（30 秒）

## Risks / Trade-offs

- **Win32 API 終端 focus**：`SetForegroundWindow` 在部分 Windows 版本會失敗（需要 `AllowSetForegroundWindow`）。Mitigation：失敗時前端顯示 toast "無法自動切換，請手動切換至終端"，不拋例外。

- **Session 狀態偵測準確性**：events.jsonl / msg 檔案格式若 provider 更新後改變，狀態推斷可能誤判。Mitigation：偵測邏輯盡可能只依賴 mtime + role 欄位等穩定字段，避免深度解析 event type 字串。

- **Kanban 視圖效能**：大量 session 時批次讀取 activity status 可能慢。Mitigation：只讀取 non-archived sessions；每個 session 只讀最後幾行（tail）而非整個 events.jsonl。

- **opencode/gemini CLI 路徑**：使用者可能沒有安裝。Mitigation：前端在啟動前不驗證，啟動失敗時終端自行顯示錯誤；按鈕永遠可見（不做安裝偵測，保持簡單）。

## Migration Plan

1. 新增 Rust types + commands（向後相容，不刪除現有 command）
2. 更新 `AppSettings` 型別（新增 optional 欄位，舊 settings.json 向後相容）
3. 前端新增元件與視圖，現有元件只做增量修改
4. 設定頁新增 `defaultLauncher` 選項
5. 無資料庫 schema 變更，無 migration 需求

## Open Questions

- OpenCode 的 `gemini` provider 與 Gemini CLI 是不同工具。Gemini CLI 啟動指令假設為 `gemini`（Google 官方 CLI），實作時需確認正確的 CLI 指令名稱。
- Copilot `events.jsonl` 經實際調查確認，事件類型包括：`session.start`、`session.shutdown`、`session.task_complete`、`assistant.turn_start`、`assistant.turn_end`、`assistant.message`、`tool.execution_start`、`tool.execution_complete`、`session.mode_changed`。工具名稱在 `data.toolName` 欄位（如 `task_complete`、`read_file` 等）。
- OpenCode 狀態偵測來源確認為 `storage/message/ses_xxx/msg_*.json`（含 `role`, `finish`, `time` 欄位）與 `storage/part/msg_xxx/prt_*.json`（含 `type`, `tool`, `state.status` 欄位）。工具名稱如 `edit`、`glob`、`task`、`call_omo_agent`、`patch` 等。
- `events.jsonl` 無 `thinking` 相關 event type（Copilot 不直接暴露 reasoning 事件），thinking 細節僅 OpenCode 可偵測（part type=reasoning）。


---

## UX Refinements (2026-04-03)

### viewMode 持久化

- `viewMode` 狀態提升至 `App.tsx`，以 `dashboardViewMode` state 管理
- 預設值為 `"kanban"`（原本為 `"list"`）
- 當使用者切換至專案頁或設定頁後返回儀表板，保留上次的視圖模式（不重置）
- Props: `DashboardView` 新增 `viewMode: "list" | "kanban"` 和 `onViewModeChange: (mode) => void`

### 切換按鈕順序

- 看板 (Kanban) 在左側（第一個），清單 (List) 在右側（第二個）
- 預設選中看板

### Launcher 下拉選單定位修正

- `.kanban-card` 移除 `overflow: hidden`，避免下拉選單被卡片邊框裁切
- 狀態色條改用 `border-radius: 6px 6px 0 0` 保持圓角
- 下拉包裝在 `position: relative` 的 div 中，確保定位正確

### 預設開啟工具設定修正

- `handleSaveSettings` 補上 `defaultLauncher: settingsForm.defaultLauncher ?? null`
- 先前遺漏此欄位導致設定無法儲存

### 工具可用性偵測

- Rust command `check_tool_availability`：執行 `where <cmd>` 偵測 copilot / opencode / gemini / code 是否在 PATH
- App 初始化時查詢一次（staleTime: Infinity），結果傳入 DashboardView 與 ProjectView
- 選單中未偵測到的工具顯示 disabled 狀態

### 統計卡片重設計

- 從橫向 `stat-bar` 改為彈性排列的獨立 card（各有背景色）
- parseErrorCount 為 0 時不顯示 error card

### IdeLauncherType 更新

- `"gh-copilot"` 改為 `"copilot"`，指令從 `gh copilot session` 改為 `copilot session`

---

## UX Refinements (2026-04-03, Phase 2)

### 已知問題：Copilot CLI 無法開啟

- 測試發現 `copilot session` 指令無法成功開啟 session
- 需調查正確的 Copilot CLI resume 指令格式（可能為 `copilot resume <session_id>` 或其他格式）
- 在查明前，Copilot 工具在選單中保持顯示但應有明確錯誤提示

### Platform-Aware 預設啟動工具

- 開啟工具選單的預設選項應依照 **session 的 provider** 決定，而非全域設定
  - Copilot session → 預設 `copilot`
  - OpenCode session → 預設 `opencode`
  - 無法判斷 / 其他 → 退回 AppSettings.defaultLauncher，再退回 `terminal`
- 全域設定中的「預設啟動工具」改為**覆蓋**用途（optional override），不再是唯一依據
- KanbanCard 與 SessionCard 的主按鈕 icon 應反映 platform-aware 預設工具

### Done 欄位數量限制

- Done 欄位（已完成 sessions）僅顯示最新的 **10 個**，避免大量已完成 session 佔滿畫面
- 超過 10 個時，顯示「載入更多」按鈕或支援捲動觸底自動載入（infinite scroll）
- 已封存 sessions 與超過 24h 無活動 sessions 均歸入 Done

### 看板欄位寬度

- 四欄預設**平均分配**寬度（各 25%），取代原本內容驅動的寬度
- 支援**手動拖拉調整**各欄寬度（CSS resize 或 mousedown 拖拉）
- 使用者調整的欄寬應**持久化儲存**至 AppSettings 或 localStorage，下次開啟時恢復

### 看板卡片：以專案為單位

- 看板顯示單位從「一個 session 一張卡片」改為**「一個專案一張卡片」**
- 同一專案、同一狀態欄的 sessions 集合在同一 ProjectCard 中
- ProjectCard 預設**展開**顯示 session 清單，可手動**收折**（toggle）
- ProjectCard 標頭顯示：專案名稱、session 數量、平台標籤群、最後更新時間
- 收折後標頭仍顯示 session 數量與狀態摘要
- 展開後每個 session 以輕量列表行顯示（summary、activity badge、啟動按鈕）

### 統計週期切換排版調整

- 原本的「本週 / 本月」切換按鈕從嵌入 stat card 改為獨立的切換列
- 放置位置：**Kanban/清單切換按鈕的下方**，統計卡片的上方
- 排版為**橫向（水平）排列**（非垂直）

### Kanban/清單切換按鈕位置調整

- 切換按鈕（Kanban / 清單）維持現有位置（Dashboard 頂部）
- 統計週期切換（本週 / 本月）放在切換按鈕正下方，作為次級控制列
