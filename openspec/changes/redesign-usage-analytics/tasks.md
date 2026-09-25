# Tasks

## 1. 資料契約與實作準備

- [x] 1.0 將 analytics-charts-ui、analytics-controls、analytics-query-engine、project-analytics-tab、project-subtabs 主規格的既有 delta header 正規化為 Requirements（僅結構、不改行為）；以 OpenSpec 查詢確認 requirement 可見，且本 change 嚴格驗證不再出現 archive 結構阻擋提示。
- [x] 1.1 核對現有工作樹 quota／plan 變更與 analytics 呼叫端，列出需同步遷移的型別及入口；以影響清單確認未將既有方案辨識與 reset 操作重做。
- [x] 1.2 建立 Claude、OpenCode、Copilot 去識別 usage fixtures，記錄時間、ID、cache／reasoning 包含關係及成本單位；以可重現預期總量驗證各來源資料語意。
- [x] 1.3 查證 Codex 官方事件格式並建立含重複、累計倒退、首筆無基準與未知格式 fixtures；以明確 expected outcomes 確認哪些欄位可支援、哪些走 partial／unsupported，不以 quota 推算。
- [x] 1.4 定義 Rust／TypeScript query、report、coverage、revision 與 detail 型別；以序列化契約測試驗證 camelCase、null、獨立成本單位及跨 provider session key。

## 2. 衍生用量儲存與正規化

- [x] 2.1 新增 usage events、session summaries、ingestion state 與 schema migration；以新 DB、既有 DB、重跑及交易失敗測試確認不破壞 session metadata。
- [x] 2.2 實作事件 upsert／單 session 原子替換與 provider 複合識別；以重複匯入、跨 provider 相同 ID 和中途失敗測試驗證冪等及原子性。
- [x] 2.3 實作 Claude message 去重與事件時間／cache／估價正規化；以串流修訂 fixture 驗證只計最後有效 usage 且不重複加 cache。
- [x] 2.4 實作 OpenCode JSON／DB 共用正規化；以相同 message 雙來源 fixture 驗證結果一致且不重複，reasoning 不重複計入 total。
- [x] 2.5 實作 Copilot output 事件、模型時序來源標記與 shutdown summary 分流；以跨日 session fixture 驗證兩種資料不相加且未知 input 不當成零。
- [x] 2.6 由於官方協定尚未證實本機 rollout JSONL 的持久化包裝，Codex ingestion 暫回 unsupported 且不輸出事件或摘要；以 1.3 fixtures 驗證首筆累計不當成用量、重複／倒退不產生差值、未知格式不推算，待持久化格式可驗證後再加入解析器。
- [x] 2.7 實作 nullable 指標、估價來源版本與 provider 能力輸出，包含 Antigravity unsupported；以混合來源測試確認點數／USD 分離且未知成本不當免費。

## 3. 增量索引與生命週期

- [x] 3.1 接入來源 fingerprint／DB 更新資訊與背景批次索引，每批至多 50 sessions；以第二次掃描驗證未變更來源不重新解析、前景查詢不讀原始檔。
- [x] 3.2 實作半筆來源、解析錯誤、重啟續跑及 revision 發布；以中斷恢復測試確認保留完整舊結果、無重複計量且單一錯誤不阻斷其他來源。
- [x] 3.3 處理來源改寫、截短、刪除、根目錄失聯與設定切換；以生命週期測試確認刪除需成功完整掃描、失聯不清空、新根目錄不混入舊資料。
- [x] 3.4 實作 parser version 重建與補算進度；以版本升級與索引損壞恢復測試確認只重建衍生資料且 session 備註／標籤保留。

## 4. 統一分析查詢

- [x] 4.1 實作前端 Intl IANA 時區取得、明確 UTC 備援、本地日界線及週／月分組；以時區不可用、台北跨午夜、DST、跨 ISO 週年、閏日及單日測試驗證無靜默備援與正確 UTC 半開區間。
- [x] 4.2 實作 provider／cwd／model／封存／來源設定共同篩選；以空 provider、停用來源、unknown model、封存切換及跨分支同 cwd 測試驗證範圍。
- [x] 4.3 實作 summary、series、完整排行、逐指標 coverage 與 summary-only 分區；以部分 output、未知 input、未解析來源、全零 fixture 驗證 eligible 集合、缺漏 counts、排行分母及 distinct session 正確，完整空桶才補零。
- [x] 4.4 實作前一等長本地日區間比較與固定 6 位成本加總；以 half-up 邊界、混合估價版本、零基期、缺漏與能力不同測試確認精度、不重估歷史且不產生假精確成長率。
- [x] 4.5 實作全域交易 revision、分頁明細、穩定排序與 stale-revision；以跨頁同值、超限 pageSize、補算／封存／cwd 異動中翻頁測試確認無遺漏重複或混合版本。
- [x] 4.6 更新 analytics commands、internal helpers 與 Tauri 註冊；以 IPC 契約測試驗證前後端同步，並確認分析不再使用混合 costPoints 或 updated_at 歸因。

