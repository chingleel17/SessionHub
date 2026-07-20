## 1. 型別擴充

- [x] 1.1 `src/types/index.ts`：`IdeLauncherType` 新增 `"codex"`、`"claude"` 兩個字面量
- [x] 1.2 `src/types/index.ts`：`ToolAvailability` 新增 `codex: boolean`、`claude: boolean` 欄位
- [x] 1.3 `src-tauri/src/types.rs`：`ToolAvailability` struct 新增 `pub codex: bool`、`pub claude: bool` 欄位

## 2. 後端啟動與偵測

- [x] 2.1 `src-tauri/src/commands/tools.rs`：`open_in_tool_internal` 新增 `"claude"` 分支，沿用終端保留模式於 cwd 執行 `claude`
- [x] 2.2 `src-tauri/src/commands/tools.rs`：`open_in_tool_internal` 新增 `"codex"` 分支，沿用終端保留模式於 cwd 執行 `codex`
- [x] 2.3 `src-tauri/src/commands/tools.rs`：`check_tool_availability_internal` 以 `which_exists("codex")`、`which_exists("claude")` 補齊兩個新欄位

## 3. 前端選單清單與順序

- [x] 3.1 `src/components/ProjectView.tsx`：`PROJECT_LAUNCHER_OPTIONS` 新增 `claude`、`codex` 項目，`availKey` 指向對應可用性欄位
- [x] 3.2 `src/components/ProjectView.tsx`：重排 `PROJECT_LAUNCHER_OPTIONS` 為 Terminal → VS Code → Explorer → OpenCode → Claude → Codex → Copilot → Gemini
- [x] 3.3 設定頁預設啟動工具選項若硬列工具清單，同步加入 Claude、Codex 並比對順序（`src/components/SettingsView.tsx`）

## 4. 翻譯與驗證

- [x] 4.1 若工具標籤走 `t()`，於 `src/locales/zh-TW.ts` 與 `src/locales/en-US.ts` 補上 Claude、Codex 標籤鍵（既有工具標籤未使用 `t()`）
- [x] 4.2 `bun run build`（或 `npm run build`）確認前端型別檢查通過
- [x] 4.3 `cargo build`（於 `src-tauri`）確認後端編譯通過
- [x] 4.4 手動驗證：選單依序顯示八個工具，Claude / Codex 可正確在專案目錄開啟終端並執行對應 CLI，未安裝時標示不可用
