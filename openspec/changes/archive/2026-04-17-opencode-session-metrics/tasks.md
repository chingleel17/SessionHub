## 1. Rust 後端 - SQLite schema 與資料結構

- [x] 1.1 在 `lib.rs` 的 `SessionStats` struct 新增 `model_metrics: BTreeMap<String, ModelMetricsEntry>` 欄位，並新增 `ModelMetricsEntry` struct（requestsCount: u64, requestsCost: f64, inputTokens: u64, outputTokens: u64）
- [x] 1.2 在資料庫初始化與 migration 邏輯中，執行 `ALTER TABLE session_stats ADD COLUMN model_metrics TEXT` 若欄位不存在
- [x] 1.3 修改 `upsert_session_stats_cache` 函式：新增 model_metrics JSON 序列化並寫入 `model_metrics` 欄位
- [x] 1.4 修改 `get_session_stats_cache` 函式：從 `model_metrics` 欄位讀取並反序列化回 `BTreeMap<String, ModelMetricsEntry>`

## 2. Rust 後端 - Copilot modelMetrics 解析

- [x] 2.1 新增 `SessionShutdownData` struct（使用 serde），解析 `data.modelMetrics`：`HashMap<String, ModelMetricsRaw>`，其中 `ModelMetricsRaw` 含 `requests: { count: f64, cost: f64 }`、`usage: { inputTokens: u64, outputTokens: u64 }`
- [x] 2.2 在 `parse_session_stats_internal` 的 event 解析迴圈中，新增 `"session.shutdown"` 分支，讀取 `modelMetrics` 並填入 `stats.model_metrics`

## 3. Rust 後端 - OpenCode 路徑診斷修復

- [x] 3.1 確認 `SessionInfo.session_dir` 在 OpenCode session 下傳入的實際格式（新增 debug logging 或檢查現有路徑建構邏輯）
- [x] 3.2 在 `get_session_stats_internal` 的 OpenCode 分支中，若 `session_path`（即傳入的 `session_dir`）不是一個目錄，而是一個 `.json` 檔，則取 stem 為 sessionID，組合 `<opencodeRoot>/message/{sessionID}/` 路徑
- [x] 3.3 驗證 `calculate_opencode_session_stats` 的 `message_dir.parent().parent()` storage root 推導邏輯，確認在各種 `session_dir` 輸入格式下均正確

## 4. TypeScript 前端型別

- [x] 4.1 在 `src/types/index.ts` 新增 `ModelMetricsEntry` 介面（requestsCount, requestsCost, inputTokens, outputTokens），並在 `SessionStats` 加入 `modelMetrics: Record<string, ModelMetricsEntry>`（可選，預設 `{}`）

## 5. 前端 UI - SessionStatsPanel 計費顯示

- [x] 5.1 在 `SessionStatsPanel.tsx` 加入 modelMetrics 顯示區塊：僅在 provider 為 `copilot` 且 `modelMetrics` 非空時顯示
- [x] 5.2 顯示各 model 的 requestsCost，並計算所有 model 的 `totalCost`（加總）
- [x] 5.3 在 `src/locales/zh-TW.ts` 與 `en-US.ts` 新增對應翻譯 key（如 `stats.modelCost`、`stats.totalCost`、`stats.requestsCount`）

## 6. 驗收測試

- [x] 6.1 手動以含 `session.shutdown` 的 `events.jsonl` 測試，確認 `modelMetrics` 正確解析並顯示於 UI
- [x] 6.2 測試缺少 `session.shutdown` 的 session，確認統計正常、UI 不顯示計費欄位
- [x] 6.3 測試 OpenCode session 統計，確認修復後能正確抓到 message 資料
- [x] 6.4 執行 `cargo test`，確認無 regression
