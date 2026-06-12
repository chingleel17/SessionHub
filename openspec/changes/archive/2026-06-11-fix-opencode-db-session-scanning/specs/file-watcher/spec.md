## MODIFIED Requirements

### Requirement: OpenCode WAL watcher

系統 SHALL 監看 OpenCode SQLite database 與 WAL 檔案的異動以偵測新 session 或更新。

#### Scenario: WAL 檔案異動

- **WHEN** OpenCode `opencode.db-wal` 發生寫入
- **THEN** 系統觸發 OpenCode session 增量掃描

#### Scenario: 主 DB 檔異動

- **WHEN** OpenCode `opencode.db` 發生足以影響 session 清單的異動
- **THEN** 系統觸發 OpenCode session 增量掃描

### Requirement: Provider 事件區分

系統 SHALL 對不同 provider 的 filesystem 事件分別處理，互不干擾。

#### Scenario: Copilot 事件不影響 OpenCode

- **WHEN** Copilot session-state 有事件
- **THEN** 系統僅觸發 Copilot session 增量掃描，不重新掃描 OpenCode

#### Scenario: OpenCode DB 事件不依賴舊 JSON session 目錄

- **WHEN** OpenCode 的有效變更只出現在 `opencode.db` / WAL
- **THEN** 系統仍可觸發 OpenCode session refresh
- **AND** 不要求舊 `storage/session` JSON 檔一定同時變更
