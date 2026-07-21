## Why

專案頁面的「開啟專案」選單目前只提供 Terminal、VS Code、Explorer、Copilot、OpenCode、Gemini 六個工具，缺少 SessionHub 已支援的 `codex` 與 `claude`（Claude Code）兩個 CLI，使用者無法從專案選單直接以這兩個工具開啟專案。此外現有選單順序未依使用習慣排列。

## What Changes

- 新增 `codex` 與 `claude` 兩個專案啟動器工具，於 Windows 以保留模式終端在專案目錄啟動對應 CLI。
- 重新排列「開啟專案」選單的工具順序為：Terminal → VS Code → Explorer → OpenCode → Claude → Codex → Copilot → Gemini。
- 擴充工具可用性偵測，額外偵測 `codex`、`claude` 是否存在於 PATH，並在選單中依可用性標示。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `multi-ide-launcher`: 專案層級開啟選單新增 `codex`、`claude` 兩個啟動工具、調整工具排列順序，並將可用性偵測範圍擴及這兩個 CLI。

## Impact

- 前端：`src/types/index.ts`（`IdeLauncherType`、`ToolAvailability`）、`src/components/ProjectView.tsx`（`PROJECT_LAUNCHER_OPTIONS` 清單與順序）、`src/locales/*.ts`（工具標籤翻譯，如有）。
- 後端：`src-tauri/src/commands/tools.rs`（`open_in_tool_internal` 新增 `codex`/`claude` 分支、`check_tool_availability_internal` 擴充偵測）、`src-tauri/src/types.rs`（`ToolAvailability` 欄位）。
- 無資料庫 schema 變更；無 breaking change。
