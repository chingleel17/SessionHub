# unified-session-status-count Specification

## Purpose
TBD - created by archiving change hook-driven-status-sync. Update Purpose after archive.
## Requirements
### Requirement: 狀態計數以 activityStatusMap 為唯一來源，範圍限定當前看板週期
`activeSessions`、`waitingSessions`、`idleSessions`、`doneSessions` 的計算 SHALL 僅以當前看板週期內的 session（`filteredDashboardSessions`，即更新時間落在所選「本周 / 本月」範圍者）為計算範圍，並以 `activityStatusMap` 判斷各 session 狀態，不得使用 `sessionStatsMap.isLive` 或 `updatedAt` 時間差推算狀態。計算範圍限定看板週期，以避免狀態列顯示歷史全部 session 而與看板數字嚴重落差。

#### Scenario: 計數範圍與看板週期一致
- **WHEN** 看板選擇「本周」週期，`filteredDashboardSessions` 含 M 個 session
- **THEN** active/waiting/idle/done 四項計數總和不超過 M（僅統計這 M 個 session）
- **AND** 切換為「本月」時，計數範圍同步改為本月週期內的 session

#### Scenario: active 計數正確映射
- **WHEN** `filteredDashboardSessions` 中有 N 個 session 於 `activityStatusMap` 的 `status="active"`
- **THEN** `activeSessions === N`

#### Scenario: waiting 計數正確映射
- **WHEN** `filteredDashboardSessions` 中有 N 個 session 於 `activityStatusMap` 的 `status="waiting"`
- **THEN** `waitingSessions === N`

#### Scenario: idle 計數正確映射
- **WHEN** `filteredDashboardSessions` 中有 N 個 session 於 `activityStatusMap` 的 `status="idle"`
- **THEN** `idleSessions === N`

#### Scenario: done 計數維持以 isArchived 判斷
- **WHEN** `filteredDashboardSessions` 中某 session 的 `isArchived=true`
- **THEN** 該 session 計入 `doneSessions`，不計入其他欄位

#### Scenario: activityStatusMap 無資料時計數為零
- **WHEN** app 剛啟動，`activityStatusMap` 為空
- **THEN** active/waiting/idle 計數皆為 0，不顯示錯誤

### Requirement: activityStatusQuery 事件驅動，hook 已安裝時不輪詢
當 Claude integration 狀態為 `installed` 時，`activityStatusQuery` SHALL 不設定 `refetchInterval`（純事件驅動更新）；integration 未安裝時 SHALL fallback 為每 30 秒輪詢一次。

#### Scenario: hook 已安裝時不產生定時 IPC
- **WHEN** `providerIntegrations` 中 Claude 的 state 為 `installed`
- **THEN** `activityStatusQuery` 的 `refetchInterval` 為 `false`

#### Scenario: hook 未安裝時維持輪詢
- **WHEN** Claude integration 未安裝（state 不為 `installed`）
- **THEN** `activityStatusQuery` 每 30 秒自動 refetch 一次

#### Scenario: mount 時執行一次查詢取得初始值
- **WHEN** app 首次載入，hook 已安裝
- **THEN** `activityStatusQuery` 在 mount 時執行一次（`refetchOnMount: true`），建立初始狀態

