## Context

SessionHub 目前對 Copilot session 的統計從 `events.jsonl` 中解析，但只讀取 `session.start`、`assistant.message`、`tool.execution_start` 等事件，**沒有讀取 `session.shutdown` 中的 `modelMetrics`**。`modelMetrics` 欄位包含每個 model 的 `requests.count`（整數或小數）、`requests.cost`（Copilot 計費點數，可為小數）、`inputTokens`、`outputTokens`，是 Copilot 的原始計費資料。

OpenCode 的 session 資料讀取目前依賴 `scan_opencode_messages_for_session`，可能因為：
1. storage root 路徑組合（`message_dir.parent().parent()`）在不同 OpenCode 版本下路徑層級錯誤
2. `-v2` plugin（`opencode.db` 模式）與 JSON storage 混用導致找不到訊息

## Goals / Non-Goals

**Goals:**
- 從 `session.shutdown` 讀取 `modelMetrics`，每個 model 的 `requests.cost`（f64, 支援小數）儲存至 SQLite
- 在 SessionStatsPanel UI 顯示 Copilot 計費點數（per session + 跨 session 累計）
- 診斷並修正 OpenCode session 統計抓不到資料的根本原因

**Non-Goals:**
- 不支援 OpenCode 的計費點數（OpenCode 無此欄位）
- 不修改 Copilot 以外的計費邏輯
- 不實作帳戶餘額查詢 API

## Decisions

### 1. model_metrics 儲存方式：JSON TEXT 欄位

新增 `session_stats.model_metrics TEXT` 欄位，儲存 JSON 格式：
```json
{"claude-sonnet-4.6": {"requestsCount": 3, "requestsCost": 2.5, "inputTokens": 45388, "outputTokens": 447}}
```

**選擇理由**：`modelMetrics` 的 model key 為動態字串，JSON 比新增多個欄位更靈活；查詢累計時可在 Rust 端反序列化後加總，不需要複雜 SQL。

**替代方案**：獨立 `session_model_costs` 資料表（model, cost 各一列）→ 過度設計，查詢複雜度無明顯優勢。

### 2. 累計統計在前端計算

前端在收到多個 session 的 `SessionStats` 後，直接做 `reduce` 累計 `totalCost`。不在後端另建 `get_total_metrics` command。

**選擇理由**：sessions 已在前端 React Query 快取，不需額外 IPC round trip；累計邏輯簡單。

### 3. OpenCode 問題診斷策略

`calculate_opencode_session_stats` 接收 `message_dir`（即 `<opencodeRoot>/message/{ses_xxx}/`），然後 `.parent().parent()` 取得 storage root（`<opencodeRoot>/`）。若 `session_dir` 傳入的路徑已是 `<opencodeRoot>/session/<projectID>/ses_xxx.json`（session JSON 路徑，而非 message 目錄），則路徑拆解會出錯。

修正策略：在 `get_session_stats_internal` 中，OpenCode session 的 `session_dir` 應傳入正確的 message 目錄路徑（`<opencodeRoot>/message/{session_id}/`），確認前端傳入的 `sessionDir` 與後端期待的格式一致。

## Risks / Trade-offs

- [風險] `session.shutdown` 不一定存在（session 被強制終止）→ 緩解：fallback 為空的 `modelMetrics`，不影響現有統計
- [風險] `requests.cost` 為小數，`f64` 精度在累計大量小數時可能有浮點誤差 → 緩解：顯示時四捨五入到小數點 2-3 位，不做財務精算
- [Trade-off] model_metrics 以 JSON 存在 TEXT 欄位，無法直接 SQL 查詢單一 model 費用 → 已知限制，目前只需跨 session 總和，可接受
