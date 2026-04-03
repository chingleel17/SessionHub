## ADDED Requirements

### Requirement: Session 活動狀態批次查詢

系統 SHALL 提供批次查詢多個 session 活動狀態的 Tauri command，一次呼叫回傳所有指定 sessions 的狀態。

#### Scenario: 批次查詢成功

- **WHEN** 前端呼叫 `get_session_activity_statuses(session_ids)` 並傳入 session ID 陣列
- **THEN** 系統回傳對應每個 ID 的 `SessionActivityStatus` 陣列
- **AND** 每個 status 包含 `sessionId`, `status`（idle/active/waiting/done）, `detail`（可選的細節標籤）, `lastActivityAt`（最後活動時間）

#### Scenario: 查詢包含無效 session ID

- **WHEN** 傳入的 session_ids 中有不存在的 ID
- **THEN** 系統對該 ID 回傳狀態為 `idle`，不中斷其他 session 的查詢

### Requirement: Copilot Session 狀態推斷（基於 events.jsonl）

系統 SHALL 透過讀取 Copilot session 目錄的 `events.jsonl` 末尾事件推斷活動狀態。`events.jsonl` 每行為一個 JSON，包含 `type`（事件類型）與 `timestamp`（ISO 字串）。

#### Scenario: 推斷為 done 狀態

- **WHEN** events.jsonl 最後數行中出現 `type: "session.task_complete"` 或 `type: "session.shutdown"`
- **THEN** 系統回傳狀態 `done`，detail 為 `completed`

#### Scenario: 推斷為 waiting 狀態（等待用戶輸入）

- **WHEN** events.jsonl 最後一個事件的 type 為 `"assistant.turn_end"`
- **AND** 該 timestamp 距現在不超過 2 小時
- **THEN** 系統回傳狀態 `waiting`（AI 已完成回應，等待用戶下一個指令）

#### Scenario: 推斷為 active 狀態

- **WHEN** events.jsonl 最後一個事件的 type 為 `"tool.execution_start"` 或 `"assistant.turn_start"`
- **AND** 該 timestamp 距現在不超過 30 分鐘
- **THEN** 系統回傳狀態 `active`

#### Scenario: 推斷為 idle 狀態

- **WHEN** events.jsonl 存在但最後事件的 timestamp 距現在超過 30 分鐘，且不滿足 done 條件
- **THEN** 系統回傳狀態 `idle`

#### Scenario: Copilot Active 工具細節推斷

- **WHEN** 最後幾行中出現 `type: "tool.execution_start"`
- **AND** `data.toolName` 包含 `file`、`read`、`write`、`edit`、`patch`
- **THEN** detail 為 `file_op`
- **WHEN** `data.toolName` 包含 `agent`、`task`、`subagent`
- **THEN** detail 為 `sub_agent`
- **WHEN** `data.toolName` 為其他工具
- **THEN** detail 為 `tool_call`

### Requirement: OpenCode Session 狀態推斷（基於 storage JSON 檔案）

系統 SHALL 透過讀取 OpenCode storage 目錄下的 message 與 part JSON 檔案推斷活動狀態。message 檔位於 `<opencodeRoot>/storage/message/<sessionId>/msg_*.json`，part 檔位於 `<opencodeRoot>/storage/part/<messageId>/prt_*.json`。

#### Scenario: 推斷為 waiting 狀態

- **WHEN** 該 session 最新的 msg_*.json 的 `role` 為 `"assistant"` 且 `finish` 為 `"stop"`
- **AND** `time.completed` 距現在不超過 2 小時
- **THEN** 系統回傳狀態 `waiting`

#### Scenario: 推斷為 active 狀態（用戶已發送，AI 未回應）

- **WHEN** 最新 msg_*.json 的 `role` 為 `"user"`
- **AND** 不存在後續 role=assistant 的 message
- **AND** `time.created` 距現在不超過 30 分鐘
- **THEN** 系統回傳狀態 `active`，detail 為 `working`

#### Scenario: 推斷為 done 狀態

- **WHEN** session 的 `time_archived` 不為 null（OpenCode 已封存）
- **OR** 最後 message 距現在超過 24 小時
- **THEN** 系統回傳狀態 `done`

#### Scenario: OpenCode Active 工具細節推斷

- **WHEN** 最新 assistant message 的對應 prt_*.json 中存在 `type: "tool"` 的 part
- **AND** `tool` 欄位為 `"edit"`、`"write"`、`"file"`、`"patch"` 等
- **THEN** detail 為 `file_op`
- **WHEN** `tool` 欄位為 `"task"`、`"call_omo_agent"`、`"subtask"` 等
- **THEN** detail 為 `sub_agent`
- **WHEN** 最新 part 的 `type` 為 `"reasoning"`
- **THEN** detail 為 `thinking`

### Requirement: 狀態偵測的精確度說明

系統 SHALL 以「最後已完成的動作」為基礎推斷狀態，而非即時 streaming 監控。

#### Scenario: 說明偵測精度限制

- **WHEN** 系統顯示 session 活動狀態
- **THEN** 狀態反映的是「最後一個寫入檔案的事件」，而非「本毫秒正在執行的操作」
- **AND** 此精度等同於 VibePulse 等同類工具的做法（基於靜態檔案推斷）

### Requirement: 狀態查詢效能限制

系統 SHALL 對每個 session 的狀態查詢進行資料讀取量限制。

#### Scenario: 限制讀取行數（Copilot）

- **WHEN** 讀取 events.jsonl 推斷狀態
- **THEN** 系統只讀取檔案末尾最多 30 行，不讀取整個檔案

#### Scenario: 限制讀取數量（OpenCode）

- **WHEN** 讀取 message/part JSON 檔案推斷狀態
- **THEN** 系統只讀取最新的 2 個 message 檔案與其對應的最新 5 個 part 檔案
