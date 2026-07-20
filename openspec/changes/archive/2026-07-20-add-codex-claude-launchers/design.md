## Context

「開啟專案」選單的工具清單由前端 `src/components/ProjectView.tsx` 的 `PROJECT_LAUNCHER_OPTIONS` 常數驅動，型別 `IdeLauncherType`（`src/types/index.ts`）約束可用值，實際啟動邏輯集中在後端 `src-tauri/src/commands/tools.rs` 的 `open_in_tool_internal`。可用性偵測由同檔 `check_tool_availability_internal` 以 `where <cmd>`（Windows）檢查 PATH，結果型別為 `ToolAvailability`（前端 `src/types/index.ts` 與後端 `src-tauri/src/types.rs` 各一份，透過 serde camelCase 對應）。

`codex` 與 `claude` 已是 SessionHub 支援的 provider，但尚未納入專案啟動器。現有的 `copilot`、`gemini` 分支已示範「以保留模式終端在 cwd 執行 CLI」的既有模式，`codex`/`claude` 可直接沿用。

## Goals / Non-Goals

**Goals:**
- 於專案開啟選單新增 `codex`、`claude` 兩個啟動器，並依指定順序排列全部八個工具。
- 可用性偵測涵蓋 `codex`、`claude`。

**Non-Goals:**
- 不改變 session 卡片的開啟行為（session 仍以 provider resume 指令開啟，見 `multi-ide-launcher` 既有需求）。
- 不新增非 Windows 平台的啟動邏輯（現有工具即以 Windows 為主）。
- 不調整 `defaultLauncher` 的儲存機制，只擴充其可選值。

## Decisions

- **啟動方式沿用終端保留模式**：`claude`、`codex` 與 `copilot`/`gemini` 同屬互動式 CLI，於 `open_in_tool_internal` 新增分支，重用既有 `terminal_path` + `term_stem`（cmd `/K` 或 pwsh `-NoExit -Command`）樣板，在 cwd 執行對應指令。不採用 `opencode` 那種直接 `Command::new` 開新 console 的方式，因為互動式 CLI 需在使用者選定的終端中執行以保留輸出。
  - 指令對照：`claude` → 終端執行 `claude`；`codex` → 終端執行 `codex`。
- **順序集中於前端常數**：選單順序純由 `PROJECT_LAUNCHER_OPTIONS` 陣列決定，後端不感知順序，因此重排只改前端一處。
- **`ToolAvailability` 擴充兩個布林欄位**：前後端型別同步新增 `codex`、`claude`，`check_tool_availability_internal` 以 `which_exists("codex")`、`which_exists("claude")` 補齊，維持既有偵測慣例。
- **`IdeLauncherType` 擴充兩個字面量**：新增 `"codex"`、`"claude"`，前端選單 `availKey` 指向對應可用性欄位。

## Risks / Trade-offs

- [`claude` PATH 名稱可能與其他工具衝突] → 以 `which_exists("claude")` 偵測，找不到時選單標示不可用，不強制啟動；與現有 `copilot`/`gemini` 行為一致。
- [新增啟動分支若指令名稱錯誤會靜默開啟空終端] → 沿用既有 CLI 命名（`claude`、`codex`），與 `resume_session_command` 內已驗證的指令前綴一致。

## Migration Plan

無資料遷移。前後端型別同步更新後重新編譯即可；舊版 `defaultLauncher` 設定值仍相容（新值為擴充，不移除既有值）。
