## 1. Settings And Provider Wiring

- [x] 1.1 在 Rust `AppSettings`、前端 `AppSettings` 型別與 settings.json 預設值加入 `codexRoot`
- [x] 1.2 擴充 `default_enabled_providers`、`get_sessions`、`restart_session_watcher` 相關參數與呼叫鏈，讓 Codex 可被啟用/停用
- [x] 1.3 更新設定頁表單、provider 勾選區與翻譯字串，加入 Codex root 與 Codex provider 文案

## 2. Codex Session Scanning

- [x] 2.1 新增 `src-tauri/src/sessions/codex.rs`，實作日期分層 `.jsonl` 檔案遞迴掃描
- [x] 2.2 解析 `session_meta` 與事件 `timestamp`，將 Codex 檔案映射為 `SessionInfo`
- [x] 2.3 將 Codex 掃描整合進 `sessions/mod.rs`，與 Copilot / OpenCode 合併排序與專案分組

## 3. Cache And Watchers

- [x] 3.1 擴充 `ProviderCache` / `ScanCache` 以支援 Codex 的 per-file mtime 增量快取
- [x] 3.2 在 `watcher.rs` 新增 Codex watcher，遞迴監看 `codexRoot/sessions` 的 `.jsonl` 變更
- [x] 3.3 確保 Codex watcher 與刷新去重邏輯不影響現有 Copilot / OpenCode 行為

## 4. UI Integration And Validation

- [x] 4.1 更新 provider label、provider filter、provider tag 與排序常數，讓 Codex 在列表與卡片中正確顯示
- [x] 4.2 檢查並處理不適用於 Codex 的 session actions / session-dir 假設，避免 UI 操作誤用 JSONL 路徑
- [x] 4.3 補上 Rust 與前端測試案例，驗證 Codex session 解析、設定預設值、provider 篩選與 watcher 刷新
