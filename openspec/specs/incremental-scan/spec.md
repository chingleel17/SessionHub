## ADDED Requirements

### Requirement: Copilot session 增量掃描

系統 SHALL 以 mtime 差異偵測僅掃描有變更的 Copilot session，而非每次全量重掃。

#### Scenario: 增量掃描流程

- **WHEN** filesystem 事件觸發 Copilot session 重新掃描
- **THEN** 系統比對 `session_mtimes`（HashMap<session_id, i64>）與當前 workspace.yaml 的 mtime
- **AND** 僅重新解析 mtime 有變化的 session
- **AND** 移除不再存在的 session（目錄已刪除）

#### Scenario: 全量掃描門檻

- **WHEN** 距離上次全量掃描超過 30 分鐘，或 mtime map 為空
- **THEN** 系統執行完整全量掃描並重建 mtime map

### Requirement: OpenCode cursor-based 增量掃描

系統 SHALL 以 cursor（最後掃描位置）進行 OpenCode session 增量掃描。

#### Scenario: OpenCode 增量掃描

- **WHEN** OpenCode WAL 異動觸發掃描
- **THEN** 系統從上次的 `last_cursor`（檔案位置或時間戳）起繼續掃描
- **AND** 只處理 cursor 之後的新 session / message 記錄

#### Scenario: cursor 重置

- **WHEN** last_cursor 為 0 或 OpenCode 根目錄結構有重大變更
- **THEN** 系統執行完整重掃，並重設 cursor

### Requirement: ScanCache 記憶體結構

ScanCache SHALL 儲存於 AppState，包含 Copilot 與 OpenCode 各自的 ProviderCache。

#### Scenario: ScanCache 結構

- **WHEN** AppState 初始化
- **THEN** ScanCache 包含：
  - `copilot: Mutex<Option<ProviderCache>>`
  - `opencode: Mutex<Option<ProviderCache>>`
- **AND** ProviderCache 包含：`sessions`、`session_mtimes`、`last_full_scan_at`、`last_cursor`
