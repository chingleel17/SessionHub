# Design

## Context

動機見 proposal.md。現有 `commands/analytics.rs` 以 session `updated_at` 聚合 `session_stats`；專案與 Dashboard 分布另在前端計算。Claude 有帶時間與訊息 ID 的 usage，OpenCode 有訊息時間與 tokens，Copilot 完整模型統計主要在 shutdown；Codex 尚無本機 stats parser。不能把所有 provider 都視為事件粒度完整。

本案採用工作樹現有 quota／plan 變更作為基準。前端 IPC 留在 App.tsx，Rust commands 委派 internal 函式，衍生資料放現有 SQLite；設計遵循 sessionhub-minimal-ui。

## Goals / Non-Goals

**Goals:**
- 日期、單位、去重與涵蓋範圍可追溯；同一查詢下所有圖表和明細可對帳。
- 全域、專案、Dashboard 共用查詢服務與呈現元件，資料更新不阻塞 session 操作。
- 支援逐步補算和部分來源，不將缺資料誤報成零。

**Non-Goals:**
- 不重構所有 session stats 或改寫現有 provider quota adapters。
- 不導入雲端服務、帳單結算、訂閱續約偵測、完整對話時間軸、匯出或長期配額歷史。
- 不使用 quota 百分比推算 tokens，不為未知模型自行猜價格。

## Decisions

### 1. 獨立工作區與共用專案視圖

在現有 activeView union 增加 `analytics`，沿用 Sidebar 導覽，不引入 URL router。入口位於 Dashboard 之後；全域 Agents、Settings 及釘選排序維持各自角色。保留專案 Analytics 子頁籤及其相對順序，專案版鎖定該專案的 canonical cwd，跨分支 session 依目前專案路徑範圍統計，不把 branch 顯示名稱當 cwd。

全域預設近 30 個本地日（含今天）、按日、已啟用 providers、未封存、所有專案／模型。專案版使用相同預設但固定 cwd。Dashboard 使用原本「本週／本月」並保留折疊、10／30 分鐘刷新設定；只呈現摘要與迷你趨勢，再導向完整分析並攜帶相同篩選。

相較把全部內容塞進 Dashboard，此方案保留 session 管理空間，又不犧牲專案工作流。

### 2. 事件紀錄與 session 彙總分開

新增三類衍生資料：
- `usage_events`：provider、session_id、source_event_id、UTC occurred_at、canonical cwd、model、正規化 token 欄位、nullable estimated_usd／cost_points、source_kind、parser_version。
- `usage_session_summaries`：僅有 session 粒度的總量、模型與單位，以及可得的 session 時間範圍；不與事件表相加。
- `usage_ingestion_state`：來源 identity／fingerprint、parser version、完整性、已提交 revision、錯誤摘要與掃描狀態。

事件唯一鍵為 `(provider, session_id, source_event_id)`；所有 metadata join 必須同時含 provider 與 session_id。欄位未知使用 null，完整性按指標區分。來源未提供 model 時用明確 unknown 分組，不沿用另一個 provider 或 session 的值。

`inputTokens` 正規化為含 cache 的輸入總量；cacheRead/cacheWrite 是輸入子集合，reasoning 是輸出子集合。`totalTokens = inputTokens + outputTokens`，不得再加 cache 或 reasoning；任一總量必要欄位未知則 totalTokens 未知，另外顯示已知 output 等欄位。供應商若 input 原值排除 cache，adapter 必須依已驗證格式補入，保留來源語意於 fixture 與 parser 註解。

成本為獨立 nullable 指標。USD 沿用可得價格來源並保存估價版本／模型，明示估算；Copilot 點數若只有 shutdown 彙總，只出現在 session 彙總區，不做逐日曲線。無可得價格的事件計入成本缺漏，不是免費。

相較遷移舊 session totals 成每日記錄，新表可由原始來源重建，避免把錯誤日期延續下去。

### 3. Provider 能力與去重規則

