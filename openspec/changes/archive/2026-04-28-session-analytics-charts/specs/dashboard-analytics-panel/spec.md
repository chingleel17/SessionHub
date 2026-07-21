## ADDED Requirements

### Requirement: Dashboard Analytics Panel

Dashboard SHALL 在統計卡片區塊下方顯示 Analytics Panel，以折線趨勢圖與圓餅圖呈現所有專案在當前統計週期的全域統計走勢。

#### Scenario: 進入 Dashboard 自動載入圖表

- **WHEN** 使用者切換至 Dashboard 頁面
- **THEN** Analytics Panel 自動觸發 `get_analytics_data`（`cwd: null`，時間範圍對應當前統計週期「本周 / 本月」，`groupBy: "day"`）
- **AND** 顯示全域 token 趨勢折線圖與各專案 token 佔比圓餅圖

#### Scenario: 定時自動重整

- **WHEN** 使用者在 Dashboard 頁面停留超過重整間隔（預設 30 分鐘）
- **THEN** Analytics Panel 自動重新呼叫 `get_analytics_data` 更新圖表
- **AND** 重整期間不清空現有圖表，而是在背景載入後替換

#### Scenario: 切換統計週期同步更新圖表

- **WHEN** 使用者切換「本周 / 本月」統計週期
- **THEN** Analytics Panel 同步以新時間範圍重新查詢並更新圖表

#### Scenario: 首次載入失敗

- **WHEN** `get_analytics_data` 在進入 Dashboard 後第一次呼叫失敗
- **THEN** Analytics Panel 顯示錯誤提示，提供「重試」按鈕，不影響 Dashboard 其他統計卡片的顯示

### Requirement: Analytics Panel 重整間隔設定

系統 SHALL 支援透過應用設定調整 Analytics Panel 的自動重整間隔（10 分鐘或 30 分鐘）。

#### Scenario: 設定重整間隔

- **WHEN** 使用者在設定頁面選擇 Analytics 重整間隔（10 分鐘 / 30 分鐘）
- **THEN** Dashboard Analytics Panel 依新間隔調整 `setInterval` 計時器
- **AND** 設定值持久化至 `settings.json`

#### Scenario: 預設間隔

- **WHEN** 使用者未設定過重整間隔
- **THEN** Analytics Panel 使用 30 分鐘作為預設值

### Requirement: Analytics Panel 可折疊

Dashboard Analytics Panel SHALL 可以折疊 / 展開，折疊狀態持久化至設定。

#### Scenario: 折疊 Analytics Panel

- **WHEN** 使用者點擊 Analytics Panel 的折疊按鈕
- **THEN** 圖表內容收起，只顯示標題列
- **AND** 折疊狀態寫入 `settings.json`

#### Scenario: 展開時若資料過期則重新查詢

- **WHEN** 使用者展開已折疊的 Analytics Panel 且距上次查詢超過重整間隔
- **THEN** 自動觸發重新查詢
