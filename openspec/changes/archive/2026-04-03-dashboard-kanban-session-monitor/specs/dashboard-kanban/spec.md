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

系統 SHALL 在 Kanban 看板中展示所有專案的 sessions，以**專案為單位**分組顯示。

#### Scenario: 跨專案 session 顯示（ProjectCard 分組）

- **WHEN** 使用者瀏覽 Kanban 視圖
- **THEN** 同一欄中的 sessions 依所屬專案分組，每個專案顯示為一張 `ProjectCard`
- **AND** 每張 ProjectCard 標頭顯示：專案名稱、該欄中的 session 數量、平台標籤、最後更新時間
- **AND** ProjectCard 預設為展開狀態

#### Scenario: ProjectCard 收折

- **WHEN** 使用者點擊 ProjectCard 的收折按鈕
- **THEN** 隱藏 session 列表，僅顯示標頭摘要（session 數量保留可見）
- **AND** 再次點擊時展開，恢復顯示

#### Scenario: ProjectCard 展開後 session 列表

- **WHEN** ProjectCard 處於展開狀態
- **THEN** 以輕量列表行顯示每個 session：summary（截至 60 字元）、activity badge、啟動按鈕
- **AND** Active 狀態的 session 額外顯示活動細節（Thinking / Tool Call / File Op / Sub-Agent / Working）

### Requirement: Done 欄位數量限制

Done 欄位 SHALL 限制顯示數量，避免已完成 session 大量佔用畫面。

#### Scenario: Done 欄預設顯示 10 個

- **WHEN** Done 欄的 ProjectCard 總數（或 session 數）超過 10 個
- **THEN** 系統只顯示最新的 10 個
- **AND** 底部顯示「載入更多」按鈕

#### Scenario: 載入更多

- **WHEN** 使用者點擊「載入更多」按鈕，或捲動至 Done 欄底部
- **THEN** 系統追加顯示下一批（10 個）Done 狀態的項目

### Requirement: 看板欄位寬度調整

系統 SHALL 支援看板欄位寬度的手動調整與持久化。

#### Scenario: 預設平均寬度

- **WHEN** 使用者初次開啟看板視圖，或無已儲存寬度設定
- **THEN** 四欄平均分配可用寬度（各 25%）

#### Scenario: 手動調整欄位寬度

- **WHEN** 使用者拖拉欄位分隔線
- **THEN** 即時調整對應欄的寬度，並壓縮相鄰欄

#### Scenario: 欄寬持久化

- **WHEN** 使用者完成欄寬調整
- **THEN** 系統將欄寬設定儲存（localStorage 或 AppSettings）
- **AND** 下次開啟看板時恢復上次設定的欄寬
