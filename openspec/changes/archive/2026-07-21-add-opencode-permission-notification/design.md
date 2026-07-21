# Design: Claude 與 opencode 權限請求介入通知

## Context

Claude 或 opencode session 卡在工具權限授權（例如 Bash 指令、檔案讀寫或「Access external directory」）時，SessionHub 可能不會發出介入通知。現況鏈路缺口：

- **Plugin**（`src-tauri/src/provider/opencode.rs` 的 `render_opencode_integration`）：產生的 opencode bridge plugin 只監聽 `session.updated` / `session.created` / `session.error`，未監聽權限事件。
- **Rust bridge**（`src-tauri/src/provider/bridge.rs` 的 `process_provider_bridge_event`）：對 opencode 只落到最後的 `emit_provider_refresh`（全量刷新），沒有 activity 分支；`derive_activity_status` 也只識別 copilot/claude 的事件類型。
- **前端**（`src/App.tsx`）：只監聽 `copilot-activity-hint` 與 `claude-activity-hint`。介入通知的實際觸發點在 `App.tsx:1708`（某 session 狀態轉 `waiting` 且 `enableWaiting` → `invoke("send_intervention_notification")`）；狀態來源是 `activityStatusMap`。opencode 沒有對應 hint 監聽器，永遠進不了此判定。
- **Claude hook**（`.claude/hooks/on-notification.cjs`）：`Notification` payload 的通知類型欄位是 `notification_type`，現有腳本卻讀取 `matcher` / `event`，所以 `permission_prompt` 與 `idle_prompt` 分支不會命中。整合也未納管專門在工具要求授權時觸發的 `PermissionRequest` hook。

已查證的 opencode 權限事件 API（綁定 `.opencode` 的 `@opencode-ai/plugin` / `@opencode-ai/sdk` 1.14.25）：

- 權限請求：`event.type === "permission.updated"`；`event.properties` 為 `Permission` 物件 `{ id, type, pattern?, sessionID, messageID, callID?, title, metadata, time: { created } }`。`title` 即 UI 顯示文字。
- 權限回覆：`event.type === "permission.replied"`；`event.properties` 為 `{ sessionID, permissionID, response }`。
- v1 事件名為 `permission.asked` / `permission.replied`；現行 v2 事件名為 `permission.v2.asked` / `permission.v2.replied`，payload 使用 `action`、`resources`、`requestID` 等欄位。plugin 需相容三個事件家族。
- `permission.ask` hook 在 1.14.25 從未被實際呼叫（GitHub issue #7006），不採用。

已查證的 Claude Code hook 契約：

- `PermissionRequest` 在工具真正需要使用者授權時觸發，payload 包含 `session_id`、`cwd`、`tool_name`、`tool_input` 與 `permission_suggestions`，可涵蓋 Bash、Read、Edit、Write 與跨目錄存取。
- `Notification` 的通知類型位於 `notification_type`；`permission_prompt` 代表授權提示，`idle_prompt` 代表等待回應。
- `PreToolUse` 代表即將使用工具，不等於一定需要授權，因此不可用來判定授權提示。

## Goals / Non-Goals

**Goals**
- Claude 與 opencode 的所有工具授權請求均能觸發介入通知，不限於單一工具或單一路徑。
- opencode 權限請求發生時，該 session 進入 `waiting`，並沿用既有介入通知機制發送 Windows Toast。
- 權限回覆後清除 `waiting`，使後續權限請求能再次觸發通知。
- 跨 SDK 版本相容 opencode `permission.updated`、`permission.asked` 與 `permission.v2.asked` 事件家族。
- Claude 以 `PermissionRequest` 為主要授權訊號，並保留 `Notification.permission_prompt` 相容備援。

**Non-Goals**
- 不攔截／自動回覆權限請求（不使用 `permission.ask` hook）。
- 不新增 opencode 的 hook 腳本路徑（opencode 走 plugin bridge，非 `hooks/` 腳本）。
- 不將 Claude `PreToolUse` 一律視為需授權，避免每次工具執行都誤報。
- 不移除 Claude 的離線原生通知能力；SessionHub 未執行時仍須通知。
- 不改動 Copilot／Codex 既有 bridge、activity、通知行為。
- 不新增設定項；沿用既有 `enable_intervention_notification`。

