## ADDED Requirements

### Requirement: 後端維護 waiting 清單為 single source of truth

系統 SHALL 在後端維護一份目前處於 `waiting`（等待授權）狀態的 session 清單（`InterventionRegistry`），作為介入提醒的唯一資料來源；清單更新點位於後端 activity 狀態計算之後，不依賴任何前端視窗的 state。

#### Scenario: 進入 waiting 時加入清單

- **WHEN** 後端 activity 狀態計算判定某 session 由非 `waiting` 轉為 `waiting`
- **THEN** 系統將該 session upsert 進 `InterventionRegistry`，記錄 `sessionId`、`projectName`、`toolLabel`
- **AND** `projectName` 取自 session `cwd` 最後一段路徑，取不到時以 `sessionId` 尾段或 provider 名替代
- **AND** `toolLabel` 僅為工具類型（如 `Bash` / `Read` / `Edit` / `Write`），不含指令、檔案內容或完整路徑

#### Scenario: 離開 waiting 時移出清單

- **WHEN** 某 session 的 activity 狀態由 `waiting` 轉為其他狀態（active/done/idle 等）
- **THEN** 系統將該 session 自 `InterventionRegistry` 移除

#### Scenario: 清單獨立於主視窗運行狀態

- **WHEN** 主視窗關閉或最小化
- **THEN** `InterventionRegistry` 仍隨後端 activity 事件正常更新，不因主視窗未顯示而停止

### Requirement: 廣播 intervention-list-changed 事件

系統 SHALL 在 `InterventionRegistry` 內容變動時，emit app 級 `intervention-list-changed` 事件，使 quota overlay 視窗與主視窗皆可訂閱；payload 為當前清單快照且僅含最小化欄位。

#### Scenario: 清單變動時廣播

- **WHEN** `InterventionRegistry` 因加入或移除 session 而變動
- **THEN** 系統 emit `intervention-list-changed`，payload 為當前所有 waiting 項目的陣列
- **AND** 每個項目僅含 `sessionId`、`projectName`、`toolLabel`，不含指令、檔案內容、resources 或完整路徑

#### Scenario: 視窗初次訂閱取得當前快照

- **WHEN** quota overlay 視窗或主視窗建立並開始訂閱後，可能已錯過先前的 emit
- **THEN** 系統提供機制使該視窗能取得當前清單快照（建立時補發一次事件，或提供可查詢當前清單的 command）
- **AND** 訂閱者不會因錯過歷史事件而永久顯示空清單
