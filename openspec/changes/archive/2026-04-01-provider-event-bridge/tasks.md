## 1. Provider bridge backend

- [x] 1.1 定義 provider integration 狀態型別、bridge record 結構與 bridge 檔案路徑解析
- [x] 1.2 實作 Copilot / OpenCode integration 狀態偵測與最後錯誤回傳
- [x] 1.3 實作安裝、更新、重新檢查 integration 的 Tauri commands
- [x] 1.4 實作 bridge 事件讀取、去重與 refresh 觸發流程

## 2. Settings UI integration

- [x] 2.1 擴充 settings 資料模型與 IPC，讓前端可取得 provider integration 狀態與路徑
- [x] 2.2 在設定頁新增 Copilot / OpenCode integration 區塊與狀態呈現
- [x] 2.3 加入安裝、更新、重新檢查、快速開啟與直接編輯操作
- [x] 2.4 補上對應 i18n 文案與錯誤提示

## 3. Watcher fallback hardening

- [x] 3.1 將 filesystem watcher 調整為只監看關鍵路徑或檔案
- [x] 3.2 根據 event kind 與 path 過濾無關事件
- [x] 3.3 在 emit 前加入 cheap verify，避免無效 refresh
- [x] 3.4 讓 bridge 與 watcher fallback 共存時具備短時間去重

## 4. Validation

- [x] 4.1 驗證已安裝 OpenCode plugin 時，session 結束後不再持續誤報更新
- [x] 4.2 驗證 provider integration 缺失時，設定頁正確顯示 fallback / manual_required 狀態
- [x] 4.3 執行 `bun run build`
- [x] 4.4 執行 `cargo build`