## Decisions

### 決策 1：sessionID 對齊 — 直接沿用 opencode 內部 session id

`Permission.sessionID`、opencode 內部 session table 的 `s.id`（`sessions/opencode.rs` 的 `SessionInfo.id` 直接取自 SQL `s.id`）、以及前端 `sessionsQuery` 的 session id **是同一個 id**（opencode SDK 內部一致）。因此 plugin 只需把 `event.properties.sessionID` 原樣寫入 bridge record 的 `sessionId`，即可讓 waiting 狀態掛在前端認得的 session 上、並讓通知反查到 `cwd` → 專案名。

- **替代方案**：由 cwd 反查 session id。否決 —— `Permission` payload 沒有 directory/cwd 欄位，只有 `sessionID`；且 cwd 反查已是 copilot 路徑的既有複雜度，此處無此需要。
- **cwd 欄位處理**：`ActivityHintPayload.cwd` 為必填 `String`，但 `Permission` payload 無 cwd。因前端 activity hint handler 在有 `sessionId` 時**優先以 sessionId 反查 session**（`App.tsx:1308-1310`，cwd 僅為 fallback），bridge 分支 emit 時 `cwd` 傳空字串即可，不影響狀態更新與通知反查專案名（通知端 `App.tsx:1705` 亦以 session id 反查 cwd）。
- **session 尚未掃描到的降級**：權限請求可能發生在 SessionHub 尚未掃到該 session 時。既有 activity hint handler 在 `sessionsDataRef` 找不到對應 session 時直接 `return`（`App.tsx:1311`，靜默略過，不報錯）。opencode 沿用同一 handler 模式，行為一致：此情況下該次不更新 waiting、不發通知，待下次事件或掃描補上。

### 決策 2：事件轉譯放在 Rust bridge 的 opencode 分支，比照 claude 模式

在 `process_provider_bridge_event` 新增 `provider == OPENCODE_PROVIDER` 分支：讀到 `permission.updated` / `permission.asked` → emit `opencode-activity-hint`（`ActivityHintPayload`，`status = "waiting"`，帶 `sessionId`、`title`）；讀到 `permission.replied` → emit `opencode-activity-hint`（`status = "active"`）。`derive_activity_status` 增加對權限事件類型的對映，維持現有推導函式的單一職責。

- **替代方案**：在前端直接把 opencode 全量刷新事件解讀成 waiting。否決 —— 前端拿不到權限事件語意，且違反「activity 由後端計算」的既有慣例（claude 分支即如此）。

### 決策 3：`permission.replied` 重置目標狀態為 `active`（離開 waiting 即可）

opencode 沒有其他 activity hint 來源會把狀態推回 idle。重置只需確保**離開 `waiting`**，讓下一次權限請求能重新觸發轉換（`App.tsx:1708` 判斷 `status === "waiting" && prev !== "waiting"`）。選 `active` 表示 session 仍在進行中；後續正常的 session 刷新會自然更新其顯示。

### 決策 4：Claude 使用 `PermissionRequest` 主路徑，`Notification` 作為相容備援

新增 `on-permission-request.cjs`，由 `PermissionRequest` hook 在 Bash、Read、Edit、Write 或其他工具真正要求授權時直接發送 hook 原生通知。既有 `Notification` hook 改讀 `notification_type`，保留 `permission_prompt` 備援與 `idle_prompt` 等待回應通知。

- `PreToolUse` 只維持 `active/tool_call`，不得作為授權判定，因為獲准工具同樣會觸發。
- 兩條 Claude 授權通知路徑使用相同 `sessionhub-{session_id}` 識別，若同一請求同時觸發 `PermissionRequest` 與 `Notification`，後者取代前者而非堆疊。
- Claude 沿用 hook 直接通知，不額外把同一授權事件送進應用內 `waiting → send_intervention_notification` 路徑，避免 SessionHub 開啟時同時出現 hook 與 Tauri 兩則通知。

