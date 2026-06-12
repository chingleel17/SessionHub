## 1. Provider Integration Backend

- [x] 1.1 新增 `provider/codex.rs`，實作 Codex integration 的路徑解析、安裝、更新與狀態檢查
- [x] 1.2 擴充 provider integration 聚合流程，讓 `collect_provider_integration_statuses`、install/update/recheck commands 支援 Codex
- [x] 1.3 將 Copilot、OpenCode、Codex 的 integration 內容拆成各自獨立管理的模板或腳本來源，保留共用 helper 在 `provider/mod.rs`

## 2. Bridge And Watcher Wiring

- [x] 2.1 擴充 provider bridge refresh event 映射，補上 Codex 的 `codex-sessions-updated` 事件名稱與 bridge diagnostics 流程
- [x] 2.2 調整 `restart_session_watcher_internal`，讓 Copilot、OpenCode、Codex 都先依 integration 狀態決定 bridge 或 filesystem watcher
- [x] 2.3 保留 Codex filesystem fallback watcher，並確保 bridge installed 時不會與 fallback watcher 造成重複刷新

## 3. Settings And UI Integration

- [x] 3.1 更新 settings 載入與型別，確保 provider integration 狀態回傳包含 Codex 與相關路徑資訊
- [x] 3.2 更新 `App.tsx` 的 provider integration action 流程，讓 Codex 安裝、更新、重新檢查走同一套 mutation 與 toast 邏輯
- [x] 3.3 更新 `SettingsView.tsx` 與 i18n 文案，讓 provider integration 區塊明確涵蓋 Copilot hook、OpenCode plugin、Codex hook

## 4. Validation

- [x] 4.1 補上 Rust 測試，驗證 Codex integration 安裝、缺失、過舊與 bridge refresh event 名稱
- [x] 4.2 補上 watcher 相關測試，驗證 Codex bridge 可用時走 provider bridge、不可用時退回 filesystem watcher
- [x] 4.3 執行相關驗證指令並確認 OpenSpec change 進入可實作狀態
