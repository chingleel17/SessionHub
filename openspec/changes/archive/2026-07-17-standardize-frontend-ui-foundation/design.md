## Context

SessionHub 使用 React 19、TypeScript、Vite 與純 CSS，現有 `Icons.tsx` 已提供一部分線性 SVG，但仍有元件內嵌 SVG、Emoji 與文字圖示。按鈕與下拉選單則散落在各元件與 `App.css`，造成尺寸、圓角、hover、focus 與 tooltip 行為不一致。專案已具備 CSS theme token 與元件純顯示層慣例，適合以小型基礎元件逐步收斂，而非置換整套 UI。

## Goals / Non-Goals

**Goals:**

- 以可 tree-shake 的線性 icon library 統一操作與導覽圖示。
- 建立不耦合 Tauri IPC 的 Button、IconButton、Select/Dropdown、Tooltip 基礎元件。
- 將設計 token 套用到共用互動元件，並遷移高頻使用畫面。
- 加入可本機執行且可在 CI 阻擋違規的前端 lint。
- 保留既有 i18n、React Query、App.tsx IPC 集中與純 CSS 架構。

**Non-Goals:**

- 不導入完整 UI framework、Tailwind、CSS Modules 或全域狀態管理套件。
- 不重畫 Dashboard、Charts、Quota rings、品牌 logo 或 provider 品牌資產。
- 不在此 change 變更後端 command、資料庫 schema 或 provider 資料解析。
- 不要求一次遷移所有頁面的版面結構；未使用共用 primitive 的低風險視覺化 SVG 可保留。

## Decisions

### 採用 Lucide React 作為操作圖示唯一來源

`lucide-react` 提供 tree-shake 的 outline SVG 與完整 TypeScript API，風格接近現有 `Icons.tsx`。建立集中 re-export module 與 `ProviderIcon` 元件，讓頁面不直接依賴套件名稱，也能在未來更換實作。一般操作與導覽圖示固定使用 `currentColor`、16px 預設尺寸與 1.8 stroke width；資料圖表、quota ring 與品牌 logo 保留專用 SVG。

替代方案：維持自製 SVG 會持續增加維護與不一致成本；導入多個 icon library 或使用 Emoji 會讓筆畫、色彩與 Windows 字型渲染不一致。

### 用小型自訂 primitives，而非完整元件庫

建立 `src/components/ui/`，只包含目前重複且具互動契約的 Button、IconButton、Select/Dropdown 與 Tooltip。它們使用既有 CSS variables、BEM-like class，透過 props 呈現，不呼叫 `invoke()`。原生 `<button>` 與 `<select>` 保留語意與鍵盤行為；需要自訂選單時使用共用 Dropdown，統一外部點擊、Escape、focus 與 ARIA。

替代方案：MUI、Ant Design、Chakra UI 會帶入與 Minimal UI 衝突的預設視覺、較高 bundle 成本與大規模遷移；Headless UI/Radix 雖可行，但目前需求不需額外引入多個 primitive 相依。

### 使用 Oxlint 作為第一階段 lint gate

加入 Oxlint 與 `bun run lint`，先檢查 TypeScript/React 的常見錯誤與未使用程式碼，且 CI 將 lint 設為阻擋檢查。規則以可在現有程式碼庫通過為前提，避免一次導入大量格式化或 hooks 重構。格式化仍由既有工具與 code review 處理。

替代方案：ESLint 生態完整但設定與執行成本較高；僅依賴 TypeScript build 無法涵蓋 React 與常見程式碼品質問題。

### 分階段遷移並以視覺回歸驗證

先遷移共用 action icon、按鈕與 dropdown 出現最頻繁的元件（SessionCard、Sidebar、Settings、Agents/MCP、Dialogs），再以搜尋確認不再為操作 UI 新增 inline SVG 或 Emoji。每一階段執行 lint、build，並在亮色與暗色主題手動檢查 focus、disabled、hover、長文案與窄寬度。

## Risks / Trade-offs

- [共用元件 API 過度抽象] → 只涵蓋已重複出現的 variants 與 states，遇到單次使用需求先保留局部樣式。
- [圖示替換改變辨識度] → 維持既有語意與 i18n tooltip，圖表及 provider 品牌符號不強制替換。
- [原生 select 在平台間外觀不同] → 先以 token 統一容器、尺寸與 focus；僅在現有需求確實需要客製選單時使用 Dropdown。
- [新 lint 造成既有 PR 難以通過] → 在導入前執行並修正本 change 涉及的違規，CI 僅在 `bun run lint` 綠燈後設為 required check。
- [依賴增加] → 僅新增 Lucide React 與 Oxlint，並使用 lockfile 固定版本解析。

## Migration Plan

1. 安裝並設定 lint、Lucide React，確認既有建置可通過。
2. 建立 primitives 與圖示出口，加入基本互動與無障礙測試/檢查。
3. 遷移高頻元件與刪除被取代的 icon 定義、局部重複樣式。
4. 更新貢獻與元件規範，CI 執行 lint。
5. 若遷移出現視覺或互動回歸，可保留新 foundation 並逐個元件回退至舊 markup；不涉及資料遷移或後端回滾。

## Open Questions

- 無。Provider 圖示初期採統一的文字縮寫/色彩 badge 或 Lucide 通用標記，是否要在後續 change 引入經授權的官方品牌 SVG，需另行確認授權與設計需求。
