## MODIFIED Requirements

### Requirement: Dashboard Analytics Panel

Dashboard SHALL 在統計卡片區塊下方顯示 Analytics Panel，以折線趨勢圖與圓餅圖呈現所有專案在當前統計週期的全域統計走勢，並可在同一區域顯示 provider quota overview。

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

## ADDED Requirements

### Requirement: Dashboard 顯示 provider quota overview

Dashboard SHALL 顯示 provider quota overview，讓使用者在同一畫面查看主要平台的剩餘用量、資料來源與最後刷新時間。

#### Scenario: 顯示 quota overview
- **WHEN** Dashboard 載入且 quota monitoring 已啟用
- **THEN** 系統顯示各 provider 的 quota summary
- **AND** 包含至少 provider 名稱、狀態與最後刷新時間

#### Scenario: 手動刷新 quota overview
- **WHEN** 使用者在 Dashboard 點擊 quota refresh
- **THEN** 系統重新查詢 provider quota snapshots
- **AND** 用新結果更新 quota overview
