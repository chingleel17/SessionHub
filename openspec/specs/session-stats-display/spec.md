## ADDED Requirements

### Requirement: Session 對話時長自動換算單位

系統 SHALL 在 SessionStatsBadge 顯示對話時長時，自動換算為適合閱讀的單位：不足 60 分鐘顯示為分鐘（`Xm`），達 60 分鐘以上換算為小時加分鐘（`XhXm` 或 `Xh`）。

#### Scenario: 時長不足 60 分鐘

- **WHEN** session 的 `durationMinutes` 小於 60
- **THEN** 顯示為 `{durationMinutes}m`（例如 `45m`）

#### Scenario: 時長超過 60 分鐘

- **WHEN** session 的 `durationMinutes` 超過 60
- **THEN** 顯示為 `XhYm` 格式，若可整除則省略分鐘（例如 `2h`）

### Requirement: Sidebar 收合時顯示版本號

系統 SHALL 在 sidebar 收合狀態下於 footer 顯示縮短版本號，不得完全隱藏。

#### Scenario: Sidebar 收合時

- **WHEN** sidebar 處於收合狀態
- **THEN** footer 顯示縮短版本號（例如 `v0.1`），hover 時 tooltip 顯示完整版本號

### Requirement: SessionStatsBadge 顯示欄位

SessionStatsBadge SHALL 以緊湊 badge 格式顯示 session 統計摘要。

#### Scenario: Badge 欄位

- **WHEN** session stats 已載入
- **THEN** badge 顯示：互動次數、output tokens（K 格式）、時長（Xm / XhYm）
- **AND** 若 isLive 為 true，顯示「LIVE」標記

### Requirement: SessionStatsPanel 顯示完整統計

SessionStatsPanel SHALL 顯示 session 的完整詳細統計。

#### Scenario: Panel 顯示欄位

- **WHEN** 使用者開啟 stats 詳情 panel
- **THEN** panel 顯示：output tokens、input tokens（大於 0 時）、互動次數、工具呼叫次數、時長、reasoning count、models used 列表、tool breakdown 表格（依呼叫次數降冪排列）

#### Scenario: inputTokens 為 0 時不顯示

- **WHEN** session 的 `inputTokens` 等於 0
- **THEN** SessionStatsPanel 不顯示 inputTokens 欄位

#### Scenario: Live session 說明

- **WHEN** session 的 `isLive` 為 true
- **THEN** panel 顯示「Session 進行中」提示，統計標示為當前快照

### Requirement: OpenCode session 統計資料完整性

系統 SHALL 確保 OpenCode session 的統計資料從 JSON storage 正確讀取，顯示欄位與 Copilot session 一致。

#### Scenario: OpenCode session 統計顯示

- **WHEN** 使用者查看 OpenCode session 的統計
- **THEN** 系統顯示非零的 outputTokens（從 JSON storage 讀取）
- **AND** 顯示正確的 interactionCount、toolCallCount、modelsUsed、durationMinutes

#### Scenario: OpenCode session 無訊息資料時

- **WHEN** OpenCode session 的 message 目錄為空或不存在
- **THEN** 統計顯示全零值，不顯示錯誤訊息
