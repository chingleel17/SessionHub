## ADDED Requirements

### Requirement: 解析 OpenCode JSON storage 取得 session 統計

系統 SHALL 掃描 OpenCode 的 `storage/message/<sessionID>/msg_*.json` 與 `storage/part/<messageID>/prt_*.json` 以計算 session 統計資料。

#### Scenario: Token 統計來源

- **WHEN** 系統解析 OpenCode session 統計
- **THEN** 從 role 為 `assistant` 的 msg\_\*.json 讀取 `tokens.input`、`tokens.output`、`tokens.reasoning`、`tokens.cache.read` 欄位並累計
- **AND** user 訊息不計入 token 統計，但每筆 user 訊息計入 `interactionCount`

#### Scenario: 工具呼叫統計

- **WHEN** 系統解析 prt\_\*.json
- **THEN** 從 type 為 `tool` 的 part 讀取 `state.tool` 欄位，累計至 `toolBreakdown`
- **AND** 若 `state.tool` 為空，以 `"unknown"` 記錄

#### Scenario: 時長計算

- **WHEN** session 下有至少兩筆訊息
- **THEN** 以最早與最晚訊息的 `time.created` 差值計算時長（分鐘）
- **AND** 單一訊息 session 的 `durationMinutes` 返回 0

#### Scenario: 模型列表

- **WHEN** 系統解析所有訊息
- **THEN** 收集所有 assistant 訊息的 `modelID` 去重，回傳 modelsUsed 陣列

#### Scenario: 快取統計

- **WHEN** OpenCode session 非 live 狀態且 metadata DB 有快取
- **THEN** 直接回傳快取結果，不重新掃描 JSON 檔案
