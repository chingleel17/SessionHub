## Why

SessionHub 每次偵測到 Copilot hook 事件（session 開始／結束、tool 呼叫等）時，都會觸發**整份 session 列表的重新掃描與 UI 重繪**，即使只有一個 session 發生了變化。這導致 session 數量多時（例如超過 50 個），UI 明顯卡頓、前端全量重整。Hook 事件本身已包含 `cwd`（工作目錄），足以精確識別是哪個 session 產生了變化，應據此做到**定向更新**而非全量刷新。

## What Changes

- **新增 Rust 命令 `get_session_by_cwd`**：根據 cwd 路徑快速找出對應 session（只掃一個目錄），回傳單一 `SessionInfo`。
- **Bridge 事件夾帶 cwd**：`emit_provider_refresh` 改為 `copilot-session-targeted`（含 cwd）與 `copilot-sessions-updated`（全量 fallback）兩種事件模式。
- **前端定向更新**：收到 `copilot-session-targeted` 事件時，只刷新 React Query cache 中對應 session 的資料，不觸發全量 `invalidateQueries`。
- **全量 fallback 保留**：若 cwd 無法對應到任何 session（例如新 session 尚未建立），則退回原有的全量刷新流程。
- **session-state 檔案監聽在 hook 已安裝時維持停用**（現有行為不變，此次加以文件化並強化測試覆蓋）。

## Capabilities

### New Capabilities

- `targeted-session-refresh`：根據 hook 事件的 cwd 定向解析 session ID，並以最小代價更新單一 session 的 UI 狀態，取代全量重整。

### Modified Capabilities

（無 spec 層級的需求變更）

## Impact

- **`src-tauri/src/provider/bridge.rs`**：`emit_provider_refresh` 新增 cwd 參數，改為發送含 payload 的定向事件。
- **`src-tauri/src/watcher.rs`**：`process_provider_bridge_event` 傳遞 cwd 給新的 emit 函式。
- **`src-tauri/src/commands/sessions.rs`**：新增 `get_session_by_cwd` command，並在 `invoke_handler!` 登記。
- **`src-tauri/src/sessions/mod.rs`** 或 `copilot.rs`：新增 `find_session_by_cwd_internal` 輔助函式。
- **`src/App.tsx`**：新增 `copilot-session-targeted` 事件監聽，執行 React Query 的單一 session 更新邏輯。
- **`src/types/index.ts`**：新增 `SessionTargetedPayload` 型別定義。
- 不引入新外部相依套件。
