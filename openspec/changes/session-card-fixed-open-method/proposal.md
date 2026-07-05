# Proposal: session-card-fixed-open-method

## Why

目前 SessionCard 上的「⋯」多工具啟動選單允許使用者任選工具（Terminal / Copilot / OpenCode / Gemini / VS Code / Explorer）開啟任何 session，但這在語意上是錯的：一個 Claude Code session 只能用 `claude --resume` 恢復、Codex session 只能用 `codex resume` 恢復，用其他工具開啟只是在該目錄開了一個不相干的新環境，容易誤導使用者以為在延續原 session。「自由選擇開啟方式」真正的用途是針對**專案路徑**（開啟目錄、用 VS Code 開啟專案等），應該移到專案頁面頂部，而非放在每張 session 卡片上。

## What Changes

- **SessionCard 開啟行為改為 provider 綁定**：移除卡片上的「⋯」多工具選單，主要開啟按鈕改為「以該 session 的 provider 工具 resume」——在終端機中執行對應指令（`claude --resume=<id>`、`codex resume <id>`、`copilot --resume=<id>`、`opencode session <id>` 等），工作目錄為 session cwd。
- **新增後端命令 `open_session_in_provider`**：依 provider 組出 resume 指令並在終端機中啟動；provider 無對應 CLI 或工具未安裝時顯示錯誤 toast。
- **專案層級「開啟方式」選單**：在 ProjectView 頂部（sticky header 區）新增「開啟專案」下拉選單，選項為針對專案路徑的動作：Terminal、VS Code、File Explorer、Copilot CLI、OpenCode、Gemini CLI（沿用現有 `open_in_tool` 後端命令，cwd 改用專案路徑）。
- **Dashboard 看板卡片同步調整**：看板 session 卡片的啟動選單同樣改為 provider 綁定開啟，維持全 app 行為一致。
- **BREAKING（UI 行為）**：設定頁的「預設啟動工具」（`AppSettings.defaultLauncher`）不再影響 session 開啟行為，僅保留（或轉為）專案層級開啟選單的預設值。

## Capabilities

### New Capabilities
- `project-launcher`: 專案頁面頂部的「以指定方式開啟專案路徑」選單（Terminal / VS Code / Explorer / 其他 CLI 工具），以及其預設值行為。

### Modified Capabilities
- `multi-ide-launcher`: 移除 SessionCard／看板卡片的自由選擇工具選單；session 開啟改為 provider 綁定的 resume 行為；`defaultLauncher` 設定的作用範圍改為專案層級選單。

## Impact

- **前端**：`src/components/SessionCard.tsx`（移除 launcher dropdown、改開啟按鈕行為）、`src/components/ProjectView.tsx`（header 新增專案開啟選單）、`src/components/DashboardView.tsx`（看板卡片啟動邏輯）、`src/App.tsx`（新增 `handleOpenSessionInProvider`、傳遞專案開啟 handler）、`src/locales/zh-TW.ts` / `en-US.ts`（新增翻譯鍵）、`src/App.css`（選單樣式沿用/調整）。
- **後端**：`src-tauri/src/commands/tools.rs` 新增 `open_session_in_provider` 命令（複用 `open_terminal_internal` 的終端啟動模式）；`src-tauri/src/lib.rs` 註冊命令。
- **設定**：`AppSettings.defaultLauncher` 語意變更（不新增欄位，僅改用途）。
- **規格**：`openspec/specs/multi-ide-launcher/spec.md` 需要 delta spec；新增 `project-launcher` spec。
