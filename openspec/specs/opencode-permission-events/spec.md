## Purpose

定義 opencode bridge plugin 對權限請求與回覆事件的監聽、bridge record 正規化、activity hint 轉譯與前端消費行為。

## Requirements

### Requirement: Plugin 監聽權限請求事件

SessionHub 產生的 opencode bridge plugin SHALL 在 `event` handler 中監聽舊版、v1 與 v2 權限請求事件；當 `event.type` 為 `permission.updated`、`permission.asked` 或 `permission.v2.asked` 時，寫入一筆正規化 bridge record，使 Bash、檔案讀寫、跨專案／跨目錄存取與其他受管制操作皆能被 SessionHub 後端消費。

#### Scenario: 權限請求產生時寫入 bridge record

- **WHEN** opencode 發出 `event.type === "permission.updated"` 事件（例如使用者遇到「Permission required - Access external directory」）
- **THEN** plugin 透過 `appendRecord` 寫入一筆 record
- **AND** record 的 `provider` 為 `"opencode"`、`eventType` 為 `"permission.updated"`
- **AND** record 的 `sessionId` 取自 `event.properties.sessionID`
- **AND** record 的 `title` 取自 `event.properties.title`（權限請求的顯示文字）

#### Scenario: 跨版本相容權限事件名稱

- **WHEN** opencode SDK 升級後改以 `event.type === "permission.asked"` 發出權限請求事件
- **THEN** plugin 同樣寫入 bridge record，不因事件名稱改變而漏接
- **AND** plugin 對 `permission.updated` 與 `permission.asked` 兩種事件名稱皆判斷處理

#### Scenario: 現行 v2 權限請求寫入 bridge record

- **WHEN** opencode 發出 `event.type === "permission.v2.asked"`
- **THEN** plugin 寫入一筆 bridge record，`eventType` 保留為 `permission.v2.asked`
- **AND** `sessionId` 取自 `event.properties.sessionID`
- **AND** `title` 包含可識別的 `action` 與不透明 request id，但不包含完整 resources 清單
- **AND** request `id` 參與 record 唯一性，避免同 session 的連續請求被去重

#### Scenario: 不限定權限操作種類

- **WHEN** 權限請求的操作為 Bash 指令、Read、Edit、Write、external directory 或其他 opencode 權限 action
- **THEN** plugin 依事件家族統一處理，不以 action 白名單過濾
- **AND** 每個真正的 asked/updated 事件都會產生 bridge record

#### Scenario: 權限 payload 不洩漏敏感內容

- **WHEN** 權限 payload 包含 Bash 指令、完整檔案路徑、resources 或其他工具輸入
- **THEN** bridge record 與通知只保留權限 action/type、session id 與不透明 request id 等必要欄位
- **AND** 不寫入 Bash 完整指令、檔案內容或完整 resources 清單

### Requirement: Plugin 監聽權限回覆事件

SessionHub 產生的 opencode bridge plugin SHALL 監聽 `permission.replied` 與 `permission.v2.replied` 事件並寫入 bridge record，作為授權完成、清除待介入狀態的輔助訊號。

#### Scenario: 權限回覆時寫入 bridge record

- **WHEN** opencode 發出 `event.type === "permission.replied"` 事件（使用者已允許或拒絕該權限請求）
- **THEN** plugin 寫入一筆 record，`eventType` 為 `"permission.replied"`、`sessionId` 取自 `event.properties.sessionID`

#### Scenario: v2 權限回覆時寫入 bridge record

- **WHEN** opencode 發出 `event.type === "permission.v2.replied"`
- **THEN** plugin 寫入一筆 record，`eventType` 為 `"permission.v2.replied"`、`sessionId` 取自 `event.properties.sessionID`
- **AND** request id 可用於區分不同權限請求

### Requirement: Bridge 將權限請求轉譯為 waiting activity hint

Rust bridge（`process_provider_bridge_event`）SHALL 針對 opencode 舊版、v1 與 v2 權限請求事件使用相同 activity 分支，將其轉譯為 `opencode-activity-hint` 並將 status 設為 `waiting`，使前端能將該 opencode session 判定為需介入。

#### Scenario: 權限請求 record 轉為 waiting hint

- **WHEN** 後端讀到 opencode bridge record 且 `eventType` 為 `permission.updated`、`permission.asked` 或 `permission.v2.asked`
- **THEN** 系統 emit `opencode-activity-hint` 事件
- **AND** payload 的 `status` 為 `"waiting"`、`sessionId` 取自 record 的 `sessionId`
- **AND** payload 帶入 `title`（權限請求文字）供顯示
- **AND** payload 的 `cwd` 允許為空字串（前端在有 `sessionId` 時優先以 sessionId 反查 session）

#### Scenario: 權限回覆 record 清除 waiting

- **WHEN** 後端讀到 opencode bridge record 且 `eventType` 為 `permission.replied` 或 `permission.v2.replied`
- **THEN** 系統 emit `opencode-activity-hint` 事件，`status` 設為離開 `waiting` 的狀態（`active`）
- **AND** 同一 session 後續再次出現權限請求時，能重新觸發 `waiting`

### Requirement: 權限請求事件不因指紋重複被去重吞掉

Rust bridge 的 record 去重機制（`register_provider_bridge_record`）SHALL 確保同一 session 連續多次的權限請求事件不會因指紋相同而被誤判為重複而丟棄。

#### Scenario: 連續兩次不同權限請求皆被處理

- **WHEN** 同一 opencode session 先後產生兩次權限請求事件（不同 permission）
- **THEN** 兩次事件各自產生的 bridge record 指紋不同
- **AND** 第二次事件不會被 `register_provider_bridge_record` 當作重複而略過

### Requirement: 前端消費 opencode activity hint

前端 `App.tsx` SHALL 新增 `opencode-activity-hint` 事件監聽器，將 hint 的 status patch 進 `activityStatusMap`，使 opencode session 能進入既有的 `waiting` 偵測與介入通知流程。

#### Scenario: opencode session 進入 waiting

- **WHEN** 前端收到 `opencode-activity-hint` 事件且 payload `status` 為 `"waiting"`
- **AND** payload 的 `sessionId` 對應到目前 `sessionsQuery` 中的某 opencode session
- **THEN** 該 session 在 `activityStatusMap` 中的狀態被更新為 `waiting`
- **AND** status bar 的 waiting 計數與看板顯示隨之更新

#### Scenario: session 尚未被掃描到時靜默略過

- **WHEN** 前端收到 `opencode-activity-hint` 事件，但 payload 的 `sessionId` 在目前 `sessionsQuery` 中查無對應 session
- **THEN** 系統不更新 `activityStatusMap`、不發送通知、不報錯（沿用既有 activity hint handler 的降級行為）
- **AND** 待該 session 被掃描到後的後續事件可正常觸發 waiting
