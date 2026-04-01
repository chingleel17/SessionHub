## ADDED Requirements

### Requirement: Filesystem 事件 debounce 機制

系統 SHALL 對不同 provider 的 filesystem 事件分別套用 debounce，避免短暫大量事件造成重複掃描。

#### Scenario: Copilot 事件 debounce

- **WHEN** Copilot session-state 目錄在短時間內收到多個 filesystem 事件
- **THEN** 系統以 800ms debounce 合併，延遲後執行一次增量掃描
- **AND** debounce 期間的後續事件重置計時器

#### Scenario: OpenCode 事件 debounce

- **WHEN** OpenCode WAL 目錄在短時間內收到多個事件
- **THEN** 系統以 500ms debounce 合併，延遲後執行一次 OpenCode 增量掃描

#### Scenario: 跨 provider 事件不干擾

- **WHEN** Copilot 與 OpenCode 同時有 filesystem 事件
- **THEN** 兩個 debounce timer 獨立運行，互不影響

### Requirement: 事件類型過濾

系統 SHALL 只處理與 session 相關的事件類型，過濾無關事件。

#### Scenario: Copilot 相關事件

- **WHEN** watcher 收到 Copilot session-state 目錄內的事件
- **THEN** 只處理 workspace.yaml 的 create / modify 事件，以及子目錄的 create / delete
- **AND** 忽略其他檔案類型的事件（如 .lock、temp 檔）
