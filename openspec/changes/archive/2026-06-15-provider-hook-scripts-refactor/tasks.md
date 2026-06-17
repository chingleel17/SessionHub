## 1. 新增 Codex hook 腳本原始檔案

- [x] 1.1 建立 `hooks/codex/modules/` 目錄，複製（或 symlink 參考）`record-event.sh` 與 `record-event.psm1` 共用模組內容
- [x] 1.2 建立 `hooks/codex/on-session-start.sh`：讀取 stdin，解析 `session_id`、`cwd`、`source`，寫入 bridge record（eventType: `session.started`，title: `source`）
- [x] 1.3 建立 `hooks/codex/on-session-start.ps1`：同上邏輯的 PowerShell 版本
- [x] 1.4 建立 `hooks/codex/on-pre-tool-use.sh`：解析 `tool_name`，寫入 bridge record（eventType: `tool.pre`，title: `tool_name`）
- [x] 1.5 建立 `hooks/codex/on-pre-tool-use.ps1`
- [x] 1.6 建立 `hooks/codex/on-post-tool-use.sh`：解析 `tool_name`，寫入 bridge record（eventType: `tool.post`，title: `tool_name`）
- [x] 1.7 建立 `hooks/codex/on-post-tool-use.ps1`
- [x] 1.8 建立 `hooks/codex/on-user-prompt-submit.sh`：解析 `prompt`（截斷至 80 字元），寫入 bridge record（eventType: `prompt.submitted`）
- [x] 1.9 建立 `hooks/codex/on-user-prompt-submit.ps1`
- [x] 1.10 建立 `hooks/codex/on-stop.sh`：解析 `stop_reason`，寫入 bridge record（eventType: `session.stop`，title: `stop_reason`）
- [x] 1.11 建立 `hooks/codex/on-stop.ps1`

## 2. 新增 Copilot CLI hook 腳本原始檔案

- [x] 2.1 建立 `hooks/copilot/modules/` 目錄，複製共用模組
- [x] 2.2 建立 `hooks/copilot/on-session-start.sh`：解析 `cwd`、`timestamp`（毫秒轉 ISO），寫入 bridge record（eventType: `session.started`）
- [x] 2.3 建立 `hooks/copilot/on-session-start.ps1`：同上邏輯的 PowerShell 版本（現有內嵌邏輯搬移）
- [x] 2.4 建立 `hooks/copilot/on-session-end.sh`：解析 `reason`，若為 `error` 填入 error 欄位，寫入 bridge record（eventType: `session.ended`）
- [x] 2.5 建立 `hooks/copilot/on-session-end.ps1`
- [x] 2.6 建立 `hooks/copilot/on-user-prompt-submitted.sh`：解析 `prompt`（截斷 80 字元），寫入 bridge record（eventType: `prompt.submitted`）
- [x] 2.7 建立 `hooks/copilot/on-user-prompt-submitted.ps1`
- [x] 2.8 建立 `hooks/copilot/on-pre-tool-use.sh`：解析 `toolName`，寫入 bridge record（eventType: `tool.pre`）
- [x] 2.9 建立 `hooks/copilot/on-pre-tool-use.ps1`
- [x] 2.10 建立 `hooks/copilot/on-post-tool-use.sh`：解析 `toolName`、`toolResult.resultType`，寫入 bridge record（eventType: `tool.post`，failure/denied 時填 error）
- [x] 2.11 建立 `hooks/copilot/on-post-tool-use.ps1`
- [x] 2.12 建立 `hooks/copilot/on-error-occurred.sh`：解析 `error.message`，寫入 bridge record（eventType: `session.errored`）
- [x] 2.13 建立 `hooks/copilot/on-error-occurred.ps1`

## 3. 修改 Rust：Codex provider

- [x] 3.1 在 `src-tauri/src/provider/codex.rs` 新增 `include_str!` 載入 `hooks/codex/` 下所有腳本（參照 claude.rs 模式）
- [x] 3.2 新增 `ensure_codex_hook_scripts_installed()` 函式，將腳本寫入 `%APPDATA%\SessionHub\.codex\hooks\`
- [x] 3.3 新增 `default_codex_hook_scripts_root()` 與 `bundled_codex_hook_scripts_root()` 至 `settings.rs`
- [x] 3.4 重寫 `render_codex_hook_command()`：改為 `sh /path/to/on-xxx.sh --bridge-path ... --provider codex`（sh）與 `pwsh -File on-xxx.ps1 -BridgePath ...`（Windows）
- [x] 3.5 修改 `managed_hook_group()`：加入 `command`（sh）欄位，與 `commandWindows`（ps1）並列
- [x] 3.6 擴充 `render_codex_integration()` 的 `managed_groups` 陣列，新增 `PreToolUse`（無 matcher）與 `UserPromptSubmit` 兩個事件
- [x] 3.7 確認 `resolve_codex_integration_path()` 指向 `~/.codex/hooks.json`（非 `config.json`），如有差異則修正

## 4. 修改 Rust：Copilot CLI provider

- [x] 4.1 在 `src-tauri/src/provider/copilot.rs` 新增 `include_str!` 載入 `hooks/copilot/` 下所有腳本
- [x] 4.2 新增 `ensure_copilot_hook_scripts_installed()` 函式
- [x] 4.3 新增 `default_copilot_hook_scripts_root()` 與 `bundled_copilot_hook_scripts_root()` 至 `settings.rs`
- [x] 4.4 重寫 `render_copilot_hook_ps()` 為 `render_copilot_hook_command()`：ps1 與 sh 腳本路徑各一，不再內嵌邏輯字串
- [x] 4.5 修改 `render_copilot_integration()` 中的 hook entry：`powershell` 欄位改為腳本路徑呼叫，新增 `command` 欄位（sh 版本）

## 5. 修改 Rust：啟動安裝流程

- [x] 5.1 在 `src-tauri/src/lib.rs` 的 setup 函式中，在 `ensure_claude_hook_scripts_installed()` 之後依序呼叫 `ensure_codex_hook_scripts_installed()` 與 `ensure_copilot_hook_scripts_installed()`

## 6. 驗證

- [x] 6.1 啟動應用程式，確認 `%APPDATA%\SessionHub\.codex\hooks\` 與 `%APPDATA%\SessionHub\.copilot\hooks\` 均已建立且包含正確腳本
- [x] 6.2 安裝 Codex integration，確認 `~/.codex/hooks.json` 的 hook command 為腳本路徑形式，且涵蓋 5 個事件
- [x] 6.3 安裝 Copilot integration，確認 `~/.copilot/settings.json` 的 hook command 為腳本路徑形式，且新增 `command`（sh）欄位
- [x] 6.4 手動執行 sh 腳本（傳入測試 JSON）確認可正確寫入 bridge JSONL
