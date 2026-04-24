# targeted-session-refresh Specification

## Purpose
TBD - created by archiving change hook-driven-targeted-refresh. Update Purpose after archive.
## Requirements
### Requirement: get_session_by_cwd command
後端 SHALL 提供 `get_session_by_cwd(cwd: String, copilot_root: Option<String>)` Tauri command，根據工作目錄路徑找到對應的 Copilot session，回傳 `Option<SessionInfo>`。
查詢邏輯：掃描 `session-state/` 目錄，讀取各 session 的 `workspace.yaml`，找出 cwd 欄位與輸入路徑大小寫不敏感且路徑分隔符正規化後相符的 session。

#### Scenario: cwd 對應到已存在的 session
- **WHEN** 呼叫 `get_session_by_cwd` 且 `session-state/` 中存在 cwd 相符的 workspace.yaml
- **THEN** 回傳該 session 的完整 `SessionInfo`（含 notes、tags、has_plan 等欄位）

#### Scenario: cwd 無對應 session
- **WHEN** 呼叫 `get_session_by_cwd` 且找不到任何 cwd 相符的 session
- **THEN** 回傳 `null`（前端收到後執行全量 fallback）

#### Scenario: cwd 路徑正規化比對
- **WHEN** hook 傳入的 cwd 使用正斜線（`/`），但 workspace.yaml 存的是反斜線（`\`）
- **THEN** 系統正規化後仍能正確比對，回傳對應 `SessionInfo`

---

### Requirement: copilot-session-targeted 事件
當 Copilot hook bridge 事件含有 `cwd` 且後端能解析出對應 sessionId 時，系統 SHALL 發出 `copilot-session-targeted` 事件，payload 包含 `{ sessionId: string, cwd: string, eventType: string }`。

#### Scenario: hook 事件含 cwd 且可解析
- **WHEN** bridge file watcher 偵測到新的 bridge record，且 record 的 `cwd` 非空且可對應到 session
- **THEN** 後端發出 `copilot-session-targeted`（含 sessionId、cwd、eventType），不發出 `copilot-sessions-updated`

#### Scenario: hook 事件 cwd 為空或無法解析
- **WHEN** bridge record 的 `cwd` 為空，或無法找到對應 session
- **THEN** 後端發出原有的 `copilot-sessions-updated`（全量 fallback），行為與現有實作一致

---

### Requirement: 前端定向 session 更新
前端 SHALL 監聽 `copilot-session-targeted` 事件，收到後僅更新 React Query cache 中對應的單一 session，不觸發全量 `invalidateQueries`。

#### Scenario: 成功定向更新
- **WHEN** 前端收到 `copilot-session-targeted` 事件（含 sessionId、cwd）
- **THEN** 呼叫 `get_session_by_cwd(cwd)` 取得最新 `SessionInfo`，並使用 `queryClient.setQueryData` 只替換 sessions 列表中對應的 session 資料

#### Scenario: get_session_by_cwd 回傳 null 時 fallback
- **WHEN** 前端呼叫 `get_session_by_cwd` 後收到 null
- **THEN** 執行全量 `invalidateQueries(["sessions", ...])` 作為 fallback，確保 UI 資料一致

#### Scenario: 全量刷新路徑不受影響
- **WHEN** 前端收到 `copilot-sessions-updated` 事件（原有全量事件）
- **THEN** 行為與現有實作完全相同（全量 invalidate React Query）

