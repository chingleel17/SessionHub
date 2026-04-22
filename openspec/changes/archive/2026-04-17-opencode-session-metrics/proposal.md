## Why

目前 OpenCode 的 session 統計功能無法正確抓取資料（plugin 層可能有解析問題），同時 Copilot 的 `events.jsonl` 中已包含完整的 `modelMetrics`（含 requests count/cost，支援小數點），這些計費資料尚未被收集與累計顯示，導致使用者看不到實際的 API 點數消耗。

## What Changes

- 修正 OpenCode session 資料讀取問題：診斷並修復 OpenCode plugin 無法正確解析或載入 session 資訊的根本原因
- 新增 Copilot `modelMetrics` 擷取：從 `events.jsonl` 的 `session.shutdown` 事件中讀取 `modelMetrics`，儲存每個 model 的 `requests.count`、`requests.cost`（支援小數點）、`inputTokens`、`outputTokens`
- 新增點數累計顯示：在 UI 中呈現 Copilot 各 session 的計費點數，並提供跨 session 的累計統計

## Capabilities

### New Capabilities

- `copilot-model-metrics`: 從 `session.shutdown` 事件中擷取 `modelMetrics`，將各 model 的 requests cost（小數點）與 token 用量存入 `session_stats` 並提供累計加總

### Modified Capabilities

- `session-stats`: 擴充統計欄位，加入 `model_metrics`（JSON 欄位），記錄每個 session 各 model 的請求數、費用點數、輸入/輸出 token
- `opencode-json-parser`: 修復 OpenCode session 資料讀取異常，確保 plugin 能正確解析 OpenCode 的 session/message/part JSON

## Impact

- `src-tauri/src/lib.rs`：修改 Copilot events.jsonl 解析邏輯，新增讀取 `session.shutdown` 的 `modelMetrics` 欄位；修復 OpenCode JSON 解析路徑或結構問題
- `src/types/index.ts`：擴充 `SessionStats` 型別，加入 `modelMetrics` 欄位
- `src/components/SessionStatsPanel.tsx`：新增 model 計費點數顯示區塊
- SQLite `session_stats` 表：新增 `model_metrics` 欄位（JSON TEXT）
- 不影響現有的 token/interaction count 統計邏輯
