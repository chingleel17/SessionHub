## Why

首頁看板的「進行中/等待回應/閒置」數量與底部狀態列長期不一致，原因是兩處使用不同計算路徑（`sessionStatsMap.isLive` + 時間推算 vs `activityStatusMap`），且 `activityStatusQuery` 每 30 秒輪詢一次 IPC，造成不必要的開銷與視覺抖動。Claude hook 事件已能提供精確的即時狀態，應以此取代輪詢並統一計算來源。

## What Changes

- 將 `activeSessions / waitingSessions / idleSessions / doneSessions` 的計算統一改為以 `activityStatusMap` 為唯一來源（移除 `sessionStatsMap.isLive` + 時間判斷的舊路徑）
- `activityStatusQuery` 移除 `refetchInterval: 30_000`，改為事件驅動更新；若 Claude hook 未安裝則自動 fallback 回輪詢
- 後端 `claude-activity-hint` Tauri event payload 擴充，攜帶完整 `SessionActivityStatus`（含 `status` / `detail` / `lastActivityAt`），讓前端不需再發 IPC 即可直接更新 map
- Claude hook scripts 新增 `.sh` 版本（Git Bash + jq），`claude.rs` 產生 hook command 時：`command` 欄位使用 sh 腳本，`commandWindows` 欄位保留 `.ps1` 作為 fallback
- 設定 UI `SettingsView` 在顯示 Claude integration 安裝入口前偵測 `jq` 是否可用，不可用時顯示提示訊息但不阻擋安裝流程

## Capabilities

### New Capabilities

- `hook-driven-activity-status`: Claude hook 事件（SessionStart、UserPromptSubmit、PreToolUse、PostToolUse、Stop）驅動前端 `activityStatusMap` 即時更新，後端 activity-hint 事件攜帶完整狀態資訊
- `unified-session-status-count`: 前端狀態計數（進行中/等待回應/閒置/已完成）統一以 `activityStatusMap` 為單一來源，去除 `sessionStatsMap.isLive` + 時間推算的冗餘路徑
- `sh-hook-scripts`: Claude hook scripts 增加 `.sh` 版本（模組化，依賴 Git Bash + jq），`claude.rs` 同時維護 `command`（sh）與 `commandWindows`（ps1）兩種指令
- `jq-dependency-check`: 安裝 Claude integration 前於 UI 偵測 `jq` 可用性，不可用時顯示安裝指引

### Modified Capabilities

（無現有 spec 層級行為變更）

## Impact

- `src/App.tsx`：移除 `activityStatusQuery.refetchInterval`、新增 `claude-activity-hint` 完整 payload 處理、狀態計數 useMemo 重構
- `src-tauri/src/provider/claude.rs`：`render_claude_hook_command` 改為同時產生 `command`（sh）與 `commandWindows`（ps1）
- `src-tauri/src/watcher.rs` / bridge 處理：`claude-activity-hint` event payload 擴充
- `.claude/hooks/`：新增 `modules/*.sh`、`on-*.sh` 共 8 支 shell 腳本
- `src/components/SettingsView.tsx`：新增 jq 偵測 IPC 呼叫與 UI 提示
- `src-tauri/src/commands/tools.rs` 或新 command：新增 `check_jq_available` Tauri command
