## Purpose

定義 Cols 模式 change 列名稱區塊的寬度與截斷行為，避免底線與外框未對齊或內容溢出。

## Requirements

### Requirement: Cols 模式 change 列名稱填滿外框寬度

Cols 模式的每個 change 列（`.explorer-cols-entry`）內，名稱文字區域 SHALL 填滿外框可用寬度，底線視覺上 SHALL 與外框右側對齊。

#### Scenario: change 列名稱水平填滿

- **WHEN** Cols 模式顯示 change 列
- **THEN** `.explorer-cols-entry` 的 `box-sizing` SHALL 為 `border-box`
- **AND** `.explorer-cols-entry-name` SHALL 使用 `flex: 1` 佔滿剩餘寬度
- **AND** 名稱文字底線（若有）SHALL 延伸至外框右邊界

#### Scenario: 名稱過長時截斷

- **WHEN** change 名稱超出可用寬度
- **THEN** 系統 SHALL 使用 `text-overflow: ellipsis` 截斷
- **AND** 外框不 SHALL 橫向溢出
