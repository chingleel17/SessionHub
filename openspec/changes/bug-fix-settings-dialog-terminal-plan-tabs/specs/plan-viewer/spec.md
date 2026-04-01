## MODIFIED Requirements

### Requirement: 讀取 plan.md
系統 SHALL 偵測 session 目錄下是否存在 `plan.md`，並在 session 卡片上顯示「有 plan」的視覺標示。

#### Scenario: Session 有 plan.md
- **WHEN** 掃描 session 時發現 `<sessionID>/plan.md` 存在
- **THEN** session 卡片顯示 plan 圖示標記

## ADDED Requirements

### Requirement: Plan 編輯器位於 ProjectView 子分頁
系統 SHALL 在 ProjectView 的第二層子分頁內開啟 Plan 編輯器，不在頂層分頁列新增 Plan 分頁。

#### Scenario: 從 Session 卡片開啟 Plan
- **WHEN** 使用者點擊 session 卡片上的「開啟 Plan」按鈕
- **THEN** 對應 ProjectView 的子分頁列新增 `Plan:<sessionId>` 子分頁並顯示 Plan 編輯器內容

#### Scenario: 頂層不出現 Plan 分頁
- **WHEN** 使用者開啟任意數量的 Plan 編輯器
- **THEN** 頂層分頁列仍只顯示 Dashboard 與 Project 分頁，不顯示任何 Plan 分頁
