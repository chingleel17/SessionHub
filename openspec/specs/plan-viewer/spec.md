## ADDED Requirements

### Requirement: 讀取 plan.md

系統 SHALL 偵測 session 目錄下是否存在 `plan.md`，並在 session 卡片上顯示「有 plan」的視覺標示。

#### Scenario: Session 有 plan.md

- **WHEN** 掃描 session 時發現 `<sessionID>/plan.md` 存在
- **THEN** session 卡片顯示 plan 圖示標記

### Requirement: Plan 在 ProjectView 子分頁中顯示

Plan 內容 SHALL 以 ProjectView 內的子分頁（sub-tab）呈現，不佔用頂層分頁列位置。

#### Scenario: 使用者開啟 plan

- **WHEN** 使用者點擊 sessionCard 上的 plan 圖示，或點擊 ProjectView 內「+ Plan」按鈕
- **THEN** ProjectView 的子分頁列新增一個以 session ID 為 key 的 plan 分頁
- **AND** 切換至該分頁顯示 plan.md 內容

#### Scenario: 子分頁可關閉

- **WHEN** 使用者點擊 plan 子分頁上的關閉（×）按鈕
- **THEN** 該子分頁從列表移除
- **AND** 視圖切回 Sessions 子分頁

#### Scenario: 跨專案狀態保留

- **WHEN** 使用者切換至其他專案再切回
- **THEN** 已開啟的 plan 子分頁 SHALL 仍然存在（不因切換而關閉）

### Requirement: Plan 編輯器雙欄等高佈局

Plan 檢視器 SHALL 使用雙欄等高佈局顯示 Markdown 原文與預覽。

#### Scenario: 雙欄顯示

- **WHEN** plan 子分頁開啟
- **THEN** 左欄顯示可編輯的 Markdown 原文，右欄顯示即時渲染的 HTML 預覽
- **AND** 兩欄高度一致、可獨立滾動

#### Scenario: Plan 儲存

- **WHEN** 使用者修改 plan 原文後按下儲存
- **THEN** 系統將內容寫回 `<sessionID>/plan.md`
- **AND** 右欄預覽同步更新
