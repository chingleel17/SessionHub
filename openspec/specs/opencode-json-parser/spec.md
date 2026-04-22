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

### Requirement: 診斷與修復 OpenCode session 路徑導向問題

系統 SHALL 確保 `get_session_stats_internal` 在 OpenCode session 下，正確將 `session_dir`（前端傳入的 `sessionDir` 欄位）轉換為 `storage/message/{sessionID}/` 路徑後再傳入統計計算函式。若 `sessionDir` 指向的是 session JSON 路徑（`<opencodeRoot>/session/<projectID>/ses_xxx.json`）而非 message 目錄，系統 SHALL 自動推導出正確的 message 目錄路徑。

#### Scenario: sessionDir 為正確的 message 目錄

- **WHEN** 前端傳入的 `sessionDir` 已是 `<opencodeRoot>/message/{ses_xxx}/`
- **THEN** 系統 SHALL 直接使用此路徑掃描 message JSON 檔案，正確回傳統計資料

#### Scenario: sessionDir 為 session JSON 檔案路徑

- **WHEN** 前端傳入的 `sessionDir` 為 `<opencodeRoot>/session/<projectID>/ses_xxx.json`
- **THEN** 系統 SHALL 以 `sessionID` 組合出 `<opencodeRoot>/message/{sessionID}/` 並掃描，不回傳空統計

#### Scenario: message 目錄不存在

- **WHEN** `<opencodeRoot>/message/{sessionID}/` 目錄不存在
- **THEN** 系統 SHALL 降為全量掃描 `storage/message/` 並過濾 `sessionID` 欄位，不直接回傳預設空值

### Requirement: opencode_root 設定路徑合法性驗證

系統 SHALL 在 OpenCode session 路徑推導時，驗證 `opencode_root`（設定中的根目錄）是否含有 `storage/` 子目錄；若不存在，應在統計結果的 parse_error 欄位回傳明確的診斷訊息。

#### Scenario: opencode_root 設定錯誤

- **WHEN** `AppSettings.opencode_root` 指向不含 `storage/` 的目錄
- **THEN** 統計回傳預設空值，且 `parse_error` 欄位 SHALL 包含 `"opencode storage directory not found: <path>"` 訊息

#### Scenario: opencode_root 設定正確

- **WHEN** `AppSettings.opencode_root` 指向含有 `storage/message/` 的目錄
- **THEN** 系統正常掃描並回傳統計資料，`parse_error` 為 null
