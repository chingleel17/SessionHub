## ADDED Requirements

### Requirement: OpenCode session 資料讀取
系統 SHALL 從 OpenCode SQLite 資料庫（`opencode.db`）以唯讀方式讀取 session 與 project 資料，並映射至共用 `SessionInfo` 結構。

#### Scenario: 成功讀取 OpenCode session
- **WHEN** 使用者啟用 OpenCode provider 且 `opencode.db` 存在於設定路徑
- **THEN** 系統查詢 `session` 與 `project` 表，回傳所有 session 資訊，每筆包含：id、title(→summary)、worktree(→cwd)、time_created、time_updated、slug、provider="opencode"

#### Scenario: OpenCode 資料庫不存在
- **WHEN** 使用者啟用 OpenCode provider 但 `opencode.db` 不存在於設定路徑
- **THEN** 系統靜默忽略，不產生錯誤，回傳空 session 清單

#### Scenario: OpenCode 資料庫 schema 不相容
- **WHEN** OpenCode 資料庫結構與預期不符（如缺少欄位或表不存在）
- **THEN** 系統記錄警告訊息並靜默忽略，不影響其他 provider 的正常運作

### Requirement: OpenCode 時間戳轉換
系統 SHALL 將 OpenCode 的 unix timestamp（毫秒）轉換為 ISO 8601 格式字串，與 Copilot session 的時間格式統一。

#### Scenario: 時間戳正確轉換
- **WHEN** OpenCode session 的 `time_created` 為 `1774974837271`（unix ms）
- **THEN** 系統將其轉換為對應的 ISO 8601 字串（如 `"2026-03-31T..."`)

### Requirement: OpenCode 封存 session 處理
系統 SHALL 根據 OpenCode session 的 `time_archived` 欄位判斷封存狀態。

#### Scenario: 已封存的 OpenCode session
- **WHEN** OpenCode session 的 `time_archived` 欄位非 NULL
- **THEN** 系統將該 session 的 `isArchived` 設為 `true`

#### Scenario: 未封存的 OpenCode session
- **WHEN** OpenCode session 的 `time_archived` 欄位為 NULL
- **THEN** 系統將該 session 的 `isArchived` 設為 `false`

### Requirement: OpenCode 唯讀存取
系統 SHALL 以唯讀模式開啟 OpenCode 資料庫連線，不得寫入或修改任何資料。

#### Scenario: 唯讀連線
- **WHEN** 系統開啟 OpenCode 資料庫
- **THEN** 連線使用 `SQLITE_OPEN_READ_ONLY` flag，任何寫入操作 SHALL 被拒絕
