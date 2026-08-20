## 1. Provider 可用性與響應式版面

- [x] 1.1 將 provider integration 標題與操作改為可回退版面，並驗證窄欄寬下名稱、狀態與按鈕不溢出
- [x] 1.2 為已偵測 provider 加入純成功圖示，未偵測時留空並套用 disabled 樣式，且驗證根目錄編輯仍可操作
- [x] 1.3 將 provider 可用性套用至平台整合管理，並驗證未偵測項目的安裝、更新、解除安裝停用，而重新檢查與修復入口可用
- [x] 1.4 將 provider 可用性套用至 quota 與 overlay provider 選項，並驗證未偵測項目保留可見但無法切換
- [x] 1.5 將平台啟用、純圖示偵測狀態與根目錄編輯整合至平台整合管理，並移除一般設定的重複平台清單
- [x] 1.6 在整合列固定顯示工具根目錄與選擇資料夾操作，並將選填 plugin、hook、bridge 路徑改為無外框純文字呈現
- [x] 1.7 將工具根目錄移入展開內容，並以共用淡色路徑與 icon-only 選擇器樣式統一專案內既有路徑 UI
- [x] 1.8 建立共用 provider 顯示順序，並套用至整合管理、Quota、Dashboard、Project、Agents 與 MCP 清單

## 2. 進階設定分組

- [x] 2.1 新增預設收折的進階設定 disclosure，移入指定的五項低頻設定，並驗證展開後原值與操作保持不變
- [x] 2.2 補齊進階設定與可用性狀態的 zh-TW、en-US 文案，並驗證兩份 locale 型別與 build 通過
- [x] 2.3 將進階設定的檔案與資料夾瀏覽操作改為 icon-only 按鈕，並保留無障礙標籤與 tooltip
- [x] 2.4 重排一般設定的啟動與通知開關，將終端啟動器移至預設開啟工具上方，並把封存與狀態列開關移至進階設定最前方
- [x] 2.5 讓 Herdr 偵測與執行在 PATH 之外回退至 Windows 標準安裝位置，避免安裝版程序繼承舊環境後誤判

## 3. 驗證

- [x] 3.1 執行 `openspec validate improve-settings-responsive-availability --strict`，確認 change artifacts 通過嚴格驗證
- [x] 3.2 執行 `bun run lint` 與 `bun run build`，確認前端型別、lint 與 production build 通過
- [x] 3.3 以桌面與窄視窗、淺色與深色主題檢查設定頁，確認收折、disabled 對比與響應式排列符合規格
