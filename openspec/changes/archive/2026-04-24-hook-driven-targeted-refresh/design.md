## Context

SessionHub 目前的 Copilot session 刷新流程有兩條路徑：

1. **Hook Bridge 路徑**（優先）：Copilot CLI hook 觸發 → 寫入 `~/.config/SessionHub/provider-bridge/copilot.jsonl` → bridge file watcher 偵測到 → 發出 `copilot-sessions-updated` 事件 → 前端全量 invalidate React Query → 呼叫 `get_sessions` 全掃
2. **File Watcher 路徑**（fallback）：監聽 `~/.copilot/session-state/` 目錄的 `workspace.yaml` 變化 → 比對 snapshot → 發出 `copilot-sessions-updated` → 同上

兩條路徑在 `watcher.rs` 中是互斥的（hook bridge 已安裝時，file watcher 自動停用）。

**瓶頸**：無論哪條路徑，最終都呼叫 `get_sessions`，它在 session 數量多時會執行相對耗時的目錄掃描（增量掃描也需比對所有 mtime）。Hook 事件的 bridge record 已包含 `cwd`，但目前被忽略，直接觸發全量重整。

## Goals / Non-Goals

**Goals:**

- 當 hook bridge 觸發且 bridge record 含有 `cwd` 時，改為**定向刷新**：只更新對應的單一 session
- 新增 `get_session_by_cwd` Tauri command，僅掃描一個 session 目錄
- 後端發出兩種事件：`copilot-session-targeted`（含 cwd + sessionId）和原有 `copilot-sessions-updated`（全量 fallback）
- 前端收到 targeted 事件時，使用 `queryClient.setQueryData` 精準更新單一 session，不觸發全量 invalidate

**Non-Goals:**

- 不修改 OpenCode 的刷新邏輯（影響範圍不同）
- 不修改 File Watcher fallback 路徑（已被 hook bridge 取代時才會用到）
- 不做 session 列表的虛擬捲動或懶載入（獨立性能議題）
- 不修改 sessionId 生成方式

## Decisions

### 決策 1：用 cwd 解析 sessionId 的方式

**選項 A（採用）**：在後端 `get_session_by_cwd` 中掃描 `session-state/` 目錄，找到 `workspace.yaml` 含有匹配 cwd 的 session，回傳完整 `SessionInfo`。

**選項 B**：在前端維護 cwd → sessionId 的 mapping，收到 cwd 後直接從現有快取查 sessionId 再觸發單一 refetch。

**採用 A 的理由**：後端已有掃描邏輯，且 `ProviderCache` 中已有 mtime 快取可加速查找；前端 cache 可能過期，後端保證資料一致性。

---

### 決策 2：事件 payload 結構

定義新的 `copilot-session-targeted` 事件 payload：
```json
{
  "sessionId": "abc123",
  "cwd": "/path/to/project",
  "eventType": "session.started"
}
```

若後端無法解析 cwd 對應的 sessionId（例如 session 尚未建立），則不發此事件，改發 `copilot-sessions-updated`（全量 fallback）。

**理由**：前端需要 sessionId 才能定向更新 React Query cache；全量 fallback 確保新 session 也能被正確偵測到。

---

### 決策 3：前端 React Query 更新方式

收到 `copilot-session-targeted` 後，不呼叫 `invalidateQueries`，而是：
1. 呼叫 `get_session_by_cwd(cwd)` 取得最新 `SessionInfo`
2. 使用 `queryClient.setQueryData(["sessions", ...], updater)` 只替換對應 session
3. 若 `get_session_by_cwd` 回傳 null → fallback 全量 invalidate

**理由**：`setQueryData` 不觸發 HTTP/IPC 重新請求，直接更新快取；比 invalidate 後等 refetch 快且不閃爍。

## Risks / Trade-offs

- **Race condition**：兩個 hook 事件快速連發時，targeted 更新可能互相覆蓋。緩解：保留 `PROVIDER_REFRESH_DEDUP_MS = 1500ms` dedup 機制，短時間內多個事件只發一次 targeted 更新（取最後一個 cwd）。
- **cwd 不匹配**：hook 的 cwd 是 Copilot CLI 的工作目錄，不一定等於 `workspace.yaml` 中的 cwd 欄位。緩解：`get_session_by_cwd` 做大小寫不敏感比對，並支援路徑正規化（normalize separators）。
- **新 session 尚未建立**：`sessionStart` 觸發時 `workspace.yaml` 可能還未存在。緩解：找不到 session 時自動 fallback 全量刷新，確保新 session 被加入列表。
- **向後相容**：舊版前端（未監聽新事件）仍接收 `copilot-sessions-updated`，行為不變。

## Migration Plan

1. 後端加入新 command 與事件，保留原事件 → 零 breaking change
2. 前端加入新事件監聽，現有 `copilot-sessions-updated` 監聽保留為 fallback
3. 不需要資料庫 migration 或設定變更
4. Rollback：移除新事件監聽即可，其餘程式碼不受影響

## Open Questions

- 是否要在 `session_mtimes` 快取中建立 cwd → sessionId 的反查索引以加速解析？（目前增量掃描已有 mtime 比對，可順帶建立，但非必要）
