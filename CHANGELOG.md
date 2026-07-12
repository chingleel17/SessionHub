# Changelog

本專案的所有重要變更都記錄在此檔案。

格式依循 [Keep a Changelog](https://keepachangelog.com/zh-TW/1.1.0/)，版本號依循 [Semantic Versioning](https://semver.org/lang/zh-TW/)。

每個版本的變更分為以下類別（沒有內容的類別可省略）：

- **新增**：全新的功能
- **調整**：既有功能的行為變更、重構或體驗改進
- **修正**：錯誤修復
- **移除**：被拿掉的功能

## [Unreleased]

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

[Unreleased]: https://github.com/chingleel17/SessionHub/compare/v0.1.6...HEAD
[0.1.6]: https://github.com/chingleel17/SessionHub/releases/tag/v0.1.6
