## ADDED Requirements

### Requirement: ScanCache 結構存在於 AppState
系統 SHALL 在 Tauri AppState 中包含一個 `ScanCache` 結構，持有兩個 provider（Copilot、OpenCode）各自的 `Mutex<Option<ProviderCache>>`。`ProviderCache` SHALL 包含以下欄位：`sessions: Vec<SessionInfo>`、`session_mtimes: HashMap<String, i64>`（Copilot 用）、`last_full_scan_at: Instant`、`last_cursor: i64`（OpenCode 用）。

#### Scenario: App 啟動時快取為空
- **WHEN** App 首次啟動
- **THEN** `ScanCache` 中兩個 provider 的快取均為 `None`，下一次 `get_sessions` 呼叫必須執行全掃

#### Scenario: 全掃後快取已填充
- **WHEN** `get_sessions` 完成一次全掃
- **THEN** 對應 provider 的 `ProviderCache` 從 `None` 變為 `Some`，`last_full_scan_at` 設定為當前時刻

### Requirement: 全掃閾值觸發機制
系統 SHALL 在以下任一條件成立時執行全掃並重置快取：
1. `ProviderCache` 為 `None`
2. `last_full_scan_at.elapsed()` 超過 30 分鐘
3. `get_sessions` 收到 `force_full: Some(true)` 參數

#### Scenario: 距上次全掃超過 30 分鐘
- **WHEN** `get_sessions` 被呼叫且 `last_full_scan_at.elapsed() > 30 minutes`
- **THEN** 系統執行完整掃描（讀取所有目錄 / 查詢所有 session），並將 `last_full_scan_at` 重置為當前時刻

#### Scenario: 封存操作後強制全掃
- **WHEN** 前端呼叫 `get_sessions` 並傳入 `force_full: true`
- **THEN** 系統無視 `last_full_scan_at`，執行全掃並重置快取

#### Scenario: 30 分鐘內的正常查詢使用快取
- **WHEN** `get_sessions` 被呼叫且快取不為 `None` 且距上次全掃未超過 30 分鐘 且 `force_full` 為 `None` 或 `false`
- **THEN** 系統使用現有快取資料，不重新掃描全部項目
