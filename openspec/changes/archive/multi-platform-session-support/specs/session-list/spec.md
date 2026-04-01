## MODIFIED Requirements

### Requirement: 讀取所有 session
系統 SHALL 掃描所有已啟用平台的 session 資料來源，包含 Copilot 根目錄下 `session-state/` 子目錄的 `workspace.yaml` 以及 OpenCode SQLite 資料庫，合併回傳統一結構化 session 清單。每筆 session 包含 `provider` 欄位標識來源。

#### Scenario: 正常讀取多平台 session 列表
- **WHEN** 使用者開啟應用程式且 Copilot 與 OpenCode 資料來源皆存在
- **THEN** 系統顯示所有解析成功的 session，每筆包含：id、summary、cwd、createdAt、updatedAt、provider。Copilot session 的 provider 為 `"copilot"`，OpenCode session 的 provider 為 `"opencode"`

#### Scenario: 僅 Copilot 可用
- **WHEN** 使用者僅啟用 Copilot 或 OpenCode 資料庫不存在
- **THEN** 系統僅顯示 Copilot session，行為與擴展前相同

## ADDED Requirements

### Requirement: SessionInfo provider 欄位
系統 SHALL 在 `SessionInfo` 結構中包含 `provider` 字串欄位，標識該 session 的來源平台。

#### Scenario: Copilot session 的 provider 值
- **WHEN** session 來自 Copilot session-state 目錄掃描
- **THEN** `provider` 欄位值為 `"copilot"`

#### Scenario: OpenCode session 的 provider 值
- **WHEN** session 來自 OpenCode 資料庫查詢
- **THEN** `provider` 欄位值為 `"opencode"`

### Requirement: 多來源合併排序
系統 SHALL 將不同平台的 session 合併後依 `updatedAt` 降序排序。

#### Scenario: 混合排序
- **WHEN** Copilot 有 session A（updated 10:00）和 OpenCode 有 session B（updated 11:00）
- **THEN** 合併列表中 session B 排在 session A 之前
