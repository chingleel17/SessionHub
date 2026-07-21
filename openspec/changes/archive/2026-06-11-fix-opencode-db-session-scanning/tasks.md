## 1. OpenCode Session Source

- [x] 1.1 重構 `sessions/opencode.rs`，改以 `opencode.db` 的 `session` / `project` 資料表作為主要 session 掃描來源
- [x] 1.2 保留舊 JSON storage 掃描作 fallback，確保舊版 OpenCode 仍可被讀取
- [x] 1.3 更新 OpenCode session metadata 對映，確保 DB row 能正確轉成 `SessionInfo`

## 2. Watcher And Refresh

- [x] 2.1 調整 OpenCode fallback watcher，改監看 `opencode.db` 與 `opencode.db-wal`
- [x] 2.2 更新 OpenCode watch snapshot / cheap verify 邏輯，使其與 DB-based storage 一致
- [x] 2.3 驗證 bridge refresh 後，DB-only session 能正確寫入 `sessions_cache` 並出現在列表

## 3. Stats Compatibility

- [x] 3.1 檢查並修正 OpenCode 最新 session 的 stats / message 來源定位邏輯
- [x] 3.2 確保單一 OpenCode session stats 失敗時，不影響 session 列表顯示

## 4. Validation

- [x] 4.1 補上 DB-only OpenCode session 的 Rust 測試案例
- [x] 4.2 補上 watcher / refresh 測試，驗證 DB 與 WAL 變更可觸發 OpenCode 更新
- [x] 4.3 執行相關驗證指令並確認 OpenSpec change 進入可實作狀態