### 決策 5：opencode 事件先正規化再進入既有 activity 路徑

plugin 將舊版、v1 與 v2 payload 正規化為相同 bridge record：`sessionId` 取 `sessionID`；`title` 使用可讀標題或權限 action/type，並納入不透明的 permission/request id 以確保唯一性；`timestamp` 優先取事件時間，無時間欄位時使用當下 ISO 時間。不得將 Bash 完整指令、檔案內容或完整 resources 清單寫入 bridge record 或通知。Rust bridge 將所有 asked/updated 事件對映為 `waiting`，所有 replied 事件對映為 `active`。

## Risks / Trade-offs

- **[dedup 指紋碰撞]** `provider_bridge_record_fingerprint` 由 `version|provider|eventType|timestamp|sessionId|cwd|sourcePath|title|error` 組成。同一 session 連續兩次權限請求若 title 相同、timestamp 精度不足，指紋可能相同而被 `register_provider_bridge_record` 吞掉。**Mitigation**：plugin 寫入 record 的 `timestamp` 使用 `Permission.time.created`（每次請求不同的 epoch ms），必要時將 `permission.id` 併入 `title` 或另一欄位確保指紋唯一。
- **[整合檔需重裝]** plugin 內容變更後，`detect_opencode_integration_status` 會判為 outdated，需使用者於設定頁重新安裝／更新 opencode 整合。**Mitigation**：於 tasks 明列並在通知使用者「更新整合」；沿用既有 install/update 流程，不需額外 UI。
- **[跨版本 payload 結構不同]** dev 版 `permission.asked` 的欄位名與 1.14.25 不同（`permission`/`patterns` vs `type`/`title`）。**Mitigation**：plugin 讀欄位時對兩種形狀做防禦性存取（`title ?? permission`），目前以 1.14.25 為主。
- **[Claude 雙事件重複]** 同一授權可能先後觸發 `PermissionRequest` 與 `Notification.permission_prompt`。**Mitigation**：兩者使用相同 session 通知識別，由 Windows 通知取代而非堆疊；不再額外觸發應用內 Tauri 通知。
- **[Claude 版本相容]** 舊版 Claude Code 可能沒有 `PermissionRequest` hook。**Mitigation**：保留修正後的 `Notification.permission_prompt` 備援，整合安裝不能因未知事件而破壞其他 hooks。
- **[opencode v2 無事件時間]** v2 asked payload 不保證提供 created time。**Mitigation**：bridge record 使用寫入當下 ISO 時間，並將 request id 納入可參與指紋的欄位，確保連續請求不被去重。
- **[敏感工具輸入外洩]** Bash 指令與跨目錄 resources 可能含本機路徑或機密參數。**Mitigation**：hook/plugin 僅保留工具／action 類型與不透明 request id，不將原始 `tool_input`、命令、檔案內容或完整 resources 寫入 bridge、log 或 Toast。

## Migration Plan

1. 擴充 opencode plugin 與 Rust bridge 的 v2 權限事件對映。
2. 新增 Claude `PermissionRequest` hook 腳本，修正 `Notification.notification_type` 解析，並更新整合納管事件與版本。
3. 使用者於設定頁更新 Claude 與 opencode 整合，以套用新 hook/plugin。
4. **Rollback**：還原新增事件與腳本，重新安裝整合即可回到舊版本；不涉及資料遷移。

## Verification Strategy

- Claude：分別觸發 Bash、專案內檔案工具、跨專案／跨目錄 Read，確認真正顯示授權框時有且僅有一則介入通知；不需授權的工具執行不得誤報。
- Claude 相容備援：以 `notification_type=permission_prompt` 與 `idle_prompt` fixture 驗證正確分支，並確認舊 `matcher` 缺失不影響判斷。
- opencode：以舊版、v1、v2 asked/replied fixture 驗證 session id、標題、唯一性與 waiting/active 轉換。
- 整合升級：舊 Claude/opencode 整合顯示 outdated，更新後包含新事件且保留使用者自訂 hooks/plugin 設定。
