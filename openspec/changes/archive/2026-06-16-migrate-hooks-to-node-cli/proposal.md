## Why

目前每個 provider（claude / codex / copilot）的每個 hook 事件都維護兩份語意必須一致的腳本（`.ps1` + `.sh`），改一次 payload 結構要同步改十幾個檔案；且 `.sh` 那條路額外依賴 Git Bash 專屬的 `jq`、`bc` 與 GNU `date -d`，在 macOS 上時間戳處理還會悄悄走 fallback。改用單一 Node.js CLI 作為主軌可把主邏輯收斂成一份、用原生 `JSON.parse` 與 `Date.toISOString()` 消除外部依賴，而開發者使用 AI agent（Copilot CLI、Codex CLI 本身即 Node 生態）幾乎必有 Node 環境。

## What Changes

- 新增各 provider 的 `hooks/<provider>/*.js` 入口腳本與共用 `modules/record-event.js`，作為 hook 執行的**主軌**，以 `node <script.js>` 方式被 provider 呼叫。
- provider 設定檔（settings.json / hooks.json）的主 hook 命令改為產生 `node <script.js> --bridge-path ... --provider ...`。
- **BREAKING**：移除所有 `.ps1` hook 腳本與 Rust 端對應的 PowerShell 命令產生函式（`render_*_hook_command` 的 powershell 分支）與設定檔中的 `powershell` / `commandWindows` 欄位。Windows 改走 `node hook.js`。
- 保留 `.sh` 腳本作為**無 node 環境時的 fallback**，設定檔的 `command` 欄位維持 sh 版本。
- Rust 端：`include_str!` 改嵌入 `.js`、更新 `hook_script_entries()` 清單、`HOOK_SCRIPT_VERSION` 升版以觸發重新安裝。
- 不引入 Tauri sidecar；`.js` 沿用現有 `include_str!` + 寫檔至 provider 原生目錄的部署機制，零新增打包流程。

## Capabilities

### New Capabilities
- `node-hook-scripts`: Node.js CLI hook 主軌——各 provider 的 `*.js` 入口腳本與共用 `record-event.js` 模組，負責讀取 stdin payload、解析、寫入 bridge events.jsonl，作為 hook 執行的優先路徑。

### Modified Capabilities
- `sh-hook-scripts`: sh 腳本由主軌降為 fallback；設定檔中 sh 命令不再是唯一/主要路徑，且不再與 ps1 並存（ps1 移除）。
- `provider-integration`: hook 命令產生邏輯改為主軌 `node <script.js>`、fallback sh；移除 PowerShell / commandWindows 欄位與相關產生函式。

## Impact

- 程式碼：`hooks/{claude,codex,copilot}/`（新增 `.js`、移除 `.ps1`、保留 `.sh`）、`src-tauri/src/provider/{claude,codex,copilot}.rs`、`src-tauri/src/provider/mod.rs`（`install_hook_scripts`、`render_*_hook_command`、`hook_script_entries`）。
- 相依：主軌新增執行期相依 Node.js（裝有 Copilot/Codex CLI 者必備）；sh fallback 仍依賴 `jq`。
- 行為：升版後既有安裝的 hook 設定會在下次安裝/更新時被改寫為 node 主軌；Windows 不再需要 PowerShell 執行 hook。
- 既有 spec：`sh-hook-scripts`、`provider-integration` 需更新；`jq-dependency-check` 不變（sh fallback 仍需 jq）。
