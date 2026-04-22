## ADDED Requirements

### Requirement: 解析 events.jsonl 取得 Copilot session 統計

系統 SHALL 解析 Copilot session 目錄下的 `events.jsonl`，計算使用統計資料，包含 `modelMetrics`（各 model 的計費點數與 token 用量）。

#### Scenario: 成功解析

- **WHEN** `get_session_stats` 被呼叫且 session 目錄含有 `events.jsonl`
- **THEN** 回傳 SessionStats 包含：outputTokens、interactionCount、toolCallCount、durationMinutes、modelsUsed、reasoningCount、toolBreakdown、**modelMetrics**

#### Scenario: 缺少 events.jsonl

- **WHEN** session 目錄無 `events.jsonl`
- **THEN** 回傳所有數字欄位為 0、modelMetrics 為空物件的 SessionStats

#### Scenario: 格式錯誤行

- **WHEN** events.jsonl 含有無效 JSON 行或缺少必要欄位的行
- **THEN** 跳過該行繼續解析，不拋出錯誤

### Requirement: 排除子 agent 事件

系統 SHALL 只計算頂層事件，排除子 agent 工具呼叫。

#### Scenario: 子 agent 事件不計入

- **WHEN** events.jsonl 中有 `data.parentToolCallId` 非空的事件
- **THEN** 該事件不計入 toolCallCount、reasoningCount、interactionCount

### Requirement: Live session 標記

系統 SHALL 偵測 session 是否為進行中（live），並對 live session 不快取統計。

#### Scenario: Session 有 lock 檔

- **WHEN** session 目錄下存在 `inuse.*.lock` 檔案
- **THEN** SessionStats.isLive = true，且結果不寫入 SQLite 快取

### Requirement: SQLite 快取統計結果

系統 SHALL 將計算完成的統計結果快取至 `session_stats` 資料表，以 mtime 失效。

#### Scenario: 快取命中

- **WHEN** session 非 live 且 events_mtime 未變更
- **THEN** 直接回傳快取統計，不重新解析

### Requirement: modelMetrics 欄位包含於 SessionStats 型別

`SessionStats` 型別 SHALL 新增 `modelMetrics: Record<string, ModelMetricsEntry>` 欄位，其中 `ModelMetricsEntry` 包含 `requestsCount: number`、`requestsCost: number`（支援小數）、`inputTokens: number`、`outputTokens: number`。

#### Scenario: 前端接收含 modelMetrics 的 SessionStats

- **WHEN** 前端呼叫 `get_session_stats` 並收到回傳值
- **THEN** `stats.modelMetrics` SHALL 可正確索引各 model key 並讀取 requestsCost

### Requirement: UI 顯示 Copilot 計費點數

`SessionStatsPanel` 元件 SHALL 在 Copilot session 的統計區塊中顯示 `modelMetrics` 的各 model 計費點數（requestsCost），並顯示所有 model 的點數總和。

#### Scenario: session 有計費資料

- **WHEN** `SessionStats.modelMetrics` 非空且 provider 為 `copilot`
- **THEN** UI SHALL 顯示各 model 的 requestsCost 及總計點數

#### Scenario: session 無計費資料

- **WHEN** `SessionStats.modelMetrics` 為空物件
- **THEN** 不顯示計費區塊（或顯示 `-`）
