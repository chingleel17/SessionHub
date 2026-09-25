# Proposal

## Why

現有消耗分析缺乏一致的資訊層次與鑽取互動，而且將 session 累計用量歸到最後更新日、混加不同成本單位，無法可靠回答「何時、在哪個專案、用哪個模型消耗多少」。借鑑 TokenUsageInsights 的分析流程與 codex-reset-checker 的方案／配額呈現，建立符合 SessionHub Minimal UI 的可信消耗分析戰情室。

## What Changes

- 新增 Sidebar 獨立「消耗分析」入口；Dashboard 保留精簡摘要，專案 Analytics 子頁籤保留並共用分析元件與查詢口徑。
- 建立事件級用量紀錄與增量索引；可得事件依發生時間統計，只有 session 彙總的來源獨立揭露，不虛構逐日分攤。
- **BREAKING**：更新內部 analytics IPC 回應與指標模型，分離 Token、Copilot 點數、估算 USD，取代混用的 `costPoints` 與 session 更新日歸因；前後端在同一版本同步遷移。
- 統一日期、時區、provider、專案、模型、封存範圍；摘要、趨勢、排行、分頁明細及前期比較共用查詢。
- 以寬幅同單位趨勢、專案／模型橫條排行、精確 tooltip、鍵盤操作與 Session 明細抽屜取代分析頁的多單位折線及圓餅圖。
- 沿用目前工作樹已有的方案標籤、quota snapshots 與 Codex reset 操作，於分析頁提供獨立的帳戶配額區，顯示資料來源與新鮮度。
- 加入資料缺漏、未支援、部分涵蓋、背景補算及過期狀態；以真實資料能力決定顯示內容。
- 本次不加入訂閱到期／續約查詢、不切換 quota 後端、不新增遠端資料上傳或完整對話時間軸。

## Capabilities

### New Capabilities

- `usage-event-ledger`: 可追溯、去重、增量更新的事件用量與獨立 session 彙總資料，含 provider 能力及資料完整性。
- `usage-analytics-workspace`: 獨立消耗分析導覽、共同查詢工作區、期間比較、Session 鑽取及方案配額整合。

### Modified Capabilities

- `analytics-query-engine`: 事件日期聚合、統一篩選、單位分離、完整性與明細查詢契約。
- `analytics-charts-ui`: 同單位趨勢、橫條排行、互動鑽取、可及性與主題。
- `analytics-controls`: 共用篩選列、自動查詢與手動重新整理。
- `dashboard-analytics-panel`: 精簡摘要及前往完整分析的入口，保留折疊與刷新設定。
- `project-analytics-tab`: 保留專案頁籤，改為共用分析工作區、自動載入及查詢快取。
- `project-subtabs`: 頂層加入獨立分析入口，同時保留專案與 Plan 子頁籤行為。

## Impact

- 前端：`src/App.tsx` 路由、React Query 與集中 IPC，Sidebar、DashboardAnalyticsPanel、ProjectAnalyticsTab、TrendChart、分布元件、共用型別、zh-TW/en-US 與 CSS tokens。
- 後端：`commands/analytics.rs`、`types/analytics.rs`、`db.rs`、各 provider stats/parser、背景補算及 Tauri command 註冊；新增可重建的 SQLite 衍生表，不破壞 session 原始檔與備註／標籤。
- 依賴：預計採 Recharts 3 支援 React 19、responsive tooltip 與可及性；實作時以 CLI 安裝，先查證 Bun 相容性。
- 既有暫存的 quota／plan 變更是本案基準；不覆寫或重做其能力，也不修改其 provider-quota-monitoring 規格。
- 驗證：聚合／去重／日期／缺漏／遷移測試、互動測試、Windows Tauri 雙主題與效能實測。
- 參考：https://github.com/doggy8088/TokenUsageInsights 、https://github.com/doggy8088/codex-reset-checker 。若實際移植 MIT 程式碼或資產，保留必要授權聲明。