## 5. 共用前端狀態與導覽

- [x] 5.1 新增 activeView analytics、Sidebar 固定入口與各 scope 狀態；以導覽測試驗證 active、收折名稱、釘選排序不受影響及原專案／Plan tabs 保留。
- [x] 5.2 在 App.tsx 集中報表與明細 IPC，建立完整 React Query keys、revision invalidation 與背景刷新；以切 scope、補算更新及隱藏 view 測試驗證不串資料或持續輪詢。
- [x] 5.3 建立共用篩選列及有效日期自動提交，移除分析產生圖表按鈕；以快速期間、自訂未完成日期、封存／provider 切換與手動刷新互動驗證。
- [x] 5.4 建立 loading、partial、unsupported、error、stale 與 retry 呈現；以錯誤注入驗證舊結果保留原條件且 showToast 正常。

## 6. 戰情室視覺與互動

- [x] 6.1 查證 Recharts 3 與 Bun 相容性後依規範以 CLI 安裝，建立 lazy-loaded 圖表 wrapper；以 build、React 19 渲染、ResizeObserver resize 及產物大小記錄驗證（Recharts 3.10.1；AnalyticsTrendChart chunk 355.55 kB／gzip 103.18 kB）。
- [x] 6.2 建立 Minimal summary strip 與期間比較，分開 Token、估算 USD、點數；以完整／部分／零基期 fixture 驗證標籤、數字及比較說明。
- [x] 6.3 重構趨勢圖為同單位序列、tooltip、token 子集合明細及日期鑽取；以單點、空資料、鍵盤巡覽及指標切換驗證無混軸或重複堆疊。
- [x] 6.4 建立專案／模型排序橫條與清除鑽取，移除分析頁的圓餅分布；以長尾、單一項目、全零、partial 與 unknown model 驗證所有項目可查閱。
- [x] 6.5 建立分頁 session rows 與明細抽屜、開啟原 session 操作；以範圍內值／全期值、revision 更新、Escape 關閉與焦點歸還驗證。
- [x] 6.6 補齊 zh-TW/en-US 與 CSS tokens，完成窄視窗／雙主題／reduced-motion；以實際畫面確認內容去卡片、tooltip 對比、數字對齊與無操作遮蔽。

## 7. Dashboard、專案與配額整合

- [x] 7.1 將 Dashboard Analytics Panel 改為摘要與迷你趨勢，保留折疊及 10／30 分鐘設定；以本週／本月切換及攜帶條件開啟完整分析驗證。
- [x] 7.2 將 ProjectAnalyticsTab 改用固定 cwd 的共用工作區；以切換專案／子頁籤驗證獨立條件、快取與原 Agents 後順序，且不顯示冗餘單專案排行。
- [x] 7.3 整合獨立帳戶級 QuotaOverview 與既有 quotaPlanLabel；以日期切換、quota／analytics 各自失敗及 stale snapshot 驗證互不影響。
- [x] 7.4 回歸既有 Codex reset 確認與結果刷新，透過 mock 驗證流程未改、不發出真實 consume 請求，且不清除歷史 usage；保留既有方案標籤測試。
- [x] 7.5 移除不再使用的分析專用前端彙總與舊 IPC 呼叫，保留其他功能仍使用的 stats／元件；以引用檢查與 build 確認無遺留舊統計路徑。

## 8. 整體驗收

- [x] 8.1 執行相關前端測試（依現有 tests runner）、`bun run lint`、`bun run build` 與 src-tauri 工作目錄的 `cargo test`；修正失敗並記錄命令與結果，不以缺少 package test script 代替測試確認。
- [x] 8.2 以 100,000 事件／10,000 sessions fixture 量測 30 日查詢 20 次暖快取 p95，目標 500ms 內；記錄 Windows 硬體、build 模式、查詢計畫與補算時互動情況，超標時修正後重測。驗收記錄：Windows x64、Intel Core i7-14700F（20 核心／28 執行緒、64 GiB）、release、SQLite in-memory fixture；暖查詢 p95 371.54 ms，50-session 背景補算期間 p95 368.29 ms；query plan 使用 idx_usage_events_provider_time 與 sessions_cache 主鍵索引。
- [ ] 8.3 完成 Windows Tauri 深淺主題、窄視窗、鍵盤、螢幕閱讀文字替代、全域／專案／Dashboard 對帳驗收；保存結果及必要截圖證據。
- [x] 8.4 驗證舊 DB 升級、補算中關閉重啟及回復舊版忽略新增表；確認原始 session 與 metadata 未變更，並執行 `openspec validate redesign-usage-analytics --strict`。