| Provider | 第一版行為 |
|---|---|
| Claude | 依 assistant message ID 合併串流修訂，以首次有效事件時間定位、最後有效 usage 更新同一事件；沿用已驗證的 cache 語意與價格 |
| OpenCode | JSON 與 DB 經同一正規化層，以 message ID 去重；以 completed 時間為優先、created 備援；只填已驗證欄位 |
| Copilot | 帶時間的 assistant output 作事件資料，模型可由同 session 的 model-change 時序推定並揭露；shutdown 模型 totals 只進 summary 表，不與事件 output 相加 |
| Codex | 實作前以官方格式與去識別 fixture 查證 token_count 類事件；可驗證的逐次 usage 或相鄰累計差值才入帳；未驗證格式標示 unsupported，不從 quota 補值 |
| Antigravity | 本版明示 token analytics unsupported，保留 session 與 quota 資訊 |

Codex 差值須在同一 session、同一可辨識 counter epoch 且時間順序可信時取差；第一筆累計沒有基準不能假設從零開始，counter 倒退視為新 epoch 並重設基準。沒有 epoch/model/timestamp 證據的區段標記 partial，不推算。此能力降級規則已定案，格式查證結果只影響 adapter 可提供欄位，不改 UI 或共同查詢契約。

### 4. 增量更新採來源變更偵測與單 session 原子替換

第一版不做脆弱的 JSONL byte-offset tailing。沿用背景掃描，以 source fingerprint／DB 更新資訊找變更，只重解受影響 session；在單一交易中替換該 session 的事件、summary 和 ingestion revision。相同來源 identity 不變則跳過。來源尾端半筆先保留上次完整 revision 並標記 pending；單檔解析錯誤不影響其他 session。

中斷可恢復、重跑冪等；檔案縮短／改寫會重建該 session，刪除偵測需在來源可讀且完整掃描成功後才清除衍生列。根目錄失聯不當成全部刪除；設定根目錄變更後舊來源資料不參與目前查詢，待新來源掃描完成再清理。parser version 改變時觸發可恢復重建。

沿用每批至多 50 個 session 的背景節奏，批次完成推送 revision 讓前端 invalidate；不在查詢 command 中解析原始檔。SQLite 查詢使用讀取交易取得一致 revision；時間/provider/cwd/session 依查詢計畫設索引。

### 5. 共同查詢契約

更新 `get_analytics_data` 為 query object → `AnalyticsReport`，另加分頁 session detail command，兩者共用相同 query 正規化與篩選組合。全部 command 在 App.tsx 呼叫，後端採 `_internal`。

Query：`startDate`、`endDate`（含當日）、`timeZone`（系統 IANA 時區）、`groupBy`、`providers`、nullable `cwd`、nullable `model`、`includeArchived`、`metric`、`tokenField`（total/input/output，預設 total）、`breakdown`（total/provider/model/tokenType）。tokenField 僅在 Token 指標生效；tokenType 拆分只在 tokenField=total 時提供。明細另有 page、pageSize（預設 50，最多 200）及 sort（預設事件時間降冪，以 provider/session key 決定同值順序）。空 providers 表示空選取；「全部」由前端傳入目前啟用清單。未知 model 是可篩選的明確值。

本地 startDate 00:00 至 endDate 次日 00:00 轉為 UTC 半開區間；不得用固定 offset 取代時區規則。週從週一開始，月從一日開始，首尾桶只統計區間內事件。前一期採相同數量本地日、緊接於本期之前，使用相同非日期條件。

Report 含 summary、previousSummary、series、projectRanking、modelRanking、coverage、sessionSummaryOnly、revision、generatedAt。各 metric 回傳 knownValue、缺漏／能力資訊，避免以 nullable 總和隱藏部分涵蓋。事件型主分析只計 event-level 資料；sessionSummaryOnly 獨立區域以 session 活動範圍與查詢區間重疊篩選，明示「全 session 總量，非此期間用量」；無可得範圍者不納入日期篩選，顯示未定位數量。

已支援且索引完成的空桶補零；pending、unsupported、parse-error 的範圍不能宣稱零。沒有任何事件的查詢仍回傳 coverage 與明確空狀態。sessionCount 為 distinct provider/session 的事件活躍數，interactionCount 為去重 usage 訊息數，不假稱使用者提問次數。同一 session 跨多桶時 sessionCount 不要求桶加總等於摘要。

