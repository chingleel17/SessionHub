## ADDED Requirements

### Requirement: 從 JSON storage 讀取 OpenCode session 統計
系統 SHALL 能讀取 `~/.local/share/opencode/storage/message/{sessionID}/` 目錄下的所有 `msg_*.json` 檔案，並加總計算出該 session 的統計資料。

#### Scenario: 正常計算統計
- **WHEN** 使用者查詢一個有效的 OpenCode session 統計
- **THEN** 系統從 `storage/message/{sessionID}/` 掃描所有 msg_*.json
- **AND** 回傳正確的 `outputTokens`、`inputTokens`、`interactionCount`、`toolCallCount`、`durationMinutes`、`modelsUsed`

#### Scenario: Session 無訊息資料
- **WHEN** `storage/message/{sessionID}/` 目錄不存在或為空
- **THEN** 系統回傳全零的統計結果，不拋出錯誤

#### Scenario: JSON 格式不完整
- **WHEN** 某個 msg_*.json 缺少 `tokens` 欄位或格式不符預期
- **THEN** 系統跳過該檔案的問題欄位，其餘欄位正常累計，不中止整體計算

### Requirement: Token 統計涵蓋 input/output/reasoning/cache
系統 SHALL 在計算 OpenCode session 統計時，分別累計 input tokens、output tokens、reasoning tokens、cache read tokens。

#### Scenario: 累計各類 token
- **WHEN** 系統解析 msg_*.json 中 role 為 `assistant` 的訊息
- **THEN** 從 `tokens.input`、`tokens.output`、`tokens.reasoning`、`tokens.cache.read` 欄位累計對應值

#### Scenario: User 訊息不計入 token
- **WHEN** msg_*.json 的 `role` 為 `user`
- **THEN** 不累計 token 欄位（user 訊息無 tokens 欄位）
- **AND** 計算 `interactionCount` 時累加 1（每筆 user 訊息代表一次互動）

### Requirement: 工具呼叫統計
系統 SHALL 統計 OpenCode session 中各工具的呼叫次數。

#### Scenario: 從 assistant 訊息取得工具統計
- **WHEN** 系統解析 `storage/part/{messageID}/prt_*.json` 中 type 為 `tool` 的 part
- **THEN** 以 `state.tool` 欄位名稱為 key，累計呼叫次數至 `toolBreakdown`
- **AND** `toolCallCount` 累加對應次數

#### Scenario: 工具名稱缺失時降級處理
- **WHEN** part 的 `state.tool` 欄位為空或缺失
- **THEN** 以 `"unknown"` 作為 key 記錄，不拋出錯誤

### Requirement: Session 時長計算
系統 SHALL 計算 OpenCode session 的對話時長（分鐘）。

#### Scenario: 正常計算時長
- **WHEN** session 下存在至少兩筆訊息
- **THEN** 以最早訊息的 `time.created` 至最晚訊息的 `time.created`（或 `time.completed`）計算時長，單位為分鐘

#### Scenario: 單一訊息 session
- **WHEN** session 下只有一筆訊息
- **THEN** `durationMinutes` 回傳 0

### Requirement: 統計結果快取
系統 SHALL 將計算完成的 OpenCode session 統計結果快取至本地 metadata 資料庫，以避免重複掃描大量 JSON 檔案。

#### Scenario: 快取命中
- **WHEN** 查詢的 OpenCode session 非 live 狀態，且 metadata DB 中有快取
- **THEN** 直接回傳快取結果，不重新掃描 JSON 檔案

#### Scenario: 快取失效
- **WHEN** session 為 live 狀態，或 metadata DB 無快取
- **THEN** 重新掃描計算，並將結果寫入快取
