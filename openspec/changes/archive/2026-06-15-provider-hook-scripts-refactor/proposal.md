## Why

目前 Copilot CLI 與 Codex 的 hook 整合方式是將整段 PowerShell 內嵌在 Rust 字串中，無獨立腳本檔案，維護困難且 Codex 在 Unix 上使用 `"true"` noop 導致應用程式未開啟時 hook 可能 block。透過將各 provider 的 hook 邏輯抽出為獨立腳本（參照 Claude 的做法），可統一維護模式、修復 Codex 的 blocking 問題，並為 Copilot 補齊 Unix/sh 支援。

## What Changes

- **新增 Codex hook 腳本目錄** `hooks/codex/`，包含 `.ps1` 與 `.sh` 腳本各一組（5 個事件），取代現有 Rust 字串內嵌方式
- **新增 Copilot CLI hook 腳本目錄** `hooks/copilot/`，補齊 `.sh` 版本（6 個事件），現有 `.ps1` 內嵌邏輯搬移至獨立腳本
- **修改 Codex 整合**：`render_codex_hook_command` 改為呼叫腳本檔案（`sh .codex/hooks/on-*.sh` / `pwsh ... on-*.ps1`），同時新增 `PreToolUse`、`UserPromptSubmit` 兩個事件（Codex 文件確認支援）
- **修改 Copilot 整合**：`render_copilot_hook_ps` 改為呼叫獨立腳本，新增 `command`（sh）欄位支援
- **腳本安裝機制擴充**：`ensure_claude_hook_scripts_installed` 拆分或擴充，新增 Codex/Copilot 腳本的安裝邏輯
- **腳本目錄路徑規則**：Claude 維持 `.claude/hooks/`，Codex 使用 `.codex/hooks/`，Copilot 使用 `.copilot/hooks/`（與各工具的設定目錄並列）

## Capabilities

### New Capabilities

- `codex-hook-scripts`：Codex provider 的獨立 hook 腳本集（.ps1 + .sh），涵蓋 SessionStart、PreToolUse、PostToolUse、UserPromptSubmit、Stop 五個事件，由 SessionHub 安裝至 `~/.codex/hooks/`
- `copilot-hook-scripts`：Copilot CLI provider 的獨立 hook 腳本集，補齊 sh 版本，涵蓋 sessionStart、sessionEnd、userPromptSubmitted、preToolUse、postToolUse、errorOccurred 六個事件，由 SessionHub 安裝至 `~/.copilot/hooks/`

### Modified Capabilities

- `provider-integration`：Codex 與 Copilot 的整合安裝流程需改為呼叫腳本路徑，而非內嵌命令字串；安裝時一併複製腳本檔案至對應目錄

## Impact

- `src-tauri/src/provider/codex.rs`：`render_codex_hook_command`、`managed_hook_group` 重寫；新增腳本安裝函式
- `src-tauri/src/provider/copilot.rs`：`render_copilot_hook_ps` 重寫為呼叫腳本；`managed_hook_group` 加入 `command` 欄位
- `src-tauri/src/provider/claude.rs`：`ensure_claude_hook_scripts_installed` 可能拆分為共用的 `ensure_hook_scripts_installed(provider, entries)`
- `src-tauri/src/settings.rs`：新增 `default_codex_hook_scripts_root`、`default_copilot_hook_scripts_root`、`bundled_codex_hook_scripts_root`、`bundled_copilot_hook_scripts_root`
- `src-tauri/src/lib.rs`：啟動時一併安裝 Codex/Copilot hook 腳本
- 新增原始腳本檔案：`hooks/codex/on-*.ps1`、`hooks/codex/on-*.sh`、`hooks/copilot/on-*.ps1`、`hooks/copilot/on-*.sh`、`hooks/shared/modules/`（共用模組）
- 現有 `.claude/hooks/` 結構不變