明細帶 revision；補算使 revision 改變時，回傳 stale-revision 狀態並重新載入整份報表和第一頁明細，避免兩份 snapshot 混用。排行顯示完整可捲動清單，不把「其他」當作未定義 filter。

### 6. 精確的聚合與一致性契約

**時區取得**：前端以 `Intl.DateTimeFormat().resolvedOptions().timeZone` 取得 IANA identifier，不傳 Windows registry 時區名稱。後端以 IANA database 驗證。前端取不到或後端不認得時，不默默改用 UTC；顯示時區錯誤並提供明確「使用 UTC」操作，使用者確認後 query 與畫面都標記 UTC。系統時區變動於 view focus 時重讀並形成新 query key。

**Revision**：每個成功改變可查詢衍生內容的交易，同時遞增單一 DB 全域 `analyticsRevision`，而非拿 session revision 當報表 revision。來源範圍、metadata 中影響 cwd／封存的異動也遞增。報表的 revision 與所有聚合於同一 SQLite read transaction 讀取。明細在 read transaction 內先比對 requested revision；不符回 stale-revision，不保存長期 MVCC snapshots。Ingestion state 的 session revision 只作更新進度紀錄。

**逐指標值**：對所選條件下每筆可定位事件，metric eligible 表示該指標必需欄位均已知。`knownValue` 為 eligible 事件之和；有候選事件卻無任何 eligible 事件時為 null；已確認完整且無事件時為 0。`totalTokens` 的 eligible 必須 input 和 output 同時已知，不能把不同事件各自已知的 input/output 拼成 total。input、output 單獨查詢可使用較大的 eligible 集合；介面明示部分涵蓋。total 的 tokenType 堆疊只使用 total eligible 集合，其 input/output 分項和等於 total；cache/reasoning 只在已知時顯示 tooltip 子集合，未知顯示破折號，不推為零。Copilot 只有 output 時仍可選 output 分析，但不列入 total 趨勢。

**Coverage 不使用假百分比**：每個 metric 回 eligibleEventCount、missingFieldEventCount，另回 complete／partial／pending／unsupported／error／summaryOnly session counts 及 provider 狀態。事件 counts 的分母僅是已解析、可定位且符合篩選的事件，不能代表尚未解析的實際用量。無可定位時間的 pending/error/unsupported session 在 provider/cwd/封存 scope 下列為「無法判定期間涵蓋」，不假設零。complete 要求範圍內無 missing fields，且沒有上述未知涵蓋來源；否則標示 partial 或對應不可用狀態。排行百分比僅為該項 knownValue／所有排行 knownValue，不是 coverage 百分比，分母零時不顯示百分比。

**成本與比較**：成本的 knownValue 只加有有效價格／點數的 eligible 事件，另列 missingFieldEventCount。美元與點數各採十進位固定 6 位精度，逐事件計算以 half-up 捨入到 1e-6 後儲存整數微單位，加總後才格式化（一般 USD 顯示 2 位，tooltip 至多 6 位）；不以顯示數字重新加總。不同估價版本可合併為歷史記錄估值，顯示版本混合說明，不自動以新價格重估歷史。前期百分比僅在兩期該 metric 都 complete 且價格基準版本集合一致時提供；partial 或估價基準改變時只顯示各期已知值與不可比較原因。零基期且本期正值顯示新增用量；兩期完整皆零顯示無變化。

### 7. 快取與載入

React Query key 包含所有正規化條件、來源設定 revision 及 ingestion revision；全域與各專案 filter state 由 App 持有至應用關閉。相同 key 在既有刷新間隔內重用；回到過期 view 時背景刷新。篩選變更自動查詢，自訂日期兩端均合法才提交；不保留舊「產生圖表」操作。

載入同 key 的新 revision 時保留舊圖並標示更新中；變更 filter 時不得把舊結果標成新條件，需顯示載入或清楚保留舊條件標籤。失敗呼叫 showToast，提供重試；過期結果標明時間。隱藏 view 不啟動輪詢。重新整理分析只同步本機用量，quota 以原有獨立刷新機制運作。

### 8. 視覺與圖表技術

