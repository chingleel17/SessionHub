## Context

SessionHub 前端目前有兩套平行的 session 狀態計算路徑，導致首頁看板與底部狀態列顯示不一致：

1. **看板欄位計數**（`activeSessions` 等）：以 `sessionStatsMap[id].isLive` 判斷「進行中」，否則用 `updatedAt` 時間差推算「等待回應」與「閒置」
2. **`activityStatusMap`**：來自 `get_session_activity_statuses` IPC，每 30 秒輪詢，有 `status: "active"/"waiting"/"idle"/"done"` 欄位，但**未被用來計算計數**

後端已有完整的 Claude hook 事件流：hook scripts（.ps1）寫入 bridge JSONL → watcher 讀取 → 發射 Tauri event（`claude-activity-hint`）→ 前端目前只用來更新 `activityStatusMap`，但 payload 資訊不足（僅有 `eventType`，沒有派生的 `status`）。

## Goals / Non-Goals

**Goals:**
- 以 `activityStatusMap` 為唯一計數來源，消除雙重計算路徑
- Claude hook 事件直接攜帶 `SessionActivityStatus`，前端收到即更新 map，無需額外 IPC
- 移除 `activityStatusQuery` 的定時輪詢；hook 未安裝時自動降級為輪詢 fallback
- 新增 `.sh` hook scripts（Git Bash + jq），`commandWindows` 繼續用 .ps1，`command` 用 .sh
- 安裝前偵測 jq，不可用時顯示提示（不阻擋安裝）

**Non-Goals:**
- 不調整 Copilot / OpenCode provider 的輪詢邏輯
- 不更動 session_stats 計算或 token 用量顯示
- 不新增 WebSocket 或 SSE，僅使用現有 Tauri event 機制
- 不支援 Linux / macOS（sh 腳本雖跨平台，但 SessionHub 目前只發布 Windows）

## Decisions

### 1. 單一來源：`activityStatusMap` 取代雙路徑計算

**決策**：移除 `activeSessions` useMemo 中的 `sessionStatsMap.isLive` 與時間判斷，改為直接 `activityStatusMap.get(s.id)?.status` 映射到計數。

**理由**：`activityStatusMap` 已有完整四態（active/waiting/idle/done），是後端明確計算的結果，比前端用時間差推算更準確。兩套路徑並存只會造成不一致。

**替代方案考量**：保留 `sessionStatsMap.isLive` 作為主要判斷，僅用 `activityStatusMap` 補充 → 拒絕，仍然是兩套來源，問題未解決。

### 2. Hook 事件 payload 擴充：攜帶完整 `SessionActivityStatus`

**決策**：後端 `claude-activity-hint` event 的 payload 從 `{ cwd, eventType, title }` 擴充為 `{ cwd, eventType, title, sessionId, status, detail, lastActivityAt }`，讓前端直接 patch `activityStatusMap`。

**事件 → 狀態對應**（後端計算）：
| Hook event | status | detail |
|---|---|---|
| `SessionStart` | `active` | `thinking` |
| `UserPromptSubmit` | `active` | `thinking` |
| `PreToolUse` | `active` | `tool_call` |
| `PostToolUse` | `active` | `working` |
| `Stop`（stopReason=normal） | `idle` | — |
| `Stop`（stopReason=error/interrupt） | `waiting` | — |

**理由**：避免前端收到事件後還需發 IPC 查詢狀態，減少往返次數。

### 3. 輪詢 fallback 策略

**決策**：`activityStatusQuery` 改為：
- 若 `providerIntegrations` 中 Claude 的狀態為 `installed` → `refetchInterval: false`（純事件驅動）
- 否則 → `refetchInterval: 30_000`（維持原有輪詢）

**理由**：hook 未安裝的用戶仍能看到狀態（雖然有延遲），不造成功能退化。

### 4. Shell scripts 雙軌並存（sh + ps1）

**決策**：在 `.claude/hooks/` 下新增 `.sh` 版本的入口腳本和模組（`modules/*.sh`），`claude.rs` 的 `managed_hook_group` 同時填入：
- `command`：`sh "<path>/on-xxx.sh" --bridge-path "<path>" --provider claude`
- `commandWindows`：保留現有 `pwsh ... on-xxx.ps1 ...`

**理由**：Git Bash 的 sh 啟動速度遠快於 pwsh（< 50ms vs 300-800ms），hook 延遲直接影響 UI 更新速度。雙軌可確保不破壞現有 Windows 用戶，同時讓有 Git Bash 的環境享受更快的響應。

**sh 腳本依賴**：`jq`（JSON 解析）。sh 腳本使用 `command -v jq` 偵測，不存在時 fallback 印出錯誤到 stderr 並 exit 0（不阻斷 Claude）。

### 5. jq 偵測：後端 command + 前端提示

**決策**：新增 `check_jq_available` Tauri command（呼叫 `which jq` 或 `jq --version`），`SettingsView` 在顯示 Claude integration 安裝卡片時呼叫，結果以 banner 顯示（不阻擋安裝按鈕）。

**訊息設計**：
- 有 jq：不顯示任何提示（靜默）
- 無 jq：顯示 info banner「建議安裝 jq 以啟用 Git Bash hook 模式（更快的即時更新）。可透過 Git for Windows 安裝程式勾選，或執行 winget install jqlang.jq」

## Risks / Trade-offs

- **`activityStatusMap` 初始為空**：app 啟動後第一次 hook 事件到來前，`activityStatusMap` 沒有任何 entry，計數全部為 0。
  → 緩解：`activityStatusQuery` 仍在 mount 時執行一次（`refetchOnMount: true`），確保有初始值。

- **Stop 事件的 stopReason 判斷**：Claude hook Stop payload 的 `stop_reason` 欄位格式需驗證（normal/error/interrupt）。
  → 緩解：後端 watcher 解析時保守處理，未知 reason 一律歸為 `idle`。

- **sh 腳本 jq 不存在時靜默失敗**：hook 事件不會寫入 bridge，前端收不到更新。
  → 緩解：sh 腳本在 stderr 寫入錯誤；安裝時 UI 已提示用戶安裝 jq。

- **ps1 版本仍為 commandWindows fallback**：若用戶環境的 pwsh 啟動慢，每個 hook 事件都有 300-800ms 延遲。
  → 接受：這是現狀，sh 版本的引入正是改善途徑。

## Migration Plan

1. 後端先擴充 `claude-activity-hint` payload（向後相容，前端舊版忽略新欄位）
2. 前端更新 `activityStatusMap` patch 邏輯
3. 前端更新計數 useMemo（切換單一來源）
4. 新增 sh hook scripts 並更新 `claude.rs`（重新安裝 integration 才生效）
5. 新增 `check_jq_available` command 與 UI 提示
6. 舊的 `.ps1` 繼續保留為 `commandWindows`，不刪除

## Open Questions

- `PostToolUse` 後的狀態是否需要等 Claude 回應才改為 `waiting`？目前設計為 `active/working`，等 `Stop` 才切換。這樣比較保守，可以觀察實際使用後再調整。
