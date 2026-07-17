## Why

SessionHub 的前端技術棧仍屬現代，但目前缺少可持續執行的前端 lint 機制，操作圖示分散於自製 SVG、元件內嵌 SVG、Emoji 與文字符號，且按鈕、下拉選單等重複互動元件各自定義樣式。隨著畫面與功能持續增加，這會導致視覺不一致、無障礙行為不一致，並提高後續維護成本。

## What Changes

- 新增輕量前端 lint 與可在本機、CI 執行的品質檢查指令。
- 建立統一的線性操作圖示系統，逐步取代分散的操作型 inline SVG、Emoji 與文字圖示；保留圖表、品牌與 quota 視覺化所需的專用 SVG。
- 建立可重用的 Button、IconButton、Select/Dropdown 與 Tooltip 基礎元件，統一尺寸、狀態、鍵盤操作與設計 token 使用方式。
- 將既有頁面遷移至共用元件，避免新增功能繼續複製按鈕、選單與圖示樣式。
- 將圖示與共用元件規則記錄於前端貢獻規範，讓 AI 與人工審查可一致套用。

## Capabilities

### New Capabilities
- `frontend-quality-gates`: 前端 lint 與本機、CI 的可重現品質檢查。
- `ui-primitives`: Button、IconButton、Select/Dropdown、Tooltip 的共用互動與視覺基礎。
- `icon-system`: 統一操作與導覽圖示來源、尺寸與無障礙規則。

### Modified Capabilities
- `action-icons`: Session 操作圖示改為遵循統一圖示與共用 IconButton 規格，並維持 tooltip 與既有操作行為。

## Impact

- 前端：`package.json`、Vite/前端工具設定、`src/components/`、`src/App.css`、theme token，以及使用重複按鈕、選單或圖示的畫面。
- CI：`.github/workflows/ci.yml` 新增前端 lint 檢查。
- 文件：`CONTRIBUTING.md`、`AGENTS.md` 與前端元件規範。
- 相依性：新增一個輕量、可 tree-shake 的圖示套件與 lint 工具；不導入完整 UI framework，不改變 Tauri IPC 架構。