採 Recharts 3（React 19 peer 支援、ResponsiveContainer、Tooltip、accessibilityLayer），以共用 wrapper 統一數字與單位、主題、空狀態及鍵盤選取；排行可用原生可聚焦 row/bar，無須強制所有視覺由圖表套件產生。相較擴充現有手製 SVG，可減少互動與座標維護；不引入第二套 UI framework。

版面：共用 filter row → 無卡片框線的 summary strip → 寬幅趨勢 → 並列排行 → 分頁 session rows；窄視窗改單欄。配額在獨立可收折分區，標明帳戶級、不受歷史日期／專案篩選影響。

Token、USD、點數一次只選一個單位；Token 可切 total/provider/model/tokenType。tokenType 僅堆疊互斥的 input/output，cache/reasoning 在 tooltip 顯示子集合。選日期桶會將查詢區間收窄到該桶與原區間的交集；排行點擊套用 cwd/model；提供清除鑽取回到上一篩選。session drawer 顯示此區間 usage breakdown、來源與完整性，可開啟原有 session；不載入對話全文。

一般內容透明、留白及 hairline 分區；浮層才使用 glass 和浮層陰影。色彩使用 CSS tokens，主序列用品牌色；數字 tabular-nums；圖例同時有文字與符號；支援鍵盤與可讀數值表、reduced-motion、zh-TW/en-US。

官方技術參考：https://github.com/recharts/recharts/blob/main/package.json 、https://github.com/recharts/recharts/wiki/Recharts-and-accessibility 。安裝前再查 Bun 相容性，若無可靠證據依專案規範使用 npm；不手改 package.json。

### 9. 配額整合沿用既有成果

重用 QuotaOverview、quotaPlanLabel 與快取 snapshots；方案未知時沿用既有 fallback，顯示來源、最後刷新、stale/error。歷史用量載入失敗與 quota 失敗互不遮蔽。Codex reset 沿用既有確認與回饋流程，這次不新增 endpoint、不調整 consume 語意、不把 reset 當作歷史用量清零。

## Risks / Trade-offs

- [資料粒度不齊] → 指標級 coverage、session summary 分區，優先誠實揭露而非補假資料。
- [舊圖數字與新圖不一致] → 提供統計口徑說明，區分事件日期及 session 全期值，不沿用舊 totals 作新圖快取。
- [串流修訂、累計 counter 重複計算] → 唯一鍵、單 session 原子替換與 fixture 驗證；不明區段降級 partial。
- [長 session 重解成本] → 只處理變更來源、背景批次；先量測，未有證據不增加 cursor 引擎。
- [圖表包體積與 WebView 行為] → 分析頁 lazy loading，紀錄 build 大小與 Windows resize／鍵盤實測結果。
- [大量資料查詢延遲] → 以 100,000 事件／10,000 sessions fixture 測試 30 日查詢，暖快取 20 次 p95 目標 500ms 內；記錄硬體、build 模式與實際結果，超標分析 query plan 後處理，不虛報達標。

## Migration Plan

0. 既有 analytics-charts-ui、analytics-controls、analytics-query-engine、project-analytics-tab、project-subtabs 主規格首行誤用 delta header。實作準備時只將其 `## ADDED Requirements` 正規化為 `## Requirements`，不改既有行為，再驗證 delta 可歸檔；此次建立 change 不直接修改主規格。
1. 建立資料語意 fixture，先確認各 provider 能力與 token 定義；Codex 未驗證格式走既定 unsupported/partial。
2. 新增衍生表與 schema version，保留舊 session_stats 供其他頁面；交易化遷移失敗不標記成功。
3. 背景從目前設定來源補算；期間顯示 indexing 與局部結果，重啟後續跑。
4. 前後端同版切換 IPC，移除分析專用舊 client-side totals 路徑；不刪除仍由其他介面使用的元件或 stats。
5. 導入新工作區與 Dashboard／專案整合，驗證來源切換、封存、刪除及 quota 回歸。
6. 回復舊版時舊程式可忽略新增衍生表；不需回復原始 session 或使用者 metadata。新索引損壞時僅重建衍生表，不刪除 metadata.db。
