# Changelog

本專案的所有重要變更都記錄在此檔案。

格式依循 [Keep a Changelog](https://keepachangelog.com/zh-TW/1.1.0/)，版本號依循 [Semantic Versioning](https://semver.org/lang/zh-TW/)。

每個版本的變更分為以下類別（沒有內容的類別可省略）：

- **新增**：全新的功能
- **調整**：既有功能的行為變更、重構或體驗改進
- **修正**：錯誤修復
- **移除**：被拿掉的功能

## [Unreleased]

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

[Unreleased]: https://github.com/chingleel17/SessionHub/compare/v0.1.8...HEAD
[0.1.8]: https://github.com/chingleel17/SessionHub/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/chingleel17/SessionHub/releases/tag/v0.1.7
[0.1.6]: https://github.com/chingleel17/SessionHub/releases/tag/v0.1.6
