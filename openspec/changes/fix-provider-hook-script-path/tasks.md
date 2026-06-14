## 1. 收斂 hook 腳本根目錄解析

- [ ] 1.1 在 `codex.rs` 將 `ensure_codex_hook_scripts_installed` 由 `bundled_codex_hook_scripts_root()` 改為 `default_codex_hook_scripts_root()`
- [ ] 1.2 在 `copilot.rs` 將 `ensure_copilot_hook_scripts_installed` 由 `bundled_copilot_hook_scripts_root()` 改為 `default_copilot_hook_scripts_root()`
- [ ] 1.3 確認 `render_codex_integration` / `render_copilot_integration` 取得腳本路徑的來源與 1.1、1.2 一致（同為 `default_*_hook_scripts_root`）
- [ ] 1.4 移除 `settings.rs` 中 `bundled_codex_hook_scripts_root` / `bundled_copilot_hook_scripts_root` 的使用點；若無其他引用則一併刪除函式定義

## 2. 清除舊版內嵌 hook group

- [ ] 2.1 強化 `codex.rs` 的 `is_sessionhub_hook_group`，使其除新版 `# sessionhub-provider-event-bridge` marker 外，亦辨識舊版內嵌特徵（`command` / `commandWindows` 含 `provider = 'codex'`）
- [ ] 2.2 對 `copilot.rs` 套用對應的舊版 group 辨識強化
- [ ] 2.3 確認安裝/更新流程的 `retain(|g| !is_sessionhub_hook_group(g))` 能同時清除新舊 group，維持每事件單一 group

## 3. 清理殘留與驗證

- [ ] 3.1 刪除專案根目錄的 `bash.exe.stackdump`
- [ ] 3.2 執行 `cargo test` 確認 codex / copilot 整合相關測試通過（含路徑與 group 清理斷言）
- [ ] 3.3 手動驗證：重新安裝 Codex 整合後，`~/.codex/hooks/` 下腳本檔存在、`hooks.json` 每事件僅一個 SessionHub group、觸發事件不再回報 `hook exited with code 1`
- [ ] 3.4 手動驗證：掃描期間 UI 維持回應，未再產生 `bash.exe.stackdump`
