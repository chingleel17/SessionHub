## ADDED Requirements

### Requirement: 最近活動顯示限制與專案標籤

系統 SHALL 在首頁「最近活動」清單中限制每筆 session 標題的顯示長度，並附上所屬專案名稱。

#### Scenario: Summary 超過最大顯示長度

- **WHEN** session 的 summary 長度超過 80 個字元
- **THEN** 系統截斷並附上「…」省略符號
- **AND** 完整內容可透過 `title` 屬性（tooltip）查看

#### Scenario: 顯示所屬專案名稱

- **WHEN** session 的 `cwd` 不為空
- **THEN** 系統在 session 標題後方顯示小型專案名稱標籤（取路徑最後一段）

### Requirement: 專案卡片顯示最後一次 Session 標題

系統 SHALL 在首頁「專案分頁預覽」的每個專案卡片中，顯示該專案最近一次 session 的標題。

#### Scenario: 顯示最近 session 標題

- **WHEN** 使用者瀏覽首頁專案清單
- **THEN** 系統在每個專案卡片中以小字顯示最新一筆 session 的 summary（限 60 字元）
- **AND** 若該 session 無 summary 則不顯示此欄位

### Requirement: Session 統計摘要

系統 SHALL 在 Dashboard 頁面顯示整體 session 統計資訊。

#### Scenario: 顯示統計數字

- **WHEN** 使用者切換至 Dashboard 頁面
- **THEN** 系統顯示統計卡片：總 session 數量、已封存數量、活躍專案數量、parse 錯誤數量、token 用量（含時間範圍）、互動次數（含時間範圍）
- **AND** 每個指標前顯示對應 icon
- **AND** parse 錯誤數為 0 時不顯示對應卡片

### Requirement: 統計週期切換排版

系統 SHALL 以橫向獨立列顯示「本週 / 本月」切換按鈕，位於視圖切換按鈕下方。

#### Scenario: 週期切換按鈕位置

- **WHEN** 使用者在 Dashboard 頁面
- **THEN** 「本週 / 本月」切換按鈕顯示於 Kanban/清單切換按鈕的**正下方**，統計卡片的上方
- **AND** 排版為水平（橫向）排列

#### Scenario: 切換統計週期

- **WHEN** 使用者點擊「本週」或「本月」
- **THEN** 系統重新計算並更新 token 用量與互動次數統計
- **AND** 選中的週期按鈕以 active 樣式（藍色）標示

### Requirement: 多平台 Token 與互動統計

Dashboard SHALL 顯示跨所有 provider（Copilot + OpenCode）的合計 token 用量與互動次數。

#### Scenario: 跨 provider 統計加總

- **WHEN** 使用者查看 Dashboard 且 session stats 已載入
- **THEN** Dashboard 顯示所有已載入 session 的 total output tokens（K/M 格式）與 total interaction count
- **AND** 統計值包含 Copilot 與 OpenCode session

### Requirement: 平台分佈統計

Dashboard SHALL 顯示各 provider 的 session 數量分佈。

#### Scenario: 顯示 provider 分佈

- **WHEN** 使用者查看 Dashboard
- **THEN** 系統顯示 Copilot 與 OpenCode 各自的 session 數量與佔比

### Requirement: Dashboard 視圖模式切換

系統 SHALL 提供清單視圖與 Kanban 視圖的切換按鈕，兩種視圖並列可選。

#### Scenario: 顯示視圖切換按鈕

- **WHEN** 使用者在 Dashboard 頁面
- **THEN** 系統在 Dashboard 標題區域顯示「清單」與「Kanban」切換按鈕
- **AND** 當前視圖的按鈕以 active 樣式標示
