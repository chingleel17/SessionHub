## ADDED Requirements

### Requirement: SessionHub 支援從 OpenCode SQLite storage 讀取 session

系統 SHALL 支援從 `opencode.db` 的 session 與 project 資料讀取 OpenCode session metadata，而不僅依賴舊的 JSON storage 檔案。

#### Scenario: 最新 session 僅存在於資料庫
- **WHEN** OpenCode 最新 session 已寫入 `opencode.db`
- **AND** 舊的 `storage/session/<projectId>/*.json` 不存在對應檔案
- **THEN** SessionHub 仍可掃描並回傳該 session

#### Scenario: project 與 session 透過 DB 關聯
- **WHEN** SessionHub 讀取 OpenCode session
- **THEN** 系統以 session row 與 project row 關聯出 cwd / worktree 等顯示資訊

### Requirement: OpenCode DB 不可用時退回舊 JSON storage

系統 SHALL 在 `opencode.db` 不存在或無法讀取時，退回使用舊 JSON storage 掃描 OpenCode session。

#### Scenario: DB 缺失
- **WHEN** `opencode.db` 不存在或缺少必要資料表
- **THEN** 系統退回舊 JSON storage 掃描
- **AND** 不因此讓整體 session 查詢失敗
