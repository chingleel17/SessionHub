# hook-driven-activity-status Specification

## Purpose
TBD - created by archiving change hook-driven-status-sync. Update Purpose after archive.
## Requirements
### Requirement: claude-activity-hint payload 攜帶完整狀態
後端 `claude-activity-hint` Tauri event payload SHALL 包含 `sessionId`、`status`（active/waiting/idle）、`detail`（thinking/tool_call/working）、`lastActivityAt`（ISO 8601 UTC），讓前端無需額外 IPC 即可直接更新 `activityStatusMap`。

#### Scenario: SessionStart hook 觸發 active 狀態
- **WHEN** Claude 發出 SessionStart hook 事件
- **THEN** 後端發射 `claude-activity-hint`，payload 中 `status="active"`，`detail="thinking"`

#### Scenario: UserPromptSubmit hook 觸發 active 狀態
- **WHEN** 用戶提交 prompt，Claude 發出 UserPromptSubmit hook 事件
- **THEN** 後端發射 `claude-activity-hint`，payload 中 `status="active"`，`detail="thinking"`

#### Scenario: PreToolUse hook 觸發 active 工具狀態
- **WHEN** Claude 準備執行工具，發出 PreToolUse hook 事件
- **THEN** 後端發射 `claude-activity-hint`，payload 中 `status="active"`，`detail="tool_call"`

#### Scenario: PostToolUse hook 維持 active 狀態
- **WHEN** 工具執行完成，Claude 發出 PostToolUse hook 事件
- **THEN** 後端發射 `claude-activity-hint`，payload 中 `status="active"`，`detail="working"`

#### Scenario: Stop hook 正常結束變為 idle
- **WHEN** Claude session 正常結束（stopReason=normal），發出 Stop hook 事件
- **THEN** 後端發射 `claude-activity-hint`，payload 中 `status="idle"`，`detail=null`

#### Scenario: Stop hook 需要介入變為 waiting
- **WHEN** Claude session 因錯誤或中斷結束（stopReason=error 或 interrupt）
- **THEN** 後端發射 `claude-activity-hint`，payload 中 `status="waiting"`，`detail=null`

### Requirement: 前端收到 hint 直接 patch activityStatusMap
前端監聽 `claude-activity-hint` 時，若 payload 含有 `status` 欄位，SHALL 直接更新 `activityStatusMap` 中對應 `sessionId` 的 entry，不發送任何 IPC 呼叫。

#### Scenario: 事件抵達時 map 立即更新
- **WHEN** 前端收到帶有 `sessionId="abc"`, `status="active"` 的 `claude-activity-hint`
- **THEN** `activityStatusMap.get("abc").status` 立即變為 `"active"`，不等下次輪詢

#### Scenario: 缺少 status 的舊格式 payload 不觸發 patch
- **WHEN** 前端收到不含 `status` 欄位的 `claude-activity-hint`（舊版本向後相容）
- **THEN** 前端不更新 map，僅執行原有的 session targeted 邏輯

