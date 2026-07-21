## 1. 收斂 hook 腳本根目錄解析

- [x] 1.1 在 `codex.rs` 將 `ensure_codex_hook_scripts_installed` 由 `bundled_codex_hook_scripts_root()` 改為 `default_codex_hook_scripts_root()`
- [x] 1.2 在 `copilot.rs` 將 `ensure_copilot_hook_scripts_installed` 由 `bundled_copilot_hook_scripts_root()` 改為 `default_copilot_hook_scripts_root()`
- [x] 1.3 確認 `render_codex_integration` / `render_copilot_integration` 取得腳本路徑的來源與 1.1、1.2 一致（同為 `default_*_hook_scripts_root`）
- [x] 1.4 移除 `settings.rs` 中 `bundled_codex_hook_scripts_root` / `bundled_copilot_hook_scripts_root` 的使用點；若無其他引用則一併刪除函式定義

## 2. 清除舊版內嵌 hook group

- [x] 2.1 強化 `codex.rs` 的 `is_sessionhub_hook_group`，使其除新版 `# sessionhub-provider-event-bridge` marker 外，亦辨識舊版內嵌特徵（`command` / `commandWindows` 含 `provider = 'codex'`）
- [x] 2.2 對 `copilot.rs` 套用對應的舊版 group 辨識強化（Copilot 整合採全量覆寫，無 retain 流程，路徑修正即完成）
- [x] 2.3 確認安裝/更新流程的 `retain(|g| !is_sessionhub_hook_group(g))` 能同時清除新舊 group，維持每事件單一 group

## 3. 統一三 provider 安裝位置策略

- [x] 3a.1 將 `resolve_effective_hook_scripts_root` 改為「自訂路徑優先，否則 default 原生目錄」，移除 bundled fallback
- [x] 3a.2 `ensure_claude_hook_scripts_installed` 接收 `hook_scripts_path` 並寫到 `resolve_effective_hook_scripts_root` 同一目錄（取代寫死 `bundled_hook_scripts_root`）；同步更新 lib.rs 啟動呼叫傳入 `settings.hook_scripts_path`
- [x] 3a.3 移除 `settings.rs` 中 `bundled_hook_scripts_root` 函式（已無引用）

## 3b. uninstall 只刪除受管 hook 腳本（保留使用者自訂檔案）

- [x] 3b.1 新增共用 `uninstall_hook_scripts`：逐一刪除 `entries` 列出的受管檔案與 `.version`，僅在 `modules/`、root 變空時才移除目錄；絕不 `remove_dir_all`
- [x] 3b.2 Claude `remove_hook_scripts` 改呼叫 `uninstall_hook_scripts`，刪 effective（原生）目錄中的受管檔
- [x] 3b.3 Codex `remove_codex_hook_scripts` 改呼叫 `uninstall_hook_scripts`，刪 `~/.codex/hooks` 受管檔
- [x] 3b.4 Copilot `remove_copilot_hook_scripts` 改呼叫 `uninstall_hook_scripts`，刪 `~/.copilot/hooks` 受管檔
- [x] 3b.5 新增測試：uninstall 移除受管檔但保留使用者自訂檔案、空目錄才清除

## 4. 清理殘留與驗證

- [x] 3.1 刪除專案根目錄的 `bash.exe.stackdump`
- [x] 3.2 執行 `cargo test` 確認 codex / copilot 整合相關測試通過（含路徑與 group 清理斷言）
- [x] 3.3 手動驗證：重新安裝 Codex 整合後，`~/.codex/hooks/` 下腳本檔存在、`hooks.json` 每事件僅一個 SessionHub group、觸發事件不再回報 `hook exited with code 1`
- [x] 3.4 手動驗證：掃描期間 UI 維持回應，未再產生 `bash.exe.stackdump`
