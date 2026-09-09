# Changelog

本專案的所有重要變更都記錄在此檔案。

格式依循 [Keep a Changelog](https://keepachangelog.com/zh-TW/1.1.0/)，版本號依循 [Semantic Versioning](https://semver.org/lang/zh-TW/)。

每個版本的變更分為以下類別（沒有內容的類別可省略）：

- **新增**：全新的功能
- **調整**：既有功能的行為變更、重構或體驗改進
- **修正**：錯誤修復
- **移除**：被拿掉的功能

## [Unreleased]

## [0.1.10] - 2026-09-09

### 新增

- AGENTS.md、技能與指令顯示字元數與 Token 估算：後端新增 token_estimator 模組，支援中英混雜內容（CJK 1 字約 1 token、英文約 4 字 1 token），樹狀檢視、列表檢視與內容預覽皆呈現
- MCP HTTP server 連線測試功能，並強化 MCP server 編輯器：Header 逐列編輯與顯示/隱藏值切換、複製設定到其他工具、自訂 JSON 自動解析
- 側邊欄專案選擇器，可從未加入導覽的專案直接開啟或釘選；側邊欄支援拖曳重排，釘選區與開啟區各自排序並可將開啟項目拖曳至釘選區
- 系統匣配額面板顯示重置的確切日期時間，並加入 15 秒輪詢與即時重抓

### 調整

- 設定頁改以 provider 根目錄可用性整合各項操作，不可用時一併停用整合、quota 與安裝/移除動作；低頻選項收進「進階設定」收折區
- 新增共用的 provider 顯示順序，統一套用於整合、quota、MCP 與 Agents 等介面
- 系統匣圖示改為雙擊開啟主視窗、單擊切換 mini panel
- CLI 專屬資源（skills/commands）僅在全域範圍顯示，專案範圍自動過濾
- 主視窗最小寬度由 1280 調整為 1067，並修正工作區與狀態列的彈性佈局溢位
- 建立共用 Modal 容器元件，統一遮罩與 dialog 語意，遷移 EditDialog、Skills 預覽並新增 TagEditDialog
- 看板欄寬限制最小值，雙擊分隔線可重置為平均寬度，拖曳時維持總寬度

### 修正

- 釘選改以工作目錄作為專案身分，切換 git 分支後不再失效；同時去除重複項目並遷移舊格式設定
- 修正 session 中繼資料儲存後的快取不同步與增量掃描覆蓋問題
- MCP 連線測試改為驗證 JSON-RPC 回應，不再僅依 HTTP 狀態碼判定成功，避免誤填一般網頁端點時顯示連線正常
- dev build 不再註冊開機自動啟動，避免與正式版同時登記且手動移除後被寫回
- 修正 herdr 啟動模式下 tab 建立後看不到畫面的問題，並在 server 未執行時自動啟動並等待就緒
- Windows 下 CLI 程序啟動加入 CREATE_NO_WINDOW 旗標，避免主控台視窗閃爍
- Agents 載入狀態改用 isFetching，避免快取命中時仍顯示載入中

### 相依套件

- tauri-plugin-dialog 2.7.2 → 2.7.3、tauri-plugin-notification 2.3.3 → 2.4.0、tauri-plugin-opener 2.5.4 → 2.5.5、tauri-plugin-single-instance 2.4.3 → 2.4.4
- @tauri-apps/plugin-dialog 2.7.2 → 2.7.3

## [0.1.9] - 2026-08-19

### 新增

- herdr 終端啟動器：設定可選擇以系統 shell 或 herdr tab 開啟終端機，session 與工具終端改由統一的 launch_terminal 啟動並記錄 tab 對應，聚焦時自動切換至對應的 herdr tab
- Quota Overlay 新增 topmost 看門執行緒，定期重申視窗 Z-order，避免被其他視窗覆蓋

### 調整

- 設定頁顯示各工具目錄的偵測狀態與 herdr 可用性，終端機相關文字改為 i18n 翻譯

## [0.1.8] - 2026-07-17

### 調整

- 重製 Session 統計面板：以摘要列、模型明細與工具呼叫三區呈現；模型明細顯示輸入／輸出 Token 與可用成本，多模型提供合計列，工具呼叫預設顯示前五項並可展開
- Analytics 分頁移至 Agents 後方，控制列改為水平排列，快速日期範圍擴充為近一週、本週、近一個月與本月，並改善趨勢圖與圖例按鈕互動
- 側欄收折改為共用 DOM 與平滑過渡，修正收折過程中的圖示漂移、即時狀態綠點消失、釘選項目與已開啟項目的對齊問題
- 更新按鈕、下拉選單與 checkbox 的 hover／active 回饋，並改善 provider 標籤與 quota overlay 的視覺呈現

## [0.1.7] - 2026-07-16

### 調整

- 側欄收折展開改為平滑過渡動畫，收折按鈕改為固定位置（不再隨展開/收折移動），收折狀態下 icon 對齊一致，並修正收折後導覽區塊跑版、重新整理／版本號位置錯亂與版本號被截斷的問題
- Session 卡片與各處 provider 標籤的圖示改為可辨識的縮寫樣式，修正原本近乎空白的顯示問題
- 設定頁 quota 監控卡片的「立即刷新」改為圖示按鈕，與 Dashboard、系統匣面板一致
- 按鈕、下拉選單、checkbox 全面補上 hover／active 過渡動畫；primary、danger 按鈕的 hover 改為同色系加深，避免白字在淺色底上不易辨識
- 桌面 Quota Overlay 預設改為精簡版型（圓環一列）、預設不透明度調降為 30%，且首次啟用時預設定位於主螢幕右下角

## [0.1.6] - 2026-07-12

首個公開發布版本。SessionHub 是一個 Windows 桌面應用程式，統一管理多家 AI coding 工具的 sessions、用量配額與 hook 整合。

### 新增

- 多 provider session 管理：支援 GitHub Copilot CLI、OpenCode、Codex、Claude Code、Antigravity (Google Gemini) 五種 provider 的 session 掃描、搜尋、標籤、封存與刪除
- 用量配額監控（quota）：底部狀態列 SVG 圓環即時顯示、Dashboard 依模型群組（Gemini / Claude & GPT）分組顯示、系統匣 quota widget
- Hook 整合管理：一鍵安裝／偵測／解除安裝各 provider 的 SessionHub 事件橋接 hook，驅動即時狀態更新
- MCP 設定管理介面：GUI 檢視與編輯多平台 MCP server 設定
- Agents 設定管理：AGENTS.md / CLAUDE.md 雙檔源狀態徽章、Skills 與 Commands 瀏覽、設定搜尋
- Plans & Specs 瀏覽：檢視專案內 OpenSpec 變更（含 archive）與規格節點
- Session 統計分析：token 用量、工具呼叫、互動次數等圖表（SQLite 快取）
- 通知：agent 等待介入通知、session 結束通知
- 多語系介面（繁體中文／English）
- Minimal UI 設計系統：統一 design token、玻璃浮層、Linear 式分頁、自訂 scrollbar、`prefers-reduced-motion` 支援

[Unreleased]: https://github.com/chingleel17/SessionHub/compare/v0.1.10...HEAD
[0.1.10]: https://github.com/chingleel17/SessionHub/compare/v0.1.9...v0.1.10
[0.1.9]: https://github.com/chingleel17/SessionHub/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/chingleel17/SessionHub/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/chingleel17/SessionHub/releases/tag/v0.1.7
[0.1.6]: https://github.com/chingleel17/SessionHub/releases/tag/v0.1.6
