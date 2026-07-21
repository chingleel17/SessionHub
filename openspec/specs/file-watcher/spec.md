## ADDED Requirements

### Requirement: 目錄變更即時偵測

系統 SHALL 以 provider bridge 事件作為 session 更新的主要來源，並保留 OS filesystem watch 作為 fallback。fallback watcher MUST 僅監看關鍵路徑與檔案，且在通知前端前先完成事件過濾與 cheap verify。

#### Scenario: provider bridge 可用時的即時更新

- **WHEN** Copilot hook、OpenCode plugin 或 Codex hook 發送標準化 bridge 事件
- **THEN** 系統以該 bridge 事件觸發 session refresh
- **AND** 不以一般 filesystem event 作為主要刷新依據

#### Scenario: bridge 不可用時使用 fallback watcher

- **WHEN** provider integration 尚未安裝、失效或需要手動設定
- **THEN** 系統啟用 fallback watcher 維持基本即時更新
- **AND** 只有在 cheap verify 判定 session 清單可能變更時才通知前端

#### Scenario: 無關檔案事件被忽略

- **WHEN** watcher 收到與 session 清單無關的檔案事件
- **THEN** 系統忽略該事件
- **AND** 不更新最近活動或同步狀態

### Requirement: OpenCode WAL watcher

系統 SHALL 監看 OpenCode SQLite database 與 WAL 檔案的異動以偵測新 session 或更新。

#### Scenario: WAL 檔案異動

- **WHEN** OpenCode `opencode.db-wal` 發生寫入
- **THEN** 系統觸發 OpenCode session 增量掃描

#### Scenario: 主 DB 檔異動

- **WHEN** OpenCode `opencode.db` 發生足以影響 session 清單的異動
- **THEN** 系統觸發 OpenCode session 增量掃描

### Requirement: 事件 debounce

系統 SHALL 對 filesystem 事件進行 debounce，避免短時間大量事件造成重複掃描。

#### Scenario: Copilot filesystem 事件 debounce

- **WHEN** Copilot session-state 目錄收到多個連續事件
- **THEN** 系統以 800ms debounce 延遲後，執行一次增量掃描

#### Scenario: OpenCode WAL 事件 debounce

- **WHEN** OpenCode WAL 在短時間內收到多個事件
- **THEN** 系統以 500ms debounce 延遲，合併為一次掃描通知

### Requirement: Provider 事件區分

系統 SHALL 對不同 provider 的 filesystem 與 bridge 事件分別處理，互不干擾。

#### Scenario: Copilot 事件不影響 OpenCode

- **WHEN** Copilot session-state 有事件
- **THEN** 系統僅觸發 Copilot session 增量掃描，不重新掃描 OpenCode

#### Scenario: OpenCode DB 事件不依賴舊 JSON session 目錄

- **WHEN** OpenCode 的有效變更只出現在 `opencode.db` / WAL
- **THEN** 系統仍可觸發 OpenCode session refresh
- **AND** 不要求舊 `storage/session` JSON 檔一定同時變更

#### Scenario: Codex bridge 事件不影響其他 provider

- **WHEN** Codex bridge 檔案有新事件
- **THEN** 系統僅發出 `codex-sessions-updated` 或等效 Codex 刷新流程
- **AND** 不重新掃描 Copilot 或 OpenCode
