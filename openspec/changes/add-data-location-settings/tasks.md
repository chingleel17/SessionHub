## 1. 後端：symlink 權限偵測與現況解析

- [ ] 1.1 新增 symlink 建立權限偵測函式：檢查是否以系統管理員身分執行，或登錄機碼 `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock\AllowDevelopmentWithoutDevLicense` 是否為 1，任一為真即視為具備權限
- [ ] 1.2 新增判斷路徑是否為 symlink 及讀取連結目標的函式（`std::fs::symlink_metadata` + `read_link`）
- [ ] 1.3 為權限偵測與 symlink 判斷函式撰寫單元測試

## 2. 後端：資料位置現況與搬遷 commands

- [ ] 2.1 新增 `src-tauri/src/commands/data_location.rs`：`get_data_locations` command 回傳各已啟用 provider 與 SessionHub 自身的路徑、是否為 symlink（含連結目標）、是否存在
- [ ] 2.2 新增 `get_directory_size` async command（walkdir 遞迴計算，回傳檔案數與 bytes）
- [ ] 2.3 新增 `check_symlink_permission` command，回傳是否具備建立 symlink 的能力
- [ ] 2.4 新增 `migrate_data_location` async command：前置檢查（symlink 權限、目的地磁碟可用空間）、遞迴複製、批次 emit `data-migration-progress` 事件、複製後檔案數/大小驗證、驗證通過後將原目錄 rename 為 `.bak`、於原路徑建立 symlink 指向新目的地
- [ ] 2.5 實作取消機制：取消時停止複製並清除目的地已複製內容，不 rename 原目錄、不建 symlink
- [ ] 2.6 實作 symlink 建立失敗時的回復：若 rename 後建立 symlink 失敗，立即將 `.bak` 改回原名
- [ ] 2.7 在 `commands/mod.rs` 與 `lib.rs` 註冊新 commands
- [ ] 2.8 為複製驗證、取消清理、symlink 建立與失敗回復邏輯撰寫單元測試

## 3. 前端：資料位置區塊 UI

- [ ] 3.1 `App.tsx` 新增 IPC 呼叫（`get_data_locations` / `get_directory_size` / `check_symlink_permission` / `migrate_data_location`）與 `data-migration-progress` 事件監聽
- [ ] 3.2 設定頁新增「資料位置」區塊元件（純顯示元件）：列出各項目的路徑、是否為 symlink 標示（含連結目標）、大小（loading 狀態、不存在顯示）
- [ ] 3.3 新增搬遷引導對話框：選擇新目錄、前置提示（關閉 CLI 工具、磁碟空間）、無 symlink 權限時顯示中止訊息與「開啟開發人員模式」路徑/指令（`start ms-settings:developers`）、進度條與取消、完成提示（可自行刪除 `.bak` 備份 / 換電腦時於同一同步路徑重建 symlink 即可接續使用 / 憑證可能需重新登入）
- [ ] 3.4 新增所有 UI 文案的翻譯 key

## 4. 驗證

- [ ] 4.1 `cargo test` 與前端 build 通過
- [ ] 4.2 實機驗證：以測試目錄執行完整搬遷流程（含取消、磁碟空間不足、無 symlink 權限案例），確認 symlink 建立後 SessionHub 掃描與各 CLI 工具讀寫均正常
- [ ] 4.3 實機驗證雲端同步工具（OneDrive 等）對 symlink 佈局的同步行為（design.md Open Questions）
