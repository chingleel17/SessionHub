## ADDED Requirements

### Requirement: 共用按鈕元件

系統 SHALL 提供可重用的 Button 元件，支援至少 primary、secondary、ghost、danger 四種視覺 variant，以及 disabled 與 loading 狀態，並使用既有 theme token 呈現。

#### Scenario: 按鈕狀態一致
- **WHEN** 任一畫面使用共用 Button
- **THEN** 預設、hover、focus-visible、disabled 與 loading 狀態使用一致的圓角、色彩、間距與游標規則

#### Scenario: 按鈕無法重複觸發 loading 操作
- **WHEN** Button 處於 loading 或 disabled 狀態
- **THEN** 使用者無法以滑鼠或鍵盤觸發其 click handler

### Requirement: 共用圖示按鈕與提示

系統 SHALL 提供 IconButton 元件，且無可見文字標籤的操作必須可由鍵盤取得焦點，並具有 i18n 提供的 accessible name 與 tooltip。

#### Scenario: 使用者聚焦圖示按鈕
- **WHEN** 使用者以鍵盤聚焦 IconButton
- **THEN** 顯示清楚的 focus-visible 樣式
- **AND** 螢幕閱讀器可讀取其操作名稱

#### Scenario: 使用者 hover 圖示按鈕
- **WHEN** 使用者將指標停留於 IconButton
- **THEN** 顯示對應的 tooltip

### Requirement: 共用選擇與下拉互動

系統 SHALL 為重複使用的原生 select 與自訂 dropdown 提供共用樣式或元件，統一高度、圓角、邊框、focus-visible 與 disabled 視覺。

#### Scenario: 原生選擇欄位取得焦點
- **WHEN** 使用者以鍵盤聚焦共用 Select
- **THEN** 控制項顯示符合 theme token 的 focus-visible 樣式

#### Scenario: 自訂下拉選單關閉
- **WHEN** 使用者按下 Escape 或點擊開啟選單以外的區域
- **THEN** Dropdown 關閉並將焦點維持或回到觸發按鈕
