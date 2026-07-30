## Why

專案頁面 sessions sub-tab 的篩選工具列有幾個與實際使用情境不符的地方：排序選項「依 summary 數」在同一專案內鮮少有參考價值；搜尋範圍包含 cwd 路徑，但同一專案分組下的 session 路徑幾乎相同，比對永遠命中、等同無效條件；時間篩選只有「近一週 / 近一月」兩段固定區間，無法查詢特定期間；而使用者真正想找 session 時，記得的往往是「當時對話講過什麼」，現行搜尋卻只比對 summary、notes、tags 與 ID，搜不到對話內容。此外空 session 佔據列表卻無資訊價值，預設應收合。

## What Changes

**階段一（前端）**

- 移除排序選單中的「依 summary 數」選項（`summaryCount`），保留更新時間、建立時間、標題三種排序
- 搜尋框加寬，提高關鍵字輸入時的可讀性
- **BREAKING（搜尋語意）**：搜尋比對範圍移除 `cwd` 路徑，改為比對 session ID、summary、notes、tags
- 「更新時間」下拉選單新增「自訂區間」選項；選中時展開起訖日期輸入，以 session 更新時間落在區間內為篩選條件
- 篩選 chip 文案簡化：「顯示已封存 session」→「已封存」、「隱藏空 session」→「空對話」
- **BREAKING（預設行為）**：空 session 改為預設隱藏，「空對話」chip 由「隱藏」語意反轉為「顯示」語意，點擊後才顯示空 session

**階段二（後端 + 前端）**

- 新增 Tauri 指令 `search_session_content`，接收關鍵字與待搜尋的 session 清單，逐一讀取各 provider 的逐字稿來源並回傳命中的 session ID
- 搜尋框新增「搜尋對話內容」開關；啟用時前端 debounce 後呼叫該指令，關鍵字比對範圍擴大為「文字欄位命中 OR 對話內容命中」，再與 provider、tag、空對話、日期區間等條件以 AND 組合

## Capabilities

### New Capabilities
- `session-content-search`: 跨 provider 讀取 session 逐字稿內容並以關鍵字比對，回傳命中的 session 清單，供列表篩選使用

### Modified Capabilities
- `session-filter`: 搜尋比對欄位移除 cwd；空 session 預設隱藏且 chip 語意反轉；新增自訂日期區間篩選；篩選工具列改為允許換行，原「單行且高度不超過 72px」的排版限制放寬
- `session-list`: 排序選項移除 `summaryCount`

## Impact

**前端**
- `src/components/ProjectView.tsx` — `filterAndSortSessions`（搜尋 haystacks、排序 switch、日期區間判斷）、篩選工具列 JSX、相關 useState
- `src/types/index.ts` — `SortKey` union 移除 `"summaryCount"`
- `src/App.tsx` — `hideEmptySessions` state 預設值與 prop 命名（若改為 `showEmptySessions`）
- `src/locales/zh-TW.ts`、`src/locales/en-US.ts` — chip 文案、搜尋提示文字、移除 `session.sortSummaryCount`、新增自訂區間與內容搜尋相關鍵值
- CSS — 篩選工具列換行與搜尋框寬度樣式

**後端**
- `src-tauri/src/commands/sessions.rs` — 新增 `search_session_content` command 入口
- `src-tauri/src/sessions/` — 各 provider 新增逐字稿內容擷取函式（claude / codex 讀 `.jsonl`、copilot 讀 `events.jsonl`、antigravity 讀 `transcript.jsonl`、opencode 查 SQLite），並於 `mod.rs` 泛用分派註冊
- `src-tauri/src/lib.rs` — 註冊新 command

**無影響**
- `sortKey` 與 `hideEmptySessions` 皆為 React 元件內 state，未持久化至 settings.json 或 SQLite，因此移除排序選項與翻轉預設值不需要資料遷移
- `metadata.db` 的 schema 與資料量完全不變。對話內容搜尋採即時掃描且不落地（design.md 決策七），逐字稿內容不複製進 DB、不快取、不建索引，因此不會造成 DB 膨脹，也不存在快取與 provider 原始檔的一致性問題
