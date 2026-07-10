## 1. 後端：資料根目錄解析納入環境變數

- [ ] 1.1 `settings.rs` 的 `default_claude_root` / `default_codex_root` / `default_copilot_root` / `default_opencode_root` 加入官方環境變數判斷（`CLAUDE_CONFIG_DIR` / `CODEX_HOME` / `COPILOT_HOME` / `XDG_DATA_HOME`），優先序：手動設定 > 環境變數 > USERPROFILE 預設
- [ ] 1.2 新增路徑來源列舉（預設 / 環境變數 / 手動設定）與對應解析函式，供現況檢視回報來源
- [ ] 1.3 為解析優先序撰寫單元測試（含 `XDG_DATA_HOME` 的 `\opencode` 子路徑組合）

## 2. 後端：資料位置現況與搬遷 commands

- [ ] 2.1 新增 `src-tauri/src/commands/data_location.rs`：`get_data_locations` command 回傳各已啟用 provider 與 SessionHub 自身的路徑、來源、是否存在
- [ ] 2.2 新增 `get_directory_size` async command（walkdir 遞迴計算，回傳檔案數與 bytes）
- [ ] 2.3 新增 `migrate_data_location` async command：前置檢查（目的地磁碟可用空間）、遞迴複製、批次 emit `data-migration-progress` 事件、複製後檔案數/大小驗證
- [ ] 2.4 實作取消機制：取消時停止複製並清除目的地已複製內容
- [ ] 2.5 實作使用者層級環境變數寫入（`HKCU\Environment` + 廣播 `WM_SETTINGCHANGE`）；`XDG_DATA_HOME` 已設定為其他值時回傳不可自動處理的明確錯誤
- [ ] 2.6 搬遷成功後同步更新 AppSettings 對應 root 欄位並持久化
- [ ] 2.7 在 `commands/mod.rs` 與 `lib.rs` 註冊新 commands
- [ ] 2.8 為複製驗證、取消清理、環境變數寫入邏輯撰寫單元測試

## 3. 前端：資料位置區塊 UI

- [ ] 3.1 `App.tsx` 新增 IPC 呼叫（`get_data_locations` / `get_directory_size` / `migrate_data_location`）與 `data-migration-progress` 事件監聽
- [ ] 3.2 設定頁新增「資料位置」區塊元件（純顯示元件）：列出各項目的路徑、來源標示、大小（loading 狀態、不存在顯示）
- [ ] 3.3 新增搬遷引導對話框：選擇新目錄、前置提示（關閉 CLI 工具、磁碟空間）、進度條與取消、完成提示（重開終端機 / 可刪舊目錄 / 憑證重新登入 / SessionHub override 需重啟）
- [ ] 3.4 opencode 項目顯示 `XDG_DATA_HOME` 共用變數警示；Claude 項目標註 `CLAUDE_CONFIG_DIR` 已知 IDE 整合限制
- [ ] 3.5 新增所有 UI 文案的翻譯 key

## 4. 驗證

- [ ] 4.1 `cargo test` 與前端 build 通過
- [ ] 4.2 實機驗證：以測試目錄執行完整搬遷流程（含取消與磁碟空間不足案例），確認環境變數寫入與 SessionHub 掃描改讀新位置
- [ ] 4.3 實機驗證 `COPILOT_HOME` 與 `XDG_DATA_HOME` 對目前安裝版本的 CLI 工具實際生效（design.md Open Questions）
