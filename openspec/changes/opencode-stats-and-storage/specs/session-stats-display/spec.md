## MODIFIED Requirements

### Requirement: Session 統計顯示
系統 SHALL 在 SessionStatsBadge 與 SessionStatsPanel 中顯示 session 統計資料。對於 OpenCode session，統計資料 SHALL 從 JSON storage 讀取，顯示欄位與 Copilot session 一致。

#### Scenario: Copilot session 統計顯示（維持現有行為）
- **WHEN** 使用者查看 Copilot session 的統計
- **THEN** 系統顯示 outputTokens、interactionCount、toolCallCount、durationMinutes、modelsUsed、reasoningCount、toolBreakdown

#### Scenario: OpenCode session 統計顯示（修正後行為）
- **WHEN** 使用者查看 OpenCode session 的統計
- **THEN** 系統 SHALL 顯示非零的 outputTokens（從 JSON storage 讀取）
- **AND** 系統 SHALL 顯示正確的 interactionCount（user 訊息筆數）
- **AND** 系統 SHALL 顯示正確的 toolCallCount 與 toolBreakdown
- **AND** 系統 SHALL 顯示正確的 modelsUsed（各訊息的 modelID 去重）
- **AND** 系統 SHALL 顯示正確的 durationMinutes

#### Scenario: OpenCode session 無訊息資料時
- **WHEN** OpenCode session 的 message 目錄為空或不存在
- **THEN** 統計顯示全零值，不顯示錯誤訊息

## ADDED Requirements

### Requirement: 統計中顯示 inputTokens
系統 SHALL 在 SessionStatsPanel 中額外顯示 inputTokens 欄位（當值大於 0 時）。

#### Scenario: 顯示 input token 數
- **WHEN** session 的 `inputTokens` 大於 0
- **THEN** SessionStatsPanel 顯示 input token 計數，標籤為「輸入 Token」或對應 i18n key
- **AND** 格式與現有 outputTokens 顯示一致（例如 `1.2K`）

#### Scenario: inputTokens 為 0 時不顯示
- **WHEN** session 的 `inputTokens` 等於 0（如 Copilot session 目前不提取 input tokens）
- **THEN** SessionStatsPanel 不顯示 inputTokens 欄位，不影響其他統計項目
