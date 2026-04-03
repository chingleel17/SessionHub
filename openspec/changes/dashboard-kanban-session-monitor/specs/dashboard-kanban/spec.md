## ADDED Requirements

### Requirement: Kanban 視圖與清單視圖切換

系統 SHALL 在 Dashboard 提供 Kanban 視圖，並與現有清單視圖並列提供切換選項，切換偏好儲存於本次 session（不持久化）。

#### Scenario: 切換至 Kanban 視圖

- **WHEN** 使用者在 Dashboard 點擊「Kanban」切換按鈕
- **THEN** 系統顯示四欄 Kanban 看板（Idle / Active / Waiting / Done）
- **AND** 每欄標題顯示該狀態的 session 數量

#### Scenario: 切換回清單視圖

- **WHEN** 使用者在 Kanban 視圖點擊「清單」切換按鈕
- **THEN** 系統恢復顯示現有的統計列與專案清單視圖

### Requirement: Kanban 看板欄位定義

系統 SHALL 將所有 non-archived sessions 依自動偵測的活動狀態分配至對應欄位。

#### Scenario: Active 欄顯示

- **WHEN** session 的活動狀態為 `active`
- **THEN** 系統將該 session 的 Kanban 卡片顯示於 Active 欄

#### Scenario: Waiting 欄顯示

- **WHEN** session 的活動狀態為 `waiting`
- **THEN** 系統將該 session 的 Kanban 卡片顯示於 Waiting 欄

#### Scenario: Idle 欄顯示

- **WHEN** session 的活動狀態為 `idle`
- **THEN** 系統將該 session 的 Kanban 卡片顯示於 Idle 欄

#### Scenario: Done 欄顯示

- **WHEN** session 的活動狀態為 `done`（已封存或超過 24h 無活動）
- **THEN** 系統將該 session 的 Kanban 卡片顯示於 Done 欄

### Requirement: Kanban 卡片資訊

系統 SHALL 在每張 Kanban 卡片上顯示 session 的關鍵資訊與操作快捷鍵。

#### Scenario: 卡片基本資訊

- **WHEN** Kanban 看板顯示 session 卡片
- **THEN** 每張卡片顯示：session summary（或 ID）、所屬專案名稱、provider 標籤、最後更新時間
- **AND** Active 狀態卡片額外顯示活動細節（Thinking / Tool Call / File Op / Sub-Agent / Working）

#### Scenario: 卡片啟動工具快捷鍵

- **WHEN** 使用者在 Kanban 卡片上點擊工具啟動按鈕
- **THEN** 以預設啟動工具開啟對應 session 的工作目錄或恢復 session

### Requirement: Kanban 視圖跨專案顯示

系統 SHALL 在 Kanban 看板中展示所有專案的 sessions，不依專案分組。

#### Scenario: 跨專案 session 顯示

- **WHEN** 使用者瀏覽 Kanban 視圖
- **THEN** 來自不同專案的 sessions 在同一欄中混合顯示
- **AND** 每張卡片的所屬專案名稱清晰可見
