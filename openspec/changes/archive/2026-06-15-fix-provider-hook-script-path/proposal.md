## Why

Codex 整合安裝後，每次事件觸發都回報 `hook exited with code 1`，並在掃描時造成 UI 無回應甚至當機。根因是 provider hook 腳本「實際寫出的位置」與「寫進 codex `hooks.json` 指令所指向的位置」不一致，導致 codex 找不到 `.sh` 腳本而失敗；失敗的 hook 又與遞迴 watcher 形成回授迴圈，拖垮掃描流程。此問題阻擋 codex / copilot 的 hook 驅動活動狀態功能，須立即修正。

## What Changes

- 修正 `ensure_codex_hook_scripts_installed` 與 `ensure_copilot_hook_scripts_installed`：腳本實際寫出的根目錄改為與 `hooks.json` 指令所引用的根目錄一致（即 `default_*_hook_scripts_root`，`~/.codex/hooks`、`~/.copilot/hooks`），不再使用指向 `%APPDATA%\SessionHub\.*\hooks` 的 `bundled_*_hook_scripts_root`。
- 統一安裝與寫入 config 兩處對 hook 腳本根目錄的解析來源，確保兩者永遠指向同一路徑，避免再次發生路徑分歧。
- 強化 `is_sessionhub_hook_group` 的判定：除了新版 `# sessionhub-provider-event-bridge` marker，也需辨識並清除舊版 v4 PowerShell 內嵌 group（特徵為 `provider = 'codex'` 等內嵌指令），避免新舊 group 並存於 `hooks.json`。
- 保留 sh 與 ps1 雙版本 hook 腳本（不移除 sh），僅修正路徑一致性與舊 group 清理。
- 移除遺留的 `bash.exe.stackdump` 崩潰殘留檔。

## Capabilities

### New Capabilities
<!-- 無新增能力，純屬修正既有行為 -->

### Modified Capabilities
- `codex-provider-integration`: hook 腳本安裝路徑須與 `hooks.json` 指令引用路徑一致；安裝與更新時須清除舊版內嵌 hook group，避免新舊並存。
- `sh-hook-scripts`: 明確界定 sh / ps1 hook 腳本的安裝根目錄為 provider 設定根目錄下的 `hooks/`，且與整合檔指令引用路徑為同一來源。

## Impact

- 受影響程式碼：
  - `src-tauri/src/provider/codex.rs`（`ensure_codex_hook_scripts_installed`、`render_codex_integration`、`is_sessionhub_hook_group`）
  - `src-tauri/src/provider/copilot.rs`（對應安裝與 group 清理邏輯）
  - `src-tauri/src/settings.rs`（`default_*_hook_scripts_root` / `bundled_*_hook_scripts_root` 的使用收斂）
- 受影響使用者資料：`~/.codex/hooks.json`、`~/.codex/hooks/`、`~/.copilot/hooks/`（重新安裝後路徑歸位）
- 無對外 API 變動；屬向後相容的修正（會清理既有錯誤設定）。
