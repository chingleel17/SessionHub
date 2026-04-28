## ADDED Requirements

### Requirement: TrendChart 折線趨勢圖元件

系統 SHALL 提供 `TrendChart` React 元件，接受時序資料陣列並以純 SVG 渲染折線趨勢圖，支援多條折線（token、互動次數、點數）同時顯示。

#### Scenario: 正常渲染折線

- **WHEN** `TrendChart` 收到含有 2 個以上資料點的 `data: AnalyticsDataPoint[]` props
- **THEN** 元件以 SVG 渲染折線圖，X 軸為時間標籤（label），Y 軸為數值
- **AND** 各折線以不同顏色區分（outputTokens、interactionCount、costPoints）

#### Scenario: 單一資料點

- **WHEN** `data` 只有 1 個元素
- **THEN** 元件以單點（圓點）顯示，不繪製折線

#### Scenario: 空資料

- **WHEN** `data` 為空陣列
- **THEN** 元件顯示「無資料」空狀態文字，不顯示空 SVG 框架

#### Scenario: 無障礙支援

- **WHEN** 元件渲染
- **THEN** SVG 根元素 SHALL 帶有 `role="img"` 與 `aria-label` 描述折線圖標題

### Requirement: PieChart 圓餅分布圖元件

系統 SHALL 提供 `PieChart` React 元件，接受標籤與數值陣列，以純 SVG 渲染圓餅圖並附上圖例。

#### Scenario: 正常渲染圓餅

- **WHEN** `PieChart` 收到含有 2 個以上切片的 `slices: { label: string; value: number; color: string }[]` props
- **THEN** 元件以 SVG 繪製圓餅，各切片按比例劃分
- **AND** 每個切片旁或下方顯示標籤與百分比

#### Scenario: 全部數值為 0

- **WHEN** 所有 `slices.value` 均為 0
- **THEN** 元件顯示「無資料」空狀態，不嘗試繪製圓餅

#### Scenario: 單一切片（100%）

- **WHEN** 只有一個 `slices` 且 value > 0
- **THEN** 繪製完整圓形，附上 100% 標籤

#### Scenario: 無障礙支援

- **WHEN** 元件渲染
- **THEN** SVG 根元素 SHALL 帶有 `role="img"` 與 `aria-label`

### Requirement: 圖表顏色跟隨應用主題

圖表元件 SHALL 使用 CSS 變數定義顏色，以自動適配深色 / 淺色主題切換。

#### Scenario: 深色主題下圖表清晰可辨

- **WHEN** 應用切換至深色主題
- **THEN** TrendChart 與 PieChart 的線條、填色、文字顏色透過 CSS 變數自動調整，不出現白底白字或黑底黑線

### Requirement: 可選折線顯示切換

TrendChart SHALL 提供切換控制，讓使用者選擇顯示哪些折線指標。

#### Scenario: 切換指標顯示

- **WHEN** 使用者點擊折線圖下方的指標切換按鈕（outputTokens / interactionCount / costPoints）
- **THEN** 對應折線顯示或隱藏
- **AND** 至少保留一條折線顯示（不允許全部隱藏）
