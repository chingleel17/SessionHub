## Why

使用者在 Claude 或 opencode 遇到 Bash 指令、檔案存取、跨專案／跨目錄讀取等授權確認時，SessionHub 的「AI 介入通知」可能完全沒有觸發，導致背景 session 卡在等待授權卻無人知曉。

opencode 原先缺少 plugin、Rust bridge 與前端 activity hint 的完整鏈路；既有實作雖已補上舊版 `permission.updated` / `permission.asked`，但尚未涵蓋現行 `permission.v2.asked` / `permission.v2.replied`。Claude 則已有 `Notification` hook，卻讀取錯誤的 payload 欄位 `matcher` / `event`；現行契約實際使用 `notification_type`，且更精準的 `PermissionRequest` hook 尚未安裝，因此各類工具授權提示可能漏接。

## What Changes

- opencode bridge plugin 新增監聽權限請求事件（`permission.updated`，並相容上游 `permission.asked`），在使用者需要授權時寫入一筆 bridge record；同時監聽 `permission.replied` 作為授權完成、清除待介入狀態的輔助事件。
- opencode plugin 同時支援現行 `permission.v2.asked` / `permission.v2.replied`，將不同版本 payload 正規化為一致的 session id、權限類型、顯示文字與唯一請求識別。
- Rust bridge（`process_provider_bridge_event`）新增 opencode 的 activity 分支：將權限請求 record 轉譯為 `opencode-activity-hint` 並將 status 設為 `waiting`；`permission.replied` 則清除 `waiting`（回到 `active`）。
- 前端 `App.tsx` 新增 `opencode-activity-hint` 監聽器，patch `activityStatusMap`，使 opencode session 能進入既有的 `waiting → send_intervention_notification` 通知流程。
- opencode 權限請求觸發的通知沿用既有介入通知語意與文案（歸類為 `waiting`／需授權，與 Copilot、Claude 一致）。
- Claude integration 新增 `PermissionRequest` hook，涵蓋 Bash、Read、Edit、Write 及跨專案／跨目錄存取等所有實際要求授權的工具情境，並直接發送既有 hook 原生介入通知。
- Claude `Notification` hook 改讀 `notification_type`；`permission_prompt` 保留為相容備援、`idle_prompt` 維持等待回應通知。兩條 Claude 路徑使用相同 session tag，重複事件只會取代通知，不會堆疊。

## Capabilities

### New Capabilities

- `opencode-permission-events`: opencode bridge plugin 對權限請求／回覆事件的監聽與 bridge record 寫入，以及 Rust bridge 將其轉譯為 `opencode-activity-hint`（權限請求→`waiting`、權限回覆→清除）供前端消費的行為契約。
- `claude-permission-events`: Claude `PermissionRequest` 主路徑與 `Notification.permission_prompt` 相容備援的事件擷取、通知與去重契約。

### Modified Capabilities

- `intervention-notification`: 新增 opencode provider 經由權限請求事件觸發 `waiting` 介入通知的路徑（現行 requirement 僅涵蓋 Copilot／Codex／Claude 的 hook 觸發點，未涵蓋 opencode plugin 的權限事件觸發）。
- `hook-native-notification`: 修正 Claude 通知類型欄位，並以 `PermissionRequest` 作為工具授權通知的主要觸發點。
- `claude-hook-integration`: 安裝、更新、偵測與移除流程需納管 `PermissionRequest` hook。

## Impact

- Rust：`src-tauri/src/provider/opencode.rs`（plugin 權限事件分支與 v2 payload 正規化）、`src-tauri/src/provider/bridge.rs`（opencode activity 分支與 v2 事件對映）、`src-tauri/src/provider/claude.rs`（納管 `PermissionRequest` hook 與整合版本更新）。
- 前端：`src/App.tsx`（新增 `opencode-activity-hint` 監聽器）。
- Hook：`.claude/hooks/on-permission-request.cjs`（新增）與 `.claude/hooks/on-notification.cjs`（修正 `notification_type` 解析）。
- 整合檔重裝：使用者需在設定頁更新 Claude 與 opencode 整合，才能取得新 hook/plugin；偵測狀態必須將舊版本標為 outdated。
- 相依：opencode 同時相容舊版 `permission.updated`、v1 `permission.asked` 與現行 v2 `permission.v2.asked` 事件家族。
- 不改變 Copilot／Codex 的既有 bridge、activity 與通知行為。
