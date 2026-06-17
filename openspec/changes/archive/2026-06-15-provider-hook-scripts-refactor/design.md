## Context

目前四個 provider 的 hook 整合方式不一致：

| Provider | 整合機制 | 腳本形式 | Unix 支援 |
|---|---|---|---|
| Claude | 呼叫獨立 `.ps1`/`.sh` | 分離檔案 | ✅ sh |
| Codex | PowerShell 內嵌字串 | Rust 字串 | ❌（noop `true`）|
| Copilot CLI | PowerShell 內嵌字串 | Rust 字串 | ❌ |
| OpenCode | TypeScript Plugin | TS 檔案 | ✅（Node.js）|

Codex 在 `commandWindows` 中內嵌 `[Console]::In.ReadToEnd()`，當 SessionHub 未執行時 Codex 呼叫 hook 可能因 stdin pipe 未關閉而無限 block。

## Goals / Non-Goals

**Goals:**
- Codex 改為獨立腳本，消除 block 問題，補齊 5 個 hook 事件
- Copilot CLI 改為獨立腳本，補齊 `.sh` 版本
- 各 provider 的腳本目錄對應各自的設定目錄（`.codex/hooks/`、`.copilot/hooks/`）
- 啟動時自動安裝 Codex/Copilot hook 腳本至 AppData 的 bundled 路徑
- 共用 `record-event.sh`/`record-event.psm1` 模組不重複撰寫

**Non-Goals:**
- OpenCode 維持 TypeScript Plugin 機制，不納入此次重構
- `.claude/hooks/` 目錄結構不變
- 不修改 bridge 處理邏輯（Rust 後端 watcher/bridge.rs）

## Decisions

### 決策 1：各 provider 各自獨立的腳本目錄

**選擇**：`hooks/claude/`、`hooks/codex/`、`hooks/copilot/` 各自獨立（原始碼層），安裝至 `~/.claude/hooks/`、`~/.codex/hooks/`、`~/.copilot/hooks/`

**理由**：與各工具的設定目錄慣例對齊，且用戶可在各工具的設定目錄中找到腳本，不會混淆。

**替代方案**：全部放 `hooks/shared/` 再複製 → 共用邏輯難以個別調整 payload 欄位。

### 決策 2：共用模組透過 `include_str!` 從 `.claude/hooks/modules/` 讀取

**選擇**：Codex/Copilot 腳本也 `source` 相同的 `record-event.sh`/`record-event.psm1`，安裝時一併複製 `modules/` 到各 provider 的 hooks 目錄

**理由**：避免 `record-event` 邏輯出現三份。安裝時各目錄各有一份 modules，腳本用相對路徑 `source "$SCRIPT_DIR/modules/record-event.sh"`，不依賴跨目錄路徑。

### 決策 3：Codex 設定檔路徑

Codex hook 設定位於 `~/.codex/hooks.json`（非 `config.json`），格式與 Claude `settings.json` 的 hooks 區塊相同（`"hooks": { "EventName": [...] }`）。`src-tauri/src/settings.rs` 需新增 `resolve_codex_hooks_path()`。

### 決策 4：Codex 新增事件 PreToolUse 與 UserPromptSubmit

Codex 文件確認這兩個事件存在，payload 欄位：
- `PreToolUse`：`session_id`, `cwd`, `tool_name`, `tool_use_id`, `tool_input`
- `UserPromptSubmit`：`session_id`, `cwd`, `turn_id`, `prompt`

現有 Codex 整合只有 3 個事件，擴充至 5 個（與 Claude 對齊）。

### 決策 5：腳本安裝拆分

`ensure_claude_hook_scripts_installed()` 拆成通用的 `install_hook_scripts(entries, root)` helper，各 provider 各自呼叫。啟動時（`lib.rs`）依序安裝 Claude、Codex、Copilot 三組腳本。

## Risks / Trade-offs

- **[Risk] Codex hooks.json 格式與 config.json 不同** → Codex `render_codex_integration` 現在讀寫 `config.json`，需確認實際設定檔路徑。文件指出 `~/.codex/hooks.json`，需同步修改 `resolve_codex_integration_path`。
- **[Risk] 腳本安裝目錄權限** → Windows 上 `~/.codex/hooks/` 由 SessionHub 建立，若 Codex 設定目錄不存在則需先建立。安裝函式已有 `ensure_parent_dir` 處理。
- **[Risk] Copilot CLI 的 sh 腳本在 Windows 上的 sh 路徑** → 已有前例（Claude 的 `sh` 呼叫），直接沿用相同方式。

## Migration Plan

1. 新增腳本原始檔案（`hooks/codex/`、`hooks/copilot/`）
2. 修改 Rust provider 檔案重寫 render 函式
3. 修改 `lib.rs` 啟動時安裝三組腳本
4. 測試：安裝整合後確認腳本存在且可執行
5. 無需 rollback 機制（腳本為附加，不影響現有資料）
