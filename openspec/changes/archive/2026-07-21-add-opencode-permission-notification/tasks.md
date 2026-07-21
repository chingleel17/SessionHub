## 1. Plugin：opencode 權限事件監聽

- [x] 1.1 於 `src-tauri/src/provider/opencode.rs` 的 `render_opencode_integration` 產生的 plugin `event` handler 中，新增分支判斷 `event.type === "permission.updated" || event.type === "permission.asked"`
- [x] 1.2 權限請求分支寫入 bridge record：`eventType` 為對應事件名、`sessionId` 取自 `event.properties.sessionID`、`title` 取自 `event.properties.title`（`?? event.properties.permission` 防禦性存取）、`timestamp` 使用 `event.properties.time.created`（epoch ms → ISO）以確保 dedup 指紋唯一
- [x] 1.3 新增 `permission.replied` 分支，寫入 record：`eventType` 為 `"permission.replied"`、`sessionId` 取自 `event.properties.sessionID`
- [x] 1.4 確認 `PROVIDER_INTEGRATION_VERSION` 或 metadata 會因 plugin 內容變更而觸發 outdated 判定（`detect_opencode_integration_status`），必要時 bump 版本

## 2. Rust bridge：權限事件轉譯為 activity hint

- [x] 2.1 於 `src-tauri/src/provider/bridge.rs` 的 `derive_activity_status` 新增對 `permission.updated` / `permission.asked`（→ `waiting`）與 `permission.replied`（→ `active`）的對映
- [x] 2.2 於 `process_provider_bridge_event` 新增 `provider == OPENCODE_PROVIDER` 分支：讀到權限事件 record 時 emit `opencode-activity-hint`（`ActivityHintPayload`：`status`、`sessionId`、`title`、`last_activity_at`；`cwd` 傳空字串，前端以 sessionId 反查）
- [x] 2.3 確認 `provider_refresh_event_name` 對 opencode 的既有 `opencode-sessions-updated` 全量刷新路徑不受影響（非權限事件仍走原路徑）
- [x] 2.4 驗證同一 session 連續兩次權限請求的 record 指紋不同（timestamp 來自不同 `time.created`），不會被 `register_provider_bridge_record` 去重吞掉

## 3. 前端：消費 opencode activity hint

- [x] 3.1 於 `src/App.tsx` 新增 `opencode-activity-hint` 事件監聽器（比照 `claude-activity-hint`），將 payload patch 進 `activityStatusMap`
- [x] 3.2 確認註冊的 unlisten 於 effect cleanup 中被呼叫，避免重複註冊
- [x] 3.3 驗證 opencode session 進入 `waiting` 後能觸發 `App.tsx` 既有的 `send_intervention_notification`（`enableWaiting` 為 true 時）

## 4. 驗證與整合

- [x] 4.1 `cargo build` 通過，無新增 warning
- [x] 4.2 前端 `bun run build`（或專案既定前端建置指令）通過
- [x] 4.3 於設定頁重新安裝／更新 opencode 整合，確認 plugin 檔已更新為含權限事件監聽的版本（注意：bridge watcher 僅在 opencode 整合已安裝時啟動，見 `watcher.rs` 的 `opencode_bridge_active`，未安裝則不會處理 bridge 事件）
- [x] 4.4 實機驗證：opencode 觸發權限請求 → SessionHub 發出「需要您授權/介入」Toast；點擊通知聚焦至對應 project tab；授權後 waiting 清除
- [x] 4.5 實機驗證：確認 `Permission.title` 為可讀文字；若為空則以 `type` 補位（回填至 plugin 實作）

## 5. Claude：完整授權事件通知

- [x] 5.1 新增 `.claude/hooks/on-permission-request.cjs`，讀取 `session_id` 與 `tool_name`，寫入診斷用 bridge record 並透過既有 `notify.cjs` 發送「需授權」介入通知；不得持久化原始 `tool_input`、Bash 指令或檔案內容
- [x] 5.2 修正 `.claude/hooks/on-notification.cjs`，改由 payload 的 `notification_type` 判斷 `permission_prompt` 與 `idle_prompt`，不得依賴 `matcher` / `event`
- [x] 5.3 確認 `PermissionRequest` 與 `Notification.permission_prompt` 使用相同 `sessionhub-{session_id}` 通知識別，同一授權事件不會堆疊兩則通知
- [x] 5.4 保持 `PreToolUse` 僅產生 `tool.pre` activity，不將所有工具執行誤判為需授權

## 6. Claude integration 生命週期

- [x] 6.1 於 `src-tauri/src/provider/claude.rs` 將 `PermissionRequest` 與 `on-permission-request.cjs` 納入腳本安裝、managed events、設定 merge 與移除流程
- [x] 6.2 更新 Claude hook integration 版本／完整性判定，使缺少 `PermissionRequest` 或使用舊版通知腳本的安裝狀態顯示為 outdated
- [x] 6.3 新增 Rust 測試，驗證安裝後包含 `PermissionRequest`、更新不重複加入 managed group、移除時保留使用者自訂 hooks
- [x] 6.4 新增 Node fixture 測試，驗證 `PermissionRequest` 的 Bash、Read／跨目錄 payload，以及 `Notification.notification_type` 的 permission/idle/其他值分支

## 7. opencode：現行 v2 權限事件相容

- [x] 7.1 擴充產生的 opencode plugin，監聽 `permission.v2.asked` / `permission.v2.replied`，並將 v2 的 `action`、`id` / `requestID` 正規化至 bridge record；不得持久化 Bash 指令、檔案內容或完整 resources 清單
- [x] 7.2 擴充 `derive_activity_status` 與 opencode bridge activity 分支，使 `permission.v2.asked` 對映 `waiting`、`permission.v2.replied` 對映 `active`
- [x] 7.3 確保 v2 request id 參與 record 指紋唯一性；同一 session 連續相同 action/resources 的請求不得被錯誤去重
- [x] 7.4 新增舊版 `permission.updated`、v1 `permission.asked/replied`、v2 `permission.v2.asked/replied` 的 plugin 產出與 Rust bridge fixture 測試

## 8. 端對端驗證

- [x] 8.1 更新 Claude 與 opencode 整合後，確認設定頁均顯示 installed，且使用者原有 hook/plugin 設定未被覆蓋
- [x] 8.2 Claude 實機驗證：Bash 指令、Read、Edit／Write、跨專案／跨目錄讀取在真正要求授權時各觸發一則通知；不需授權的工具執行不通知
- [x] 8.3 Claude 實機驗證：`idle_prompt` 仍觸發等待回應通知，SessionHub 未執行時授權通知仍可送出
- [x] 8.4 opencode 實機驗證：Bash、檔案讀寫與 external directory 等不同 permission action 均進入 waiting 並觸發介入通知，回覆後回到 active
- [x] 8.5 執行 `cargo test`、`bun run lint` 與 `bun run build`，確認無新增錯誤或 warning
